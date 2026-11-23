use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::Mutex;

use leaf_core::core::{block_pos::BlockPos, holder::Holder};
use leaf_core::entity::position_move_rotation::PositionMoveRotation;
use leaf_core::network::protocol::game::{
    clientbound_add_entity_packet::ClientboundAddEntityPacket,
    clientbound_entity_position_sync_packet::ClientboundEntityPositionSyncPacket,
    clientbound_remove_entities_packet::ClientboundRemoveEntitiesPacket,
    clientbound_take_item_entity_packet::ClientboundTakeItemEntityPacket,
};
use leaf_core::util::uuid::Uuid;
use leaf_core::world::{item::item_stack::ItemStack, phys::vec3::Vec3};

use crate::gameplay::{item_data, PlayContext};
use crate::items::ITEM_ENTITY;
use crate::wire::Wire;
use crate::world::{chunk, FlatWorld, AIR};

#[derive(Default)]
pub struct Drops(Mutex<Vec<DroppedItem>>);

#[derive(Clone)]
struct DroppedItem {
    id: i32,
    uuid: Uuid,
    motion: PositionMoveRotation,
    on_ground: bool,
    stack: ItemStack,
    created: i64,
}

impl Drops {
    pub fn spawn(&self, id: i32, stack: ItemStack, pos: Vec3, velocity: Vec3, created: i64) {
        self.0.lock().unwrap().push(DroppedItem {
            id,
            uuid: Uuid::random(),
            motion: PositionMoveRotation::new(pos, velocity, 0.0, 0.0),
            on_ground: false,
            stack,
            created,
        });
    }

    pub fn update(&self, tick: i64, world: &FlatWorld) {
        self.0.lock().unwrap().retain_mut(|item| {
            if tick - item.created >= 6_000 || item.motion.position.y < -72.0 {
                return false;
            }
            let motion = &mut item.motion;
            let next = motion.position.add(
                motion.delta_movement.x,
                motion.delta_movement.y,
                motion.delta_movement.z,
            );
            let block = block_at(next.add(0.0, -0.01, 0.0));
            item.on_ground = motion.delta_movement.y <= 0.0 && world.block(block) != AIR;
            if item.on_ground {
                motion.position = Vec3::new(next.x, f64::from(block.y + 1), next.z);
                motion.delta_movement = Vec3::ZERO;
            } else {
                motion.position = next;
                motion.delta_movement = motion.delta_movement.add(0.0, -0.04, 0.0).scale(0.98);
            }
            true
        });
    }
}

#[derive(Default)]
pub struct DropTracker(HashMap<i32, (PositionMoveRotation, i32)>);

impl DropTracker {
    pub fn sync(
        &mut self,
        ctx: &PlayContext<'_>,
        loaded: &HashSet<(i32, i32)>,
        player: Vec3,
        collector: i32,
        tick: i64,
        mut insert: impl FnMut(&mut ItemStack),
        wire: &mut Wire,
    ) -> io::Result<()> {
        let mut picked_up = Vec::new();
        let visible = {
            let mut items = ctx.drops.0.lock().unwrap();
            items.retain_mut(|item| {
                let position = item.motion.position;
                if collector != 0
                    && tick - item.created >= 40
                    && position.distance_to_sqr(&player.add(0.0, 0.5, 0.0)) < 1.44
                {
                    let before = item.stack.get_count();
                    insert(&mut item.stack);
                    let collected = before - item.stack.get_count();
                    if collected > 0 && self.0.contains_key(&item.id) {
                        picked_up.push((item.id, collected));
                    }
                }
                !item.stack.is_empty()
            });
            items
                .iter()
                .filter(|item| {
                    let pos = item.motion.position;
                    loaded.contains(&chunk(block_at(pos)))
                        && player.distance_to_sqr(&pos) <= 64.0 * 64.0
                })
                .cloned()
                .collect::<Vec<_>>()
        };

        for (id, count) in picked_up {
            wire.send(
                ctx.ids,
                &ClientboundTakeItemEntityPacket::new(id, collector, count),
            )?;
        }

        let mut previous = std::mem::take(&mut self.0);
        for item in visible {
            let pos = item.motion.position;
            let count = item.stack.get_count();
            match previous.remove(&item.id) {
                Some((motion, old_count)) => {
                    if motion != item.motion {
                        wire.send(
                            ctx.ids,
                            &ClientboundEntityPositionSyncPacket::new(
                                item.id,
                                item.motion,
                                item.on_ground,
                            ),
                        )?;
                    }
                    if count != old_count {
                        wire.send(ctx.ids, &item_data(item.id, item.stack))?;
                    }
                }
                None => {
                    wire.send(
                        ctx.ids,
                        &ClientboundAddEntityPacket::new(
                            item.id,
                            item.uuid,
                            Holder::Reference(ITEM_ENTITY),
                            pos.x,
                            pos.y,
                            pos.z,
                            item.motion.delta_movement,
                            0.0,
                            0.0,
                            0.0,
                            0,
                        ),
                    )?;
                    wire.send(ctx.ids, &item_data(item.id, item.stack))?;
                }
            }
            self.0.insert(item.id, (item.motion, count));
        }
        if !previous.is_empty() {
            wire.send(
                ctx.ids,
                &ClientboundRemoveEntitiesPacket::new(previous.into_keys().collect()),
            )?;
        }
        Ok(())
    }

    pub fn reset(&mut self) {
        self.0.clear();
    }
}

fn block_at(pos: Vec3) -> BlockPos {
    BlockPos::new(
        pos.x.floor() as i32,
        pos.y.floor() as i32,
        pos.z.floor() as i32,
    )
}
