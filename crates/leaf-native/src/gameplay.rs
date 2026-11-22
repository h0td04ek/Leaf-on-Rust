use std::io;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::{Duration, Instant};

use leaf_core::core::{block_pos::BlockPos, direction::Direction, holder::Holder};
use leaf_core::entity::{pose::Pose, position_move_rotation::PositionMoveRotation};
use leaf_core::network::protocol::game::{
    clientbound_add_entity_packet::ClientboundAddEntityPacket,
    clientbound_block_changed_ack_packet::ClientboundBlockChangedAckPacket,
    clientbound_block_update_packet::ClientboundBlockUpdatePacket,
    clientbound_entity_event_packet::ClientboundEntityEventPacket,
    clientbound_entity_position_sync_packet::ClientboundEntityPositionSyncPacket,
    clientbound_remove_entities_packet::ClientboundRemoveEntitiesPacket,
    clientbound_set_entity_data_packet::ClientboundSetEntityDataPacket,
    clientbound_set_player_inventory_packet::ClientboundSetPlayerInventoryPacket,
    clientbound_take_item_entity_packet::ClientboundTakeItemEntityPacket,
};
use leaf_core::network::syncher::synched_entity_data::DataValue;
use leaf_core::resources::Identifier;
use leaf_core::util::uuid::Uuid;
use leaf_core::world::item::{component::fireworks::Fireworks, item_stack::ItemStack};
use leaf_core::world::phys::vec3::Vec3;

use crate::audience::Audience;
use crate::commands::Commands;
use crate::items::{item_id, stack, Items, ELYTRA, FIREWORK, FIREWORK_ENTITY, ITEM_ENTITY};
use crate::wire::{invalid, Cursor, PacketIds, Wire};
use crate::world::{FlatWorld, AIR, SPAWN_Y};

const CHEST: usize = 38;
const OFFHAND: usize = 40;
const TICK: Duration = Duration::from_millis(50);

pub struct PlayContext<'a> {
    pub world: &'a FlatWorld,
    pub ids: &'a PacketIds,
    pub audience: &'a Audience,
    pub commands: &'a Commands,
    pub entity_ids: &'a AtomicI32,
}

pub struct Gameplay {
    pub position: Vec3,
    entity_id: i32,
    inventory: [ItemStack; 41],
    selection: usize,
    pitch: f32,
    yaw: f32,
    on_ground: bool,
    creative_flying: bool,
    gliding: bool,
    drops: Vec<DropEntity>,
    rockets: Vec<(i32, Instant)>,
    last_tick: Instant,
}

struct DropEntity {
    id: i32,
    position: Vec3,
    velocity: Vec3,
    stack: ItemStack,
    created: Instant,
}

impl Gameplay {
    pub fn new(entity_id: i32) -> Self {
        Self {
            entity_id,
            position: Vec3::new(0.5, f64::from(SPAWN_Y), 0.5),
            inventory: std::array::from_fn(|_| ItemStack::empty()),
            selection: 0,
            pitch: 0.0,
            yaw: 0.0,
            on_ground: true,
            creative_flying: false,
            gliding: false,
            drops: vec![],
            rockets: vec![],
            last_tick: Instant::now(),
        }
    }

    pub fn movement(
        &mut self,
        kind: &str,
        mut body: Cursor,
        wire: &mut Wire,
        ids: &PacketIds,
    ) -> io::Result<()> {
        if matches!(kind, "move_player_pos" | "move_player_pos_rot") {
            let position = Vec3::new(
                f64::from_be_bytes(body.fixed()?),
                f64::from_be_bytes(body.fixed()?),
                f64::from_be_bytes(body.fixed()?),
            );
            if ![position.x, position.y, position.z]
                .iter()
                .all(|v| v.is_finite() && v.abs() <= 30_000_000.0)
            {
                return Err(invalid("invalid player position"));
            }
            self.position = position;
        }
        if matches!(kind, "move_player_rot" | "move_player_pos_rot") {
            self.yaw = f32::from_be_bytes(body.fixed()?);
            self.pitch = f32::from_be_bytes(body.fixed()?);
            if !self.yaw.is_finite() || !self.pitch.is_finite() {
                return Err(invalid("invalid player rotation"));
            }
        }
        self.on_ground = body.fixed::<1>()?[0] & 1 != 0;
        body.finish()?;
        if self.on_ground && self.gliding {
            self.set_gliding(false, wire, ids)?;
        }
        Ok(())
    }

