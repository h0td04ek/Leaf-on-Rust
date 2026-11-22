use std::io;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::{Duration, Instant};

use leaf_core::core::{block_pos::BlockPos, direction::Direction, holder::Holder};
use leaf_core::entity::{
    equipment_slot::EquipmentSlot, player::abilities::Abilities,
    position_move_rotation::PositionMoveRotation,
};
use leaf_core::network::protocol::game::{
    clientbound_add_entity_packet::ClientboundAddEntityPacket,
    clientbound_block_changed_ack_packet::ClientboundBlockChangedAckPacket,
    clientbound_block_update_packet::ClientboundBlockUpdatePacket,
    clientbound_entity_event_packet::ClientboundEntityEventPacket,
    clientbound_entity_position_sync_packet::ClientboundEntityPositionSyncPacket,
    clientbound_game_event_packet::{ClientboundGameEventPacket, CHANGE_GAME_MODE},
    clientbound_player_abilities_packet::ClientboundPlayerAbilitiesPacket,
    clientbound_remove_entities_packet::ClientboundRemoveEntitiesPacket,
    clientbound_set_entity_data_packet::ClientboundSetEntityDataPacket,
    clientbound_set_held_slot_packet::ClientboundSetHeldSlotPacket,
    clientbound_set_player_inventory_packet::ClientboundSetPlayerInventoryPacket,
    clientbound_take_item_entity_packet::ClientboundTakeItemEntityPacket,
};
use leaf_core::network::syncher::synched_entity_data::DataValue;
use leaf_core::resources::Identifier;
use leaf_core::util::uuid::Uuid;
use leaf_core::world::item::{component::fireworks::Fireworks, item_stack::ItemStack};
use leaf_core::world::level::game_type::GameType;
use leaf_core::world::phys::vec3::Vec3;

