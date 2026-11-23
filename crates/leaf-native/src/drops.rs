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

use crate::audience::Subscription;
use crate::blocks::Blocks;
use crate::gameplay::{item_data, PlayContext};
use crate::items::ITEM_ENTITY;
use crate::wire::Wire;
use crate::workers::Workers;
use crate::world::{chunk, BlockView, NativeWorld};

const DROP_WORK_BATCH: usize = 256;

#[derive(Clone, Copy)]
pub enum PickupDelay {
    Loot,
    Thrown,
}

impl PickupDelay {
    fn ticks(self) -> i64 {
        match self {
            Self::Loot => 10,
            Self::Thrown => 40,
        }
    }
}

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
    pickup_at: i64,
}

impl Drops {
    pub fn spawn(
        &self,
        id: i32,
        stack: ItemStack,
        pos: Vec3,
        velocity: Vec3,
        created: i64,
        delay: PickupDelay,
    ) {
        self.0.lock().unwrap().push(DroppedItem {
            id,
            uuid: Uuid::random(),
            motion: PositionMoveRotation::new(pos, velocity, 0.0, 0.0),
            on_ground: false,
            stack,
            created,
            pickup_at: created + delay.ticks(),
        });
    }

    pub fn update(&self, tick: i64, world: &NativeWorld, workers: &Workers) {
        let mut items = self.0.lock().unwrap();
        if items.is_empty() {
            return;
        }
        let world = world.snapshot();
        if workers.can_split(items.len(), DROP_WORK_BATCH) {
            let inputs = items
                .iter()
                .map(|item| (item.motion, item.created))
                .collect::<Vec<_>>();
            let mut updates = workers
                .map_ranges(inputs.len(), DROP_WORK_BATCH, move |range| {
                    inputs[range]
                        .iter()
                        .map(|&(motion, created)| step_drop(tick, created, motion, &world))
                        .collect::<Vec<_>>()
                })
                .into_iter()
                .flatten();
            items.retain_mut(|item| item.apply_motion(updates.next().unwrap()));
        } else {
            items.retain_mut(|item| {
                item.apply_motion(step_drop(tick, item.created, item.motion, &world))
            });
        }
    }
}

impl DroppedItem {
    fn apply_motion(&mut self, update: Option<(PositionMoveRotation, bool)>) -> bool {
        if let Some((motion, on_ground)) = update {
            self.motion = motion;
            self.on_ground = on_ground;
            true
        } else {
            false
        }
    }
}

fn step_drop(
    tick: i64,
    created: i64,
    mut motion: PositionMoveRotation,
    world: &BlockView,
) -> Option<(PositionMoveRotation, bool)> {
    if tick - created >= 6_000 || motion.position.y < -72.0 {
        return None;
    }
    let next = motion.position.add_vec(&motion.delta_movement);
    let block = block_at(next.add(0.0, -0.01, 0.0));
    let on_ground =
        motion.delta_movement.y <= 0.0 && !Blocks::vanilla().get(world.block(block)).replaceable;
    if on_ground {
        motion.position = Vec3::new(next.x, f64::from(block.y + 1), next.z);
        motion.delta_movement = Vec3::ZERO;
    } else {
        motion.position = next;
        motion.delta_movement = motion.delta_movement.add(0.0, -0.04, 0.0).scale(0.98);
    }
    Some((motion, on_ground))
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
        subscription: &Subscription<'_>,
    ) -> io::Result<()> {
        let mut picked_up = Vec::new();
        let visible = {
            let mut items = ctx.drops.0.lock().unwrap();
            items.retain_mut(|item| {
                let position = item.motion.position;
                let relative = player.vector_to(&position);
                if collector != 0
                    && tick >= item.pickup_at
                    && relative.x.abs() < 1.425
                    && relative.z.abs() < 1.425
                    && (-0.75..2.3).contains(&relative.y)
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

        // A block update may have arrived while we were collecting drops.
        // Send it before the corresponding item spawn or pickup.
        subscription.flush(wire, ctx.ids, loaded)?;
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