    pub fn handle(
        &mut self,
        name: &str,
        mut body: Cursor,
        wire: &mut Wire,
        ctx: &PlayContext<'_>,
    ) -> io::Result<()> {
        match name {
            "set_carried_item" => {
                let slot = i16::from_be_bytes(body.fixed()?);
                body.finish()?;
                if (0..9).contains(&slot) {
                    self.selection = slot as usize;
                }
            }
            "set_creative_mode_slot" => {
                let slot = i16::from_be_bytes(body.fixed()?);
                let stack = Items::vanilla().read_creative_stack(&mut body)?;
                body.finish()?;
                match slot {
                    -1 if !stack.is_empty() => self.throw_stack(stack, wire, ctx)?,
                    5..=8 => self.inventory[(44 - slot) as usize] = stack,
                    9..=35 => self.inventory[slot as usize] = stack,
                    36..=44 => self.inventory[(slot - 36) as usize] = stack,
                    45 => self.inventory[OFFHAND] = stack,
                    _ => {}
                }
                if self.gliding && !self.has_elytra() {
                    self.set_gliding(false, wire, ctx.ids)?;
                }
            }
            "player_action" => {
                let action = body.varint()?;
                let block = BlockPos::of(i64::from_be_bytes(body.fixed()?));
                body.fixed::<1>()?; // facing
                let sequence = body.varint()?;
                body.finish()?;
                match action {
                    0..=2 => {
                        if action == 0 && self.in_reach(block) && FlatWorld::editable(block) {
                            let old = ctx.world.change(block, AIR, ctx.audience);
                            if ctx.commands.world.lock().unwrap().drops_enabled() {
                                if let Some(item) = Items::vanilla().block_drop(old) {
                                    let pos = Vec3::new(
                                        f64::from(block.x) + 0.5,
                                        f64::from(block.y) + 0.35,
                                        f64::from(block.z) + 0.5,
                                    );
                                    self.spawn_drop(item, pos, Vec3::ZERO, wire, ctx)?;
                                }
                            }
                        } else {
                            wire.send(
                                ctx.ids,
                                &ClientboundBlockUpdatePacket::new(block, ctx.world.block(block)),
                            )?;
                        }
                        wire.send(ctx.ids, &ClientboundBlockChangedAckPacket::new(sequence))?;
                    }
                    3 | 4 => {
                        let amount = if action == 3 {
                            self.inventory[self.selection].get_count()
                        } else {
                            1
                        };
                        if !self.inventory[self.selection].is_empty() {
                            let stack = self.inventory[self.selection].split(amount);
                            self.sync_slot(self.selection, wire, ctx.ids)?;
                            self.throw_stack(stack, wire, ctx)?;
                        }
                    }
                    6 => {
                        self.inventory.swap(self.selection, OFFHAND);
                        self.sync_slot(self.selection, wire, ctx.ids)?;
                        self.sync_slot(OFFHAND, wire, ctx.ids)?;
                    }
                    _ => {}
                }
            }
            "player_command" => {
                let entity = body.varint()?;
                let action = body.varint()?;
                body.varint()?;
                body.finish()?;
                if entity == self.entity_id
                    && action == 6
                    && !self.on_ground
                    && !self.creative_flying
                    && self.has_elytra()
                {
                    self.set_gliding(true, wire, ctx.ids)?;
                }
            }
            "player_abilities" => {
                self.creative_flying = body.fixed::<1>()?[0] & 2 != 0;
                body.finish()?;
                if self.creative_flying && self.gliding {
                    self.set_gliding(false, wire, ctx.ids)?;
                }
            }
            "use_item" => {
                let hand = body.count(1)?;
                let sequence = body.varint()?;
                self.yaw = f32::from_be_bytes(body.fixed()?);
                self.pitch = f32::from_be_bytes(body.fixed()?);
                body.finish()?;
                if !self.yaw.is_finite() || !self.pitch.is_finite() {
                    return Err(invalid("invalid player rotation"));
                }
                self.use_item(hand, wire, ctx)?;
                wire.send(ctx.ids, &ClientboundBlockChangedAckPacket::new(sequence))?;
            }
            "use_item_on" => {
                let hand = body.count(1)?;
                let pos = BlockPos::of(i64::from_be_bytes(body.fixed()?));
                let direction = Direction::from_3d_data_value(body.count(5)? as i32);
                body.fixed::<14>()?; // hit position, inside and world border
                let sequence = body.varint()?;
                body.finish()?;
                if self.in_reach(pos) {
                    let placed = pos.relative(direction);
                    let slot = self.hand_slot(hand);
                    if let Some(state) = Items::vanilla().place_state(&self.inventory[slot]) {
                        let feet = BlockPos::new(
                            self.position.x.floor() as i32,
                            self.position.y.floor() as i32,
                            self.position.z.floor() as i32,
                        );
                        if placed != feet
                            && placed != feet.offset(0, 1, 0)
                            && ctx.world.block(pos) != AIR
                        {
                            ctx.world.change(placed, state, ctx.audience);
                        } else {
                            wire.send(
                                ctx.ids,
                                &ClientboundBlockUpdatePacket::new(placed, ctx.world.block(placed)),
                            )?;
                        }
                    } else {
                        self.use_item(hand, wire, ctx)?;
                    }
                }
                wire.send(ctx.ids, &ClientboundBlockChangedAckPacket::new(sequence))?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn kit(&mut self, wire: &mut Wire, ctx: &PlayContext<'_>) -> io::Result<()> {
        if self.inventory[CHEST].is_empty() {
            self.inventory[CHEST] = stack(ELYTRA, 1);
            self.sync_slot(CHEST, wire, ctx.ids)?;
        } else if !self.has_elytra() {
            let extra = self.insert(stack(ELYTRA, 1), wire, ctx.ids)?;
            if !extra.is_empty() {
                self.throw_stack(extra, wire, ctx)?;
            }
        }
        let extra = self.insert(stack(FIREWORK, 64), wire, ctx.ids)?;
        if !extra.is_empty() {
            self.throw_stack(extra, wire, ctx)?;
        }
        Ok(())
    }
    fn has_elytra(&self) -> bool {
        let chest = &self.inventory[CHEST];
        item_id(chest) == Some(ELYTRA) && chest.get_damage_value() < 431
    }

    fn set_gliding(&mut self, gliding: bool, wire: &mut Wire, ids: &PacketIds) -> io::Result<()> {
        if self.gliding == gliding {
            return Ok(());
        }
        self.gliding = gliding;
        wire.send(
            ids,
            &ClientboundSetEntityDataPacket::new(
                self.entity_id,
                vec![
                    data(0, 0, if gliding { -128_i8 } else { 0_i8 }),
                    data(
                        6,
                        20,
                        if gliding {
                            Pose::FallFlying
                        } else {
                            Pose::Standing
                        },
                    ),
                ],
            ),
        )
    }

    fn hand_slot(&self, hand: usize) -> usize {
        if hand == 0 {
            self.selection
        } else {
            OFFHAND
        }
    }

    fn use_item(&mut self, hand: usize, wire: &mut Wire, ctx: &PlayContext<'_>) -> io::Result<()> {
        let slot = self.hand_slot(hand);
        match item_id(&self.inventory[slot]) {
            Some(ELYTRA) => {
                self.inventory.swap(slot, CHEST);
                self.sync_slot(slot, wire, ctx.ids)?;
                self.sync_slot(CHEST, wire, ctx.ids)?;
            }
            Some(FIREWORK) if self.gliding && self.has_elytra() && self.rockets.len() < 16 => {
                let item = self.inventory[slot].copy_with_count(1);
                let duration = item
                    .get_component::<Fireworks>(&Identifier::minecraft("fireworks"))
                    .map_or(1, |fireworks| fireworks.flight_duration);
                let entity = spawn(FIREWORK_ENTITY, self.position, Vec3::ZERO, wire, ctx)?;
                wire.send(
                    ctx.ids,
                    &ClientboundSetEntityDataPacket::new(
                        entity,
                        vec![data(8, 7, item), data(9, 19, Some(self.entity_id as u32))],
                    ),
                )?;
                let lifetime =
                    Duration::from_millis(u64::try_from(10 * (1 + duration)).unwrap_or(20) * 50);
                self.rockets.push((entity, Instant::now() + lifetime));
            }
            _ => {}
        }
        Ok(())
    }

    fn in_reach(&self, pos: BlockPos) -> bool {
        let eye = Vec3::new(self.position.x, self.position.y + 1.62, self.position.z);
        let target = Vec3::new(
            f64::from(pos.x) + 0.5,
            f64::from(pos.y) + 0.5,
            f64::from(pos.z) + 0.5,
        );
        eye.distance_to_sqr(&target) <= 36.0
    }

    fn sync_slot(&self, slot: usize, wire: &mut Wire, ids: &PacketIds) -> io::Result<()> {
        wire.send(
            ids,
            &ClientboundSetPlayerInventoryPacket::new(slot as i32, self.inventory[slot].clone()),
        )
    }

    fn insert(
        &mut self,
        mut stack: ItemStack,
        wire: &mut Wire,
        ids: &PacketIds,
    ) -> io::Result<ItemStack> {
        for slot in 0..36 {
            if stack.is_empty() {
                break;
            }
            let current = &mut self.inventory[slot];
            if ItemStack::is_same_item_same_components(current, &stack) && !current.is_empty() {
                let amount = (current.get_max_stack_size() - current.count)
                    .max(0)
                    .min(stack.count);
                if amount != 0 {
                    current.grow(amount);
                    stack.shrink(amount);
                    self.sync_slot(slot, wire, ids)?;
                }
            }
        }
        if !stack.is_empty() {
            if let Some(slot) = self.inventory[..36].iter().position(ItemStack::is_empty) {
                self.inventory[slot] = stack;
                self.sync_slot(slot, wire, ids)?;
                return Ok(ItemStack::empty());
            }
        }
        Ok(stack)
    }

    fn throw_stack(
        &mut self,
        item: ItemStack,
        wire: &mut Wire,
        ctx: &PlayContext<'_>,
    ) -> io::Result<()> {
        let direction = Vec3::direction_from_rotation(self.pitch, self.yaw);
        let pos = self.position.add(0.0, 1.32, 0.0);
        self.spawn_drop(item, pos, direction.scale(0.3), wire, ctx)
    }

    fn spawn_drop(
        &mut self,
        item: ItemStack,
        pos: Vec3,
        velocity: Vec3,
        wire: &mut Wire,
        ctx: &PlayContext<'_>,
    ) -> io::Result<()> {
        if self.drops.len() >= 128 {
            let previous = self.drops.remove(0);
            wire.send(
                ctx.ids,
                &ClientboundRemoveEntitiesPacket::new(vec![previous.id]),
            )?;
        }
        let entity = spawn(ITEM_ENTITY, pos, velocity, wire, ctx)?;
        wire.send(
            ctx.ids,
            &ClientboundSetEntityDataPacket::new(entity, vec![data(8, 7, item.clone())]),
        )?;
        self.drops.push(DropEntity {
            id: entity,
            position: pos,
            velocity,
            stack: item,
            created: Instant::now(),
        });
        Ok(())
    }

    pub fn tick(&mut self, wire: &mut Wire, ctx: &PlayContext<'_>) -> io::Result<()> {
        if self.last_tick.elapsed() < TICK {
            return Ok(());
        }
        self.last_tick = Instant::now();
        let now = self.last_tick;

        let mut removed = Vec::new();
        self.rockets.retain(|&(id, end)| {
            if end <= now {
                removed.push(id);
                false
            } else {
                true
            }
        });
        for &id in &removed {
            wire.send(ctx.ids, &ClientboundEntityEventPacket::new(id, 17))?;
        }

        for mut drop in std::mem::take(&mut self.drops) {
            let age = now.saturating_duration_since(drop.created);
            if age >= Duration::from_secs(300) || drop.position.y < -72.0 {
                removed.push(drop.id);
                continue;
            }

            let next = drop
                .position
                .add(drop.velocity.x, drop.velocity.y, drop.velocity.z);
            let block = BlockPos::new(
                next.x.floor() as i32,
                next.y.floor() as i32,
                next.z.floor() as i32,
            );
            let on_ground = drop.velocity.y <= 0.0 && ctx.world.block(block) != AIR;
            if on_ground {
                drop.position = Vec3::new(next.x, f64::from(block.y + 1), next.z);
                drop.velocity = Vec3::ZERO;
            } else {
                drop.position = next;
                drop.velocity = drop.velocity.add(0.0, -0.04, 0.0).scale(0.98);
            }

            if age >= Duration::from_secs(2)
                && drop
                    .position
                    .distance_to_sqr(&self.position.add(0.0, 0.5, 0.0))
                    < 1.44
            {
                let count = drop.stack.get_count();
                let remaining = self.insert(drop.stack, wire, ctx.ids)?;
                if remaining.is_empty() {
                    wire.send(
                        ctx.ids,
                        &ClientboundTakeItemEntityPacket::new(drop.id, self.entity_id, count),
                    )?;
                    removed.push(drop.id);
                    continue;
                }
                if remaining.get_count() != count {
                    wire.send(
                        ctx.ids,
                        &ClientboundSetEntityDataPacket::new(
                            drop.id,
                            vec![data(8, 7, remaining.clone())],
                        ),
                    )?;
                }
                drop.stack = remaining;
            }
            wire.send(
                ctx.ids,
                &ClientboundEntityPositionSyncPacket::new(
                    drop.id,
                    PositionMoveRotation::new(drop.position, drop.velocity, 0.0, 0.0),
                    on_ground,
                ),
            )?;
            self.drops.push(drop);
        }
        if !removed.is_empty() {
            wire.send(ctx.ids, &ClientboundRemoveEntitiesPacket::new(removed))?;
        }
        Ok(())
    }
}

fn data(id: i32, serializer: i32, value: impl Send + Sync + 'static) -> DataValue {
    DataValue {
        id,
        serializer,
        value: Box::new(value),
    }
}

fn spawn(
    kind: i32,
    pos: Vec3,
    velocity: Vec3,
    wire: &mut Wire,
    ctx: &PlayContext<'_>,
) -> io::Result<i32> {
    let id = ctx.entity_ids.fetch_add(1, Ordering::Relaxed);
    wire.send(
        ctx.ids,
        &ClientboundAddEntityPacket::new(
            id,
            Uuid::random(),
            Holder::Reference(kind),
            pos.x,
            pos.y,
            pos.z,
            velocity,
            0.0,
            0.0,
            0.0,
            0,
        ),
    )?;
    Ok(id)
}
