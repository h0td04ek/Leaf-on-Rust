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
use crate::items::{fire_resistant, ITEM_ENTITY};
use crate::player_environment::LAVA_FIRE_TICKS;
use crate::wire::Wire;
use crate::workers::Workers;
use crate::world::{chunk, entity_bounds, BlockView, Fluid, NativeWorld};

const DROP_WORK_BATCH: usize = 256;
const ITEM_SIZE: f64 = 0.25;

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
    physics: DropPhysics,
    stack: ItemStack,
    created: i64,
    pickup_at: i64,
}

#[derive(Clone, Copy)]
struct DropPhysics {
    motion: PositionMoveRotation,
    on_ground: bool,
    health: u8,
    fire_ticks: u16,
    fire_resistant: bool,
}

impl DropPhysics {
    fn new(pos: Vec3, velocity: Vec3, fire_resistant: bool) -> Self {
        Self {
            motion: PositionMoveRotation::new(pos, velocity, 0.0, 0.0),
            on_ground: false,
            health: 5,
            fire_ticks: 0,
            fire_resistant,
        }
    }

    fn burning(&self) -> bool {
        self.fire_ticks > 0
    }
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
            physics: DropPhysics::new(pos, velocity, fire_resistant(&stack)),
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
                .map(|item| (item.physics, item.created))
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
                item.apply_motion(step_drop(tick, item.created, item.physics, &world))
            });
        }
    }
}

impl DroppedItem {
    fn apply_motion(&mut self, update: Option<DropPhysics>) -> bool {
        if let Some(physics) = update {
            self.physics = physics;
            true
        } else {
            false
        }
    }
}

fn step_drop(
    tick: i64,
    created: i64,
    mut physics: DropPhysics,
    world: &BlockView,
) -> Option<DropPhysics> {
    let motion = &mut physics.motion;
    if tick - created >= 6_000 || motion.position.y < -72.0 {
        return None;
    }
    let bounds = entity_bounds(motion.position, ITEM_SIZE, ITEM_SIZE);
    let current = world.push_by_fluid(bounds, motion.delta_movement);
    if !physics.fire_resistant {
        if current.in_lava {
            physics.health = physics.health.saturating_sub(4);
            physics.fire_ticks = LAVA_FIRE_TICKS;
        } else if current.in_water {
            physics.fire_ticks = 0;
        } else if physics.fire_ticks > 0 {
            if physics.fire_ticks % 20 == 0 {
                physics.health = physics.health.saturating_sub(1);
            }
            physics.fire_ticks -= 1;
        }
        if physics.health == 0 {
            return None;
        }
    }
    motion.delta_movement = world.clip_horizontal(bounds, current.velocity, false);
    let fluid_drag = match world.fluid_at(motion.position.add(0.0, 0.11, 0.0)) {
        Some(Fluid::Water) => Some(0.99),
        Some(Fluid::Lava) => Some(0.95),
        None => None,
    };
    if let Some(drag) = fluid_drag {
        let lift = if motion.delta_movement.y < 0.06 {
            0.0005
        } else {
            0.0
        };
        motion.delta_movement = motion
            .delta_movement
            .multiply(drag, 1.0, drag)
            .add(0.0, lift, 0.0);
    }
    let next = motion.position.add_vec(&motion.delta_movement);
    let block = block_at(next.add(0.0, -0.01, 0.0));
    physics.on_ground =
        motion.delta_movement.y <= 0.0 && !Blocks::vanilla().get(world.block(block)).replaceable;
    if physics.on_ground {
        motion.position = Vec3::new(next.x, f64::from(block.y + 1), next.z);
        motion.delta_movement = Vec3::ZERO;
    } else {
        motion.position = next;
        if fluid_drag.is_none() {
            motion.delta_movement = motion.delta_movement.add(0.0, -0.04, 0.0).scale(0.98);
        }
    }
    Some(physics)
}

#[derive(Default)]
pub struct DropTracker(HashMap<i32, (PositionMoveRotation, i32, bool)>);

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
                let position = item.physics.motion.position;
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
                    let pos = item.physics.motion.position;
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
            let motion = item.physics.motion;
            let pos = motion.position;
            let burning = item.physics.burning();
            let count = item.stack.get_count();
            match previous.remove(&item.id) {
                Some((old_motion, old_count, was_burning)) => {
                    if old_motion != motion {
                        wire.send(
                            ctx.ids,
                            &ClientboundEntityPositionSyncPacket::new(
                                item.id,
                                motion,
                                item.physics.on_ground,
                            ),
                        )?;
                    }
                    if count != old_count || burning != was_burning {
                        let flags = (burning != was_burning).then_some(burning);
                        wire.send(ctx.ids, &item_data(item.id, item.stack, flags))?;
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
                            motion.delta_movement,
                            0.0,
                            0.0,
                            0.0,
                            0,
                        ),
                    )?;
                    wire.send(
                        ctx.ids,
                        &item_data(item.id, item.stack, burning.then_some(true)),
                    )?;
                }
            }
            self.0.insert(item.id, (motion, count, burning));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audience::Audience;
    use crate::world::FillMode;

    #[test]
    fn ordinary_drops_burn_but_netherite_floats_in_lava() {
        use crate::items::{named_id, stack, Items};

        Items::vanilla();
        let world = NativeWorld::new(0);
        let pos = BlockPos::new(0, 16, 0);
        let position = pos.get_bottom_center().add(0.0, 0.2, 0.0);
        let lava = Blocks::vanilla().named("lava").unwrap().default;
        world
            .fill(pos, pos, lava, FillMode::Keep, 1, &Audience::default())
            .unwrap();
        let drops = Drops::default();
        for (id, name) in [(1, "cobblestone"), (2, "netherite_ingot")] {
            drops.spawn(
                id,
                stack(named_id(name), 1),
                position,
                Vec3::ZERO,
                0,
                PickupDelay::Thrown,
            );
        }
        let workers = Workers::new(1).unwrap();
        drops.update(1, &world, &workers);
        {
            let items = drops.0.lock().unwrap();
            assert!(items[0].physics.burning());
            assert_eq!(items[0].physics.health, 1);
            assert!(!items[1].physics.burning());
        }
        drops.update(2, &world, &workers);
        let items = drops.0.lock().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, 2);
        assert!(items[0].physics.motion.position.y > position.y);
    }

    #[test]
    fn dropped_items_rise_in_water_and_follow_current_until_the_bank() {
        let world = NativeWorld::new(0);
        let water = Blocks::vanilla().named("water").unwrap();
        let pos = BlockPos::new(0, 16, 0);
        let audience = Audience::default();
        for (block, state) in [
            (pos.offset(-1, 0, 0), water.default),
            (pos, water.state(water.default, &[("level", "1")])),
            (pos.offset(1, 0, 0), 1),
        ] {
            world
                .fill(block, block, state, FillMode::Keep, 1, &audience)
                .unwrap();
        }
        let position = pos.get_bottom_center().add(0.0, 0.2, 0.0);
        let physics = DropPhysics::new(position, Vec3::ZERO, false);
        let world = world.snapshot();
        let updated = step_drop(1, 0, physics, &world).unwrap();
        assert!(updated.motion.position.x > position.x);
        assert!(updated.motion.position.y > position.y);
        assert!(!updated.on_ground);

        let bank = Vec3::new(0.875, position.y, position.z);
        let physics = DropPhysics::new(bank, Vec3::ZERO, false);
        let updated = step_drop(1, 0, physics, &world).unwrap();
        assert_eq!(updated.motion.position.x, bank.x);
        assert!(updated.motion.position.y > bank.y);
    }
}
