use std::collections::{HashMap, HashSet};
use std::io;

use leaf_core::core::holder::Holder;
use leaf_core::entity::position_move_rotation::PositionMoveRotation;
use leaf_core::network::protocol::game::{
    clientbound_add_entity_packet::ClientboundAddEntityPacket,
    clientbound_entity_event_packet::ClientboundEntityEventPacket,
    clientbound_entity_position_sync_packet::ClientboundEntityPositionSyncPacket,
    clientbound_hurt_animation_packet::ClientboundHurtAnimationPacket,
    clientbound_remove_entities_packet::ClientboundRemoveEntitiesPacket,
    clientbound_set_entity_data_packet::ClientboundSetEntityDataPacket,
};
use leaf_core::network::syncher::synched_entity_data::DataValue;
use leaf_core::world::phys::vec3::Vec3;

use crate::audience::Subscription;
use crate::gameplay::PlayContext;
use crate::wire::{PacketIds, Wire};

use super::{at_chunk, Chunk, Mob, Scene, TRACKED_ADDITIONS_PER_TICK};

#[derive(Default)]
pub struct MobTracker {
    known: HashMap<i32, SentMob>,
    chunks: HashMap<Chunk, TrackedChunk>,
}

#[derive(Default)]
struct TrackedChunk {
    revision: Option<u64>,
    sent: HashSet<i32>,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct SentMob {
    pub motion: PositionMoveRotation,
    hurt: u32,
    on_ground: bool,
    burning: bool,
}

impl From<Mob> for SentMob {
    fn from(mob: Mob) -> Self {
        Self {
            motion: mob.motion,
            hurt: mob.hurt,
            on_ground: mob.on_ground,
            burning: mob.burning,
        }
    }
}

#[derive(Default)]
pub(super) struct Changes {
    pub added: Vec<(i32, Mob)>,
    pub moved: Vec<(i32, SentMob)>,
    pub hurt: Vec<(i32, bool)>,
    pub fire: Vec<(i32, bool)>,
    pub removed: Vec<i32>,
}

impl MobTracker {
    pub fn sees(&self, id: i32) -> bool {
        self.known.contains_key(&id)
    }

    pub fn reset(&mut self, wire: &mut Wire, ids: &PacketIds) -> io::Result<()> {
        let removed = self.known.keys().copied().collect::<Vec<_>>();
        send_removals(&removed, wire, ids)?;
        self.known.clear();
        self.chunks.clear();
        Ok(())
    }

    pub fn sync(
        &mut self,
        position: Vec3,
        loaded: &HashSet<Chunk>,
        ctx: &PlayContext<'_>,
        wire: &mut Wire,
        subscription: &Subscription<'_>,
    ) -> io::Result<()> {
        let changes = self.collect(&ctx.mobs.0.read().unwrap(), loaded, at_chunk(position));
        // A world update can be queued while the tracker waits for the scene lock.
        // Deliver it before the entity packets from that scene snapshot.
        subscription.flush(wire, ctx.ids, loaded)?;
        changes.send(wire, ctx.ids, ctx.commands.game_time())
    }

    pub(super) fn collect(
        &mut self,
        scene: &Scene,
        loaded: &HashSet<Chunk>,
        center: Chunk,
    ) -> Changes {
        let mut changes = Changes::default();
        let obsolete = self
            .chunks
            .keys()
            .copied()
            .filter(|coord| !loaded.contains(coord) || !scene.chunks.contains_key(coord))
            .collect::<Vec<_>>();
        let mut dirty = scene
            .chunks
            .iter()
            .filter_map(|(&coord, chunk)| {
                (loaded.contains(&coord)
                    && self.chunks.get(&coord).and_then(|seen| seen.revision)
                        != Some(chunk.revision))
                .then_some((coord, chunk))
            })
            .collect::<Vec<_>>();

        // A mob crossing between loaded chunks keeps the same client entity ID.
        for coord in obsolete {
            let tracked = self.chunks.remove(&coord).unwrap();
            for id in tracked.sent {
                if let Some(destination) = self.refresh(id, scene, loaded, &mut changes) {
                    self.chunks.entry(destination).or_default().sent.insert(id);
                }
            }
        }
        for &(coord, _) in &dirty {
            if let Some(tracked) = self.chunks.get_mut(&coord) {
                let mut sent = std::mem::take(&mut tracked.sent);
                sent.retain(|&id| {
                    let Some(destination) = self.refresh(id, scene, loaded, &mut changes) else {
                        return false;
                    };
                    if coord == destination {
                        return true;
                    }
                    self.chunks.entry(destination).or_default().sent.insert(id);
                    false
                });
                self.chunks.get_mut(&coord).unwrap().sent = sent;
            }
        }

        // Primed TNT has a short lifetime and should arrive before queued mobs.
        for &id in &scene.explosives {
            let mob = scene.entities[&id];
            let coord = at_chunk(mob.motion.position);
            if loaded.contains(&coord) && !self.sees(id) {
                self.add(id, mob, coord, &mut changes);
            }
        }
        dirty.sort_unstable_by_key(|&((x, z), _)| {
            let dx = x - center.0;
            let dz = z - center.1;
            (dx * dx + dz * dz, x, z)
        });
        let mut remaining = TRACKED_ADDITIONS_PER_TICK;
        for (coord, chunk) in dirty {
            let mut complete = true;
            for &id in &chunk.ids {
                if self.sees(id) {
                    continue;
                }
                let mob = scene.entities[&id];
                if mob.dying.is_some() {
                    continue;
                }
                if remaining == 0 {
                    complete = false;
                    break;
                }
                remaining -= 1;
                self.add(id, mob, coord, &mut changes);
            }
            if complete {
                self.chunks.entry(coord).or_default().revision = Some(chunk.revision);
            } else {
                // Keep the old revision so deferred additions are retried.
                break;
            }
        }
        changes
    }