use crate::audience::Audience;
use crate::commands::Commands;
use crate::items::{item_id, stack, Items, ELYTRA, FIREWORK, FIREWORK_ENTITY, ITEM_ENTITY};
use crate::peers::{gliding_packet, View};
use crate::player_data::PlayerData;
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
    pub player: PlayerData,
    entity_id: i32,
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
    pub fn new(entity_id: i32, player: PlayerData) -> Self {
        let creative_flying = player.mode == GameType::Spectator;
        Self {
            player,
            entity_id,
            on_ground: true,
            creative_flying,
            gliding: false,
            drops: vec![],
            rockets: vec![],
            last_tick: Instant::now(),
        }
    }

    pub fn view(&self) -> View {
        let equipment = [
            (EquipmentSlot::Mainhand, self.player.selection),
            (EquipmentSlot::Offhand, OFFHAND),
            (EquipmentSlot::Head, 39),
            (EquipmentSlot::Chest, CHEST),
            (EquipmentSlot::Legs, 37),
            (EquipmentSlot::Feet, 36),
        ]
        .into_iter()
        .map(|(slot, index)| (slot, self.player.inventory[index].clone()))
        .collect();
        View {
            motion: PositionMoveRotation::new(
                self.player.position,
                Vec3::ZERO,
                self.player.yaw,
                self.player.pitch,
            ),
            on_ground: self.on_ground,
            gliding: self.gliding,
            equipment,
            mode: self.player.mode,
        }
    }

    pub fn abilities(&self) -> ClientboundPlayerAbilitiesPacket {
        let mut abilities = Abilities::default();
        self.player.mode.update_player_abilities(&mut abilities);
        if self.player.mode.is_creative() {
            abilities.flying = self.creative_flying;
        }
        ClientboundPlayerAbilitiesPacket::from_abilities(&abilities)
    }

    pub fn set_mode(&mut self, mode: GameType, wire: &mut Wire, ids: &PacketIds) -> io::Result<()> {
        self.player.mode = mode;
        if !mode.is_creative() {
            self.creative_flying = mode == GameType::Spectator;
        }
        if self.creative_flying {
            self.set_gliding(false, wire, ids)?;
        }
        wire.send(
            ids,
            &ClientboundGameEventPacket::new(CHANGE_GAME_MODE, mode.get_id() as f32),
        )?;
        wire.send(ids, &self.abilities())
    }

    pub fn reset_after_death(&mut self) {
        self.player.position = Vec3::new(0.5, f64::from(SPAWN_Y), 0.5);
        self.player.yaw = 0.0;
        self.player.pitch = 0.0;
        self.player.health = 20.0;
        self.on_ground = true;
        self.creative_flying = self.player.mode == GameType::Spectator;
        self.gliding = false;
    }

    pub fn movement(
        &mut self,
        kind: &str,
        mut body: Cursor,
        wire: &mut Wire,
        ids: &PacketIds,
    ) -> io::Result<()> {
        let mut position = self.player.position;
        let mut yaw = self.player.yaw;
        let mut pitch = self.player.pitch;
        if matches!(kind, "move_player_pos" | "move_player_pos_rot") {
            position = Vec3::new(
                f64::from_be_bytes(body.fixed()?),
                f64::from_be_bytes(body.fixed()?),
                f64::from_be_bytes(body.fixed()?),
            );
            if !PlayerData::valid_position(position) {
                return Err(invalid("invalid player position"));
            }
        }
        if matches!(kind, "move_player_rot" | "move_player_pos_rot") {
            yaw = f32::from_be_bytes(body.fixed()?);
            pitch = f32::from_be_bytes(body.fixed()?);
            if !yaw.is_finite() || !pitch.is_finite() {
                return Err(invalid("invalid player rotation"));
            }
        }
        let on_ground = body.fixed::<1>()?[0] & 1 != 0;
        body.finish()?;
        self.player.position = position;
        self.player.yaw = yaw;
        self.player.pitch = pitch;
        self.on_ground = on_ground;
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
        if self.player.mode == GameType::Spectator && name != "player_abilities" {
            return Ok(());
        }
        match name {
            "set_carried_item" => {
                let slot = i16::from_be_bytes(body.fixed()?);
                body.finish()?;
                if (0..9).contains(&slot) {
                    self.player.selection = slot as usize;
                }
            }
            "set_creative_mode_slot" if self.player.mode.is_creative() => {
                let slot = i16::from_be_bytes(body.fixed()?);
                let stack = Items::vanilla().read_creative_stack(&mut body)?;
                body.finish()?;
                match slot {
                    -1 if !stack.is_empty() => self.throw_stack(stack, wire, ctx)?,
                    5..=8 => self.player.inventory[(44 - slot) as usize] = stack,
                    9..=35 => self.player.inventory[slot as usize] = stack,
                    36..=44 => self.player.inventory[(slot - 36) as usize] = stack,
                    45 => self.player.inventory[OFFHAND] = stack,
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
                        let break_action = if self.player.mode.is_creative() { 0 } else { 2 };
                        if action == break_action
                            && !self.player.mode.is_block_placing_restricted()
                            && self.in_reach(block)
                        {
                            let old = ctx.world.change(block, AIR, ctx.audience);
                            if ctx.commands.world.lock().unwrap().drops_enabled() {
                                if let Some(item) =
                                    old.and_then(|state| Items::vanilla().block_drop(state))
                                {
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
                            self.player.inventory[self.player.selection].get_count()
                        } else {
                            1
                        };
                        if !self.player.inventory[self.player.selection].is_empty() {
                            let stack = self.player.inventory[self.player.selection].split(amount);
                            self.sync_slot(self.player.selection, wire, ctx.ids)?;
                            self.throw_stack(stack, wire, ctx)?;
                        }
                    }
                    6 => {
                        self.player.inventory.swap(self.player.selection, OFFHAND);
                        self.sync_slot(self.player.selection, wire, ctx.ids)?;
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
                let flying = body.fixed::<1>()?[0] & 2 != 0;
                self.creative_flying = self.player.mode == GameType::Spectator
                    || self.player.mode.is_creative() && flying;
                body.finish()?;
                if self.creative_flying && self.gliding {
                    self.set_gliding(false, wire, ctx.ids)?;
                }
            }
            "use_item" => {
                let hand = body.count(1)?;
                let sequence = body.varint()?;
                let yaw = f32::from_be_bytes(body.fixed()?);
                let pitch = f32::from_be_bytes(body.fixed()?);
                body.finish()?;
                if !yaw.is_finite() || !pitch.is_finite() {
                    return Err(invalid("invalid player rotation"));
                }
                self.player.yaw = yaw;
                self.player.pitch = pitch;
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
                    if let Some(state) = Items::vanilla().place_state(&self.player.inventory[slot])
                    {
                        let feet = BlockPos::new(
                            self.player.position.x.floor() as i32,
                            self.player.position.y.floor() as i32,
                            self.player.position.z.floor() as i32,
                        );
                        if placed != feet
                            && placed != feet.offset(0, 1, 0)
                            && !self.player.mode.is_block_placing_restricted()
                            && ctx.world.block(pos) != AIR
                        {
                            if ctx.world.change(placed, state, ctx.audience).is_some()
                                && !self.player.mode.is_creative()
                            {
                                self.player.inventory[slot].shrink(1);
                                self.sync_slot(slot, wire, ctx.ids)?;
                            }
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

    pub fn sync_inventory(&self, wire: &mut Wire, ids: &PacketIds) -> io::Result<()> {
        for (slot, item) in self.player.inventory.iter().enumerate() {
            if !item.is_empty() {
                self.sync_slot(slot, wire, ids)?;
            }
        }
        wire.send(
            ids,
            &ClientboundSetHeldSlotPacket::new(self.player.selection as i32),
        )
    }

    pub fn kit(&mut self, wire: &mut Wire, ctx: &PlayContext<'_>) -> io::Result<()> {
        if self.player.inventory[CHEST].is_empty() {
            self.player.inventory[CHEST] = stack(ELYTRA, 1);
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
        let chest = &self.player.inventory[CHEST];
        item_id(chest) == Some(ELYTRA) && chest.get_damage_value() < 431
    }

    fn set_gliding(&mut self, gliding: bool, wire: &mut Wire, ids: &PacketIds) -> io::Result<()> {
        if self.gliding == gliding {
            return Ok(());
        }
        self.gliding = gliding;
        wire.send(ids, &gliding_packet(self.entity_id, gliding))
    }

    fn hand_slot(&self, hand: usize) -> usize {
        if hand == 0 {
            self.player.selection
        } else {
            OFFHAND
        }
    }

    fn use_item(&mut self, hand: usize, wire: &mut Wire, ctx: &PlayContext<'_>) -> io::Result<()> {
        let slot = self.hand_slot(hand);
        match item_id(&self.player.inventory[slot]) {
            Some(ELYTRA) => {
                self.player.inventory.swap(slot, CHEST);
                self.sync_slot(slot, wire, ctx.ids)?;
                self.sync_slot(CHEST, wire, ctx.ids)?;
            }
            Some(FIREWORK) if self.gliding && self.has_elytra() && self.rockets.len() < 16 => {
                let item = self.player.inventory[slot].copy_with_count(1);
                let duration = item
                    .get_component::<Fireworks>(&Identifier::minecraft("fireworks"))
                    .map_or(1, |fireworks| fireworks.flight_duration);
                let entity = spawn(FIREWORK_ENTITY, self.player.position, Vec3::ZERO, wire, ctx)?;
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
                if !self.player.mode.is_creative() {
                    self.player.inventory[slot].shrink(1);
                    self.sync_slot(slot, wire, ctx.ids)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn in_reach(&self, pos: BlockPos) -> bool {
        let eye = Vec3::new(
            self.player.position.x,
            self.player.position.y + 1.62,
            self.player.position.z,
        );
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
            &ClientboundSetPlayerInventoryPacket::new(
                slot as i32,
                self.player.inventory[slot].clone(),
            ),
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
            let current = &mut self.player.inventory[slot];
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
            if let Some(slot) = self.player.inventory[..36]
                .iter()
                .position(ItemStack::is_empty)
            {
                self.player.inventory[slot] = stack;
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
        let direction = Vec3::direction_from_rotation(self.player.pitch, self.player.yaw);
        let pos = self.player.position.add(0.0, 1.32, 0.0);
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
                    .distance_to_sqr(&self.player.position.add(0.0, 0.5, 0.0))
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