    fn refresh(
        &mut self,
        id: i32,
        scene: &Scene,
        loaded: &HashSet<Chunk>,
        changes: &mut Changes,
    ) -> Option<Chunk> {
        if let Some(mob) = scene.entities.get(&id) {
            let coord = at_chunk(mob.motion.position);
            if loaded.contains(&coord) {
                let sent = self.known.get_mut(&id).unwrap();
                let next = SentMob::from(*mob);
                if sent.motion != next.motion || sent.on_ground != next.on_ground {
                    changes.moved.push((id, next));
                }
                if sent.hurt != next.hurt {
                    changes.hurt.push((id, mob.dying.is_some()));
                }
                if sent.burning != next.burning {
                    changes.fire.push((id, next.burning));
                }
                *sent = next;
                return Some(coord);
            }
        }
        self.known.remove(&id);
        changes.removed.push(id);
        None
    }

    fn add(&mut self, id: i32, mob: Mob, coord: Chunk, changes: &mut Changes) {
        self.known.insert(id, SentMob::from(mob));
        self.chunks.entry(coord).or_default().sent.insert(id);
        changes.added.push((id, mob));
    }
}

impl Changes {
    fn send(self, wire: &mut Wire, ids: &PacketIds, now: i64) -> io::Result<()> {
        send_removals(&self.removed, wire, ids)?;
        for (id, mob) in self.added {
            let p = mob.motion.position;
            wire.send(
                ids,
                &ClientboundAddEntityPacket::new(
                    id,
                    mob.uuid,
                    Holder::Reference(mob.kind.id),
                    p.x,
                    p.y,
                    p.z,
                    mob.motion.delta_movement,
                    mob.motion.x_rot,
                    mob.motion.y_rot,
                    f64::from(mob.motion.y_rot),
                    0,
                ),
            )?;
            if mob.burning {
                send_fire_state(wire, ids, id, true)?;
            }
            if let Some(fuse) = mob.fuse {
                let ticks = (fuse - now).max(1) as i32;
                let value = DataValue {
                    id: 8,
                    serializer: 1,
                    value: Box::new(ticks),
                };
                wire.send(ids, &ClientboundSetEntityDataPacket::new(id, vec![value]))?;
            }
        }
        for (id, state) in self.moved {
            wire.send(
                ids,
                &ClientboundEntityPositionSyncPacket::new(id, state.motion, state.on_ground),
            )?;
        }
        for (id, burning) in self.fire {
            send_fire_state(wire, ids, id, burning)?;
        }
        for (id, dying) in self.hurt {
            if dying {
                wire.send(ids, &ClientboundEntityEventPacket::new(id, 3))?;
            } else {
                wire.send(ids, &ClientboundHurtAnimationPacket::new(id, 0.0))?;
            }
        }
        Ok(())
    }
}

fn send_fire_state(wire: &mut Wire, ids: &PacketIds, entity: i32, burning: bool) -> io::Result<()> {
    wire.send(
        ids,
        &ClientboundSetEntityDataPacket::new(
            entity,
            vec![DataValue {
                id: 0,
                serializer: 0,
                value: Box::new(i8::from(burning)),
            }],
        ),
    )
}

fn send_removals(removed: &[i32], wire: &mut Wire, ids: &PacketIds) -> io::Result<()> {
    for batch in removed.chunks(1024) {
        wire.send(ids, &ClientboundRemoveEntitiesPacket::new(batch.to_vec()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicI32;

    use super::*;
    use crate::mob_types;
    use crate::mobs::Mobs;

    #[test]
    fn keeps_an_entity_id_when_it_moves_between_visible_chunks() {
        let mobs = Mobs::default();
        let mut scene = mobs.0.write().unwrap();
        let mut tracker = MobTracker::default();
        let origin = (0, 0);
        let edge = (32, 0);
        let mut loaded = HashSet::from([origin, edge]);
        let id = scene.add(
            mob_types::named("pig").unwrap(),
            Vec3::new(0.5, 16.0, 0.5),
            false,
            None,
            &AtomicI32::new(1),
        );
        assert_eq!(tracker.collect(&scene, &loaded, origin).added[0].0, id);

        scene.entities.get_mut(&id).unwrap().motion.position.x = 512.5;
        scene.unindex(id, origin);
        scene.index(id, edge);
        let changes = tracker.collect(&scene, &loaded, origin);
        assert!(changes.added.is_empty());
        assert!(changes.removed.is_empty());
        assert_eq!(changes.moved[0].0, id);
        assert!(tracker.sees(id));

        loaded.remove(&edge);
        assert_eq!(tracker.collect(&scene, &loaded, origin).removed, [id]);
        assert!(!tracker.sees(id));
        loaded.insert(edge);
        assert_eq!(tracker.collect(&scene, &loaded, origin).added[0].0, id);
    }
}
