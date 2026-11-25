use std::collections::HashSet;
use std::io;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::{Duration, Instant};

use leaf_core::core::{block_pos::BlockPos, direction::Direction, holder::Holder};
use leaf_core::entity::{
    equipment_slot::EquipmentSlot, player::abilities::Abilities,
    position_move_rotation::PositionMoveRotation,
};
use leaf_core::network::chat::component::Component;
use leaf_core::network::protocol::game::{
    clientbound_add_entity_packet::ClientboundAddEntityPacket,
    clientbound_block_changed_ack_packet::ClientboundBlockChangedAckPacket,
    clientbound_container_close_packet::ClientboundContainerClosePacket,
    clientbound_entity_event_packet::ClientboundEntityEventPacket,
    clientbound_game_event_packet::{ClientboundGameEventPacket, CHANGE_GAME_MODE},
    clientbound_open_screen_packet::ClientboundOpenScreenPacket,
    clientbound_place_ghost_recipe_packet::ClientboundPlaceGhostRecipePacket,
    clientbound_player_abilities_packet::ClientboundPlayerAbilitiesPacket,
    clientbound_remove_entities_packet::ClientboundRemoveEntitiesPacket,
    clientbound_set_entity_data_packet::ClientboundSetEntityDataPacket,
    clientbound_set_held_slot_packet::ClientboundSetHeldSlotPacket,
    clientbound_set_player_inventory_packet::ClientboundSetPlayerInventoryPacket,
    clientbound_sound_packet::ClientboundSoundPacket,
};
use leaf_core::network::syncher::synched_entity_data::DataValue;
use leaf_core::random::{LegacyRandomSource, RandomSource};
use leaf_core::resources::Identifier;
use leaf_core::server::network::filterable::Filterable;
use leaf_core::sounds::{sound_event::SoundEvent, sound_source::SoundSource};
use leaf_core::util::uuid::Uuid;
use leaf_core::world::inventory::ClickType;
use leaf_core::world::item::component::{
    writable_book_content::WritableBookContent, written_book_content::WrittenBookContent,
};
use leaf_core::world::item::{component::fireworks::Fireworks, item_stack::ItemStack};
use leaf_core::world::level::game_type::GameType;
use leaf_core::world::level::saveddata::maps::MapId;
use leaf_core::world::phys::vec3::Vec3;

use crate::audience::{Audience, Subscription};
use crate::blocks::{placement_state, Blocks};
use crate::commands::Commands;
use crate::crafting::Crafting;
use crate::drops::{DropTracker, Drops, PickupDelay};
use crate::items::{item_id, named_id, stack, Items, ELYTRA, FIREWORK, FIREWORK_ENTITY};
use crate::maps::{map_id, set_map_id, Maps};
use crate::mining::{enchantment_level, Miner, Mining};
use crate::mob_types;
use crate::mobs::Mobs;
use crate::peers::{clear_sight_with, flags, gliding_packet, View};
use crate::player_data::PlayerData;
use crate::player_environment::Environment;
use crate::recipes::Recipes;
use crate::simulation::SimulationContext;
use crate::wire::{invalid, Cursor, PacketIds, Wire};
use crate::world::{bucket_hit, chunk, BucketHit, Fluid, NativeWorld, AIR};

const CHEST: usize = 38;
const OFFHAND: usize = 40;
const TICK: Duration = Duration::from_millis(50);

pub struct PlayContext<'a> {
    pub world: &'a NativeWorld,
    pub drops: &'a Drops,
    pub maps: &'a Maps,
    pub mobs: &'a Mobs,
    pub name: &'a str,
    pub player: Uuid,
    pub ids: &'a PacketIds,
    pub audience: &'a Audience,
    pub commands: &'a Commands,
    pub entity_ids: &'a AtomicI32,
}

impl PlayContext<'_> {
    pub fn simulation(&self) -> SimulationContext<'_> {
        SimulationContext {
            world: self.world,
            commands: self.commands,
            audience: self.audience,
            drops: self.drops,
            entity_ids: self.entity_ids,
        }
    }
}

#[derive(Clone, Copy)]
struct Digging {
    pos: BlockPos,
    state: i32,
    start: i64,
    released: bool,
}

impl Digging {
    fn progress(self, step: f32, now: i64) -> f32 {
        step * (now.saturating_sub(self.start) + 1) as f32
    }
}

pub struct Gameplay {
    pub player: PlayerData,
    pub crafting: Crafting,
    entity_id: i32,
    on_ground: bool,
    creative_flying: bool,
    gliding: bool,
    sent_environment: Environment,
    drops: DropTracker,
    mining: Option<Digging>,
    loot_random: LegacyRandomSource,
    sneaking: bool,
    rockets: Vec<(i32, i64)>,
    last_game_tick: i64,
    last_tick: Instant,
    shown_maps: HashSet<MapId>,
}

impl Gameplay {
    pub fn new(entity_id: i32, player: PlayerData, game_tick: i64) -> Self {
        let creative_flying = player.mode == GameType::Spectator;
        Self {
            player,
            crafting: Crafting::default(),
            entity_id,
            on_ground: true,
            creative_flying,
            gliding: false,
            sent_environment: Environment::default(),
            drops: DropTracker::default(),
            mining: None,
            loot_random: LegacyRandomSource::new(i64::from(entity_id) << 32 ^ game_tick),
            sneaking: false,
            rockets: vec![],
            last_game_tick: game_tick,
            last_tick: Instant::now(),
            shown_maps: HashSet::new(),
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
            sneaking: self.sneaking,
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
        if self.player.mode == GameType::Spectator {
            abilities.flying_speed *= 2.0;
        }
        ClientboundPlayerAbilitiesPacket::from_abilities(&abilities)
    }

    pub fn set_mode(&mut self, mode: GameType, wire: &mut Wire, ids: &PacketIds) -> io::Result<()> {
        self.player.mode = mode;
        self.mining = None;
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

    pub fn reset_after_death(&mut self, spawn: Vec3) {
        self.player.position = spawn;
        self.player.yaw = 0.0;
        self.player.pitch = 0.0;
        self.player.health = 20.0;
        self.player.environment = Environment::default();
        self.on_ground = true;
        self.creative_flying = self.player.mode == GameType::Spectator;
        self.gliding = false;
        self.drops.reset();
        self.shown_maps.clear();
        self.mining = None;
        self.sneaking = false;
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
        self.on_ground = on_ground && self.player.mode != GameType::Spectator;
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
            "container_click" => {
                let menu = body.varint()?;
                let _state = body.varint()?;
                let slot = i16::from_be_bytes(body.fixed()?);
                let [button] = body.fixed()?;
                let action = ClickType::by_id(body.count(6)? as i32);
                // The rest of the packet describes the client's prediction. The server owns every slot.
                if menu == self.crafting.id {
                    for item in self.crafting.click(
                        &mut self.player.inventory,
                        slot,
                        button,
                        action,
                        self.player.mode.is_creative(),
                        ctx.maps,
                    ) {
                        self.throw_stack(item, ctx);
                    }
                    self.sync_menu(wire, ctx)?;
                }
            }
            "container_close" => {
                let menu = body.varint()?;
                body.finish()?;
                if menu == self.crafting.id {
                    self.close_crafting(wire, ctx, false)?;
                }
            }
            "place_recipe" => {
                let menu = body.varint()?;
                let recipe = body.varint()?;
                let [maximum] = body.fixed()?;
                body.finish()?;
                if menu == self.crafting.id {
                    if let Some(recipe) = Recipes::vanilla().by_id(recipe as usize) {
                        if self
                            .crafting
                            .fill(&mut self.player.inventory, recipe, maximum != 0)
                        {
                            self.sync_menu(wire, ctx)?;
                        } else {
                            wire.send(
                                ctx.ids,
                                &ClientboundPlaceGhostRecipePacket::new(menu, recipe.display()),
                            )?;
                        }
                    }
                }
            }
            "edit_book" => {
                let slot = body.count(40)?;
                let pages = (0..body.count(100)?)
                    .map(|_| body.string(1024))
                    .collect::<io::Result<Vec<_>>>()?;
                let [has_title] = body.fixed()?;
                let title = (has_title != 0).then(|| body.string(32)).transpose()?;
                body.finish()?;
                if (slot < 9 || slot == OFFHAND)
                    && item_id(&self.player.inventory[slot]) == Some(named_id("writable_book"))
                {
                    if let Some(title) = title {
                        let pages = pages
                            .into_iter()
                            .map(|page| Filterable::pass_through(Component::literal(page)))
                            .collect();
                        let mut written = stack(named_id("written_book"), 1);
                        written.set_component(
                            &Identifier::minecraft("written_book_content"),
                            Some(WrittenBookContent::new(
                                Some(Filterable::pass_through(title.trim().to_owned())),
                                ctx.name.to_owned(),
                                0,
                                pages,
                                true,
                            )),
                        );
                        self.player.inventory[slot] = written;
                    } else {
                        let pages = pages.into_iter().map(Filterable::pass_through).collect();
                        self.player.inventory[slot].set_component(
                            &Identifier::minecraft("writable_book_content"),
                            Some(WritableBookContent::new(pages)),
                        );
                    }
                    self.sync_slot(slot, wire, ctx.ids)?;
                }
            }
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
                    -1 if !stack.is_empty() => self.throw_stack(stack, ctx),
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
                        self.mine(action, block, wire, ctx)?;
                        ctx.audience
                            .send(ctx.player, ClientboundBlockChangedAckPacket::new(sequence));
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
                            self.throw_stack(stack, ctx);
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
            "player_input" => {
                self.sneaking = body.fixed::<1>()?[0] & 32 != 0;
                body.finish()?;
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
                ctx.audience
                    .send(ctx.player, ClientboundBlockChangedAckPacket::new(sequence));
            }
            "use_item_on" => {
                let hand = body.count(1)?;
                let pos = BlockPos::of(i64::from_be_bytes(body.fixed()?));
                let direction = Direction::from_3d_data_value(body.count(5)? as i32);
                let _hit_x = f32::from_be_bytes(body.fixed()?);
                let hit_y = f32::from_be_bytes(body.fixed()?);
                let _hit_z = f32::from_be_bytes(body.fixed()?);
                body.fixed::<2>()?; // inside and world border
                let sequence = body.varint()?;
                body.finish()?;
                self.use_block(pos, direction, hit_y, hand, wire, ctx)?;
                ctx.audience
                    .send(ctx.player, ClientboundBlockChangedAckPacket::new(sequence));
            }
            _ => {}
        }
        Ok(())
    }

    fn mine(
        &mut self,
        action: i32,
        pos: BlockPos,
        wire: &mut Wire,
        ctx: &PlayContext<'_>,
    ) -> io::Result<()> {
        if action == 1 || self.player.mode.is_block_placing_restricted() || !self.in_reach(pos) {
            self.mining = None;
            ctx.world.refresh_block(pos, ctx.audience, ctx.player);
            return Ok(());
        }
        let state = ctx.world.block(pos);
        let block = Blocks::vanilla().get(state);
        let held = &self.player.inventory[self.player.selection];
        let miner = Miner::new(held, block);
        let creative = self.player.mode.is_creative();
        if state == AIR
            || (creative && !miner.can_break_in_creative)
            || (!creative && Mining::vanilla().hardness(block) < 0.0)
        {
            self.mining = None;
            ctx.world.refresh_block(pos, ctx.audience, ctx.player);
            return Ok(());
        }
        let now = ctx.commands.game_time();
        if action == 0 {
            self.mining = None;
            if creative || self.destroy_progress(&miner, block, ctx) >= 1.0 {
                return self.finish_mining(pos, state, &miner, wire, ctx);
            }
            self.mining = Some(Digging {
                pos,
                state,
                start: now,
                released: false,
            });
        } else if let Some(mut past) = self
            .mining
            .take()
            .filter(|past| past.pos == pos && past.state == state)
        {
            if past.progress(self.destroy_progress(&miner, block, ctx), now) >= 0.7 {
                return self.finish_mining(pos, state, &miner, wire, ctx);
            }
            // The client may finish slightly ahead of the server. Keep the
            // confirmed attempt running to full progress without a second packet.
            past.released = true;
            self.mining = Some(past);
        } else {
            ctx.world.refresh_block(pos, ctx.audience, ctx.player);
        }
        Ok(())
    }

    fn destroy_progress(
        &self,
        miner: &Miner,
        block: &crate::blocks::Block,
        ctx: &PlayContext<'_>,
    ) -> f32 {
        let eye = self
            .player
            .position
            .add(0.0, if self.sneaking { 1.27 } else { 1.62 }, 0.0);
        let underwater = ctx.world.snapshot().fluid_at(eye) == Some(Fluid::Water)
            && enchantment_level(&self.player.inventory[39], "aqua_affinity") == 0;
        miner.progress(block, self.on_ground, underwater)
    }

    fn tick_mining(&mut self, now: i64, wire: &mut Wire, ctx: &PlayContext<'_>) -> io::Result<()> {
        let Some(past) = self.mining.filter(|past| past.released) else {
            return Ok(());
        };
        if self.player.health <= 0.0
            || !self.in_reach(past.pos)
            || ctx.world.block(past.pos) != past.state
        {
            self.mining = None;
            ctx.world.refresh_block(past.pos, ctx.audience, ctx.player);
            return Ok(());
        }
        let block = Blocks::vanilla().get(past.state);
        let miner = Miner::new(&self.player.inventory[self.player.selection], block);
        if past.progress(self.destroy_progress(&miner, block, ctx), now) >= 1.0 {
            self.mining = None;
            self.finish_mining(past.pos, past.state, &miner, wire, ctx)?;
        }
        Ok(())
    }

    fn finish_mining(
        &mut self,
        pos: BlockPos,
        state: i32,
        miner: &Miner,
        wire: &mut Wire,
        ctx: &PlayContext<'_>,
    ) -> io::Result<()> {
        if !ctx.world.break_block(pos, state, ctx.audience) {
            ctx.world.refresh_block(pos, ctx.audience, ctx.player);
            return Ok(());
        }
        if self.player.mode.is_creative() {
            return Ok(());
        }
        let block = Blocks::vanilla().get(state);
        if ctx.commands.world.lock().unwrap().drops_enabled() {
            for (item, mut count) in miner.drops(block, state, &mut self.loot_random) {
                let center = pos.get_bottom_center().add(0.0, 0.35, 0.0);
                let mut loot = stack(item, count);
                let limit = loot.get_max_stack_size();
                while count > limit {
                    self.spawn_drop(
                        stack(item, limit),
                        center,
                        Vec3::ZERO,
                        PickupDelay::Loot,
                        ctx,
                    );
                    count -= limit;
                }
                loot.set_count(count);
                self.spawn_drop(loot, center, Vec3::ZERO, PickupDelay::Loot, ctx);
            }
        }
        let slot = self.player.selection;
        let tool = &mut self.player.inventory[slot];
        if Mining::vanilla().hardness(block) != 0.0
            && miner.damage != 0
            && tool.is_damageable_item()
        {
            let unbreaking = enchantment_level(tool, "unbreaking");
            let damage = (0..miner.damage)
                .filter(|_| self.loot_random.next_int_bound(unbreaking + 1) == 0)
                .count() as i32;
            if damage > 0 {
                let next = tool.get_damage_value().saturating_add(damage);
                if next >= tool.get_max_damage() {
                    tool.shrink(1);
                } else {
                    tool.set_damage_value(next);
                }
                self.sync_slot(slot, wire, ctx.ids)?;
            }
        }
        Ok(())
    }

    fn use_block(
        &mut self,
        pos: BlockPos,
        direction: Direction,
        hit_y: f32,
        hand: usize,
        wire: &mut Wire,
        ctx: &PlayContext<'_>,
    ) -> io::Result<()> {
        let adjacent = pos.relative(direction);
        if !self.in_reach(pos) || self.player.mode.is_block_placing_restricted() {
            ctx.world.refresh_block(adjacent, ctx.audience, ctx.player);
            return Ok(());
        }
        let inventory_slot = self.hand_slot(hand);
        let held = item_id(&self.player.inventory[inventory_slot]);
        if let Some(id) = held {
            let kind = Blocks::vanilla().get(ctx.world.block(pos));
            let flint = id == named_id("flint_and_steel");
            let charge = id == named_id("fire_charge");
            if kind.name == "tnt" && (flint || charge) {
                if ctx.mobs.ignite(pos, ctx) && !self.player.mode.is_creative() {
                    let stack = &mut self.player.inventory[inventory_slot];
                    let damage = stack.get_damage_value() + 1;
                    if charge || damage >= stack.get_max_damage() {
                        stack.shrink(1);
                    } else {
                        stack.set_damage_value(damage);
                    }
                    self.sync_slot(inventory_slot, wire, ctx.ids)?;
                }
                return Ok(());
            }
            if let Some(mob) = mob_types::from_egg(id) {
                let target = if kind.replaceable { pos } else { adjacent };
                let position = Vec3::new(
                    f64::from(target.x) + 0.5,
                    f64::from(target.y),
                    f64::from(target.z) + 0.5,
                );
                let eye = self.player.position.add(0.0, 1.62, 0.0);
                let face = pos.get_center().add(
                    f64::from(direction.get_step_x()) * 0.501,
                    f64::from(direction.get_step_y()) * 0.501,
                    f64::from(direction.get_step_z()) * 0.501,
                );
                let world = ctx.world.snapshot();
                let visible = clear_sight_with(eye, face, |block| {
                    let state = world.block(block);
                    if Blocks::vanilla().get(state).replaceable {
                        AIR
                    } else {
                        state
                    }
                });
                if visible && ctx.mobs.egg(mob, position, ctx) && !self.player.mode.is_creative() {
                    self.player.inventory[inventory_slot].shrink(1);
                    self.sync_slot(inventory_slot, wire, ctx.ids)?;
                }
                return Ok(());
            }
        }
        if !(self.sneaking && held.is_some()) {
            if Blocks::vanilla().get(ctx.world.block(pos)).name == "crafting_table" && hand == 0 {
                self.close_crafting(wire, ctx, true)?;
                self.crafting.open(pos);
                const CRAFTING_MENU: i32 = 12;
                wire.send(
                    ctx.ids,
                    &ClientboundOpenScreenPacket::new(
                        self.crafting.id,
                        CRAFTING_MENU,
                        Component::translatable("container.crafting", Vec::new()),
                    ),
                )?;
                return self.sync_menu(wire, ctx);
            }
            if ctx.world.toggle_lever(pos, ctx.audience) {
                ctx.mobs.power_changed(pos, ctx);
                return Ok(());
            }
            if ctx.world.toggle_door(pos, ctx.audience) {
                return Ok(());
            }
        }
        // Buckets raycast from the player's eye in the use_item packet.
        // The client can send both use_item_on and use_item for one click.
        if held.is_some_and(is_bucket) {
            return Ok(());
        }
        let Some(block) = held.and_then(|id| Blocks::vanilla().for_item(id, direction)) else {
            return self.use_item(hand, wire, ctx);
        };
        let clicked = Blocks::vanilla().get(ctx.world.block(pos));
        let placed = if clicked.replaceable && clicked.item != held {
            pos
        } else {
            adjacent
        };
        let state = placement_state(block, direction, self.player.yaw, hit_y);
        let player = self.player.position;
        let intersects_player = |block: BlockPos| {
            let (x, y, z) = (f64::from(block.x), f64::from(block.y), f64::from(block.z));
            x < player.x + 0.3
                && x + 1.0 > player.x - 0.3
                && y < player.y + 1.8
                && y + 1.0 > player.y
                && z < player.z + 0.3
                && z + 1.0 > player.z - 0.3
        };
        if ctx
            .world
            .place_block(pos, placed, state, intersects_player, ctx.audience)
        {
            ctx.mobs.power_changed(placed, ctx);
            if !self.player.mode.is_creative() {
                self.player.inventory[inventory_slot].shrink(1);
                self.sync_slot(inventory_slot, wire, ctx.ids)?;
            }
        } else {
            ctx.world.refresh_block(placed, ctx.audience, ctx.player);
            if let Some((other, _, _)) = block.paired(placed, state) {
                ctx.world.refresh_block(other, ctx.audience, ctx.player);
            }
            self.sync_slot(inventory_slot, wire, ctx.ids)?;
        }
        Ok(())
    }

    fn sync_menu(&mut self, wire: &mut Wire, ctx: &PlayContext<'_>) -> io::Result<()> {
        wire.send(
            ctx.ids,
            &self.crafting.contents(&self.player.inventory, ctx.maps),
        )
    }

    pub fn check_crafting(&mut self, wire: &mut Wire, ctx: &PlayContext<'_>) -> io::Result<()> {
        if self.crafting.table.is_some_and(|pos| {
            !self.in_reach(pos)
                || Blocks::vanilla().get(ctx.world.block(pos)).name != "crafting_table"
        }) {
            self.close_crafting(wire, ctx, true)?;
        }
        Ok(())
    }

    pub fn close_crafting(
        &mut self,
        wire: &mut Wire,
        ctx: &PlayContext<'_>,
        notify: bool,
    ) -> io::Result<()> {
        let open = self.crafting.id;
        let has_items = self.crafting.recovery_items().next().is_some();
        let overflow = self.crafting.close(&mut self.player.inventory);
        for item in overflow {
            self.return_drop(item, ctx);
        }
        if notify && open != 0 {
            wire.send(ctx.ids, &ClientboundContainerClosePacket::new(open))?;
        }
        if open != 0 || has_items {
            self.sync_menu(wire, ctx)?;
        }
        Ok(())
    }

    pub fn restore_crafting(&mut self, wire: &mut Wire, ctx: &PlayContext<'_>) -> io::Result<()> {
        let mut changed = Vec::new();
        for mut item in std::mem::take(&mut self.player.recovery) {
            insert_inventory(&mut self.player.inventory[..36], &mut item, &mut changed);
            if !item.is_empty() {
                self.return_drop(item, ctx);
            }
        }
        if !changed.is_empty() {
            self.sync_menu(wire, ctx)?;
        }
        Ok(())
    }

    pub fn finish_session(&mut self) {
        self.player
            .recovery
            .extend(self.crafting.close(&mut self.player.inventory));
    }

    fn return_drop(&self, item: ItemStack, ctx: &PlayContext<'_>) {
        self.spawn_drop(
            item,
            self.player.position.add(0.0, 0.3, 0.0),
            Vec3::ZERO,
            PickupDelay::Loot,
            ctx,
        );
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
                self.throw_stack(extra, ctx);
            }
        }
        let extra = self.insert(stack(FIREWORK, 64), wire, ctx.ids)?;
        if !extra.is_empty() {
            self.throw_stack(extra, ctx);
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
        wire.send(
            ids,
            &gliding_packet(self.entity_id, gliding, self.player.environment.burning()),
        )
    }

    pub fn sync_environment(
        &mut self,
        environment: Environment,
        wire: &mut Wire,
        ids: &PacketIds,
    ) -> io::Result<()> {
        self.player.environment = environment;
        let air_changed = environment.air != self.sent_environment.air;
        let fire_changed = environment.burning() != self.sent_environment.burning();
        if !air_changed && !fire_changed {
            return Ok(());
        }
        let mut values = Vec::new();
        if air_changed {
            values.push(DataValue {
                id: 1,
                serializer: 1,
                value: Box::new(environment.air),
            });
        }
        if fire_changed {
            values.push(flags(self.gliding, environment.burning()));
        }
        wire.send(
            ids,
            &ClientboundSetEntityDataPacket::new(self.entity_id, values),
        )?;
        self.sent_environment = environment;
        Ok(())
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
            Some(id) if is_bucket(id) => self.use_bucket(slot, id, wire, ctx)?,
            Some(id) if id == named_id("map") => {
                let position = self.player.position;
                let id = ctx
                    .maps
                    .create(position.x.floor() as i32, position.z.floor() as i32)?;
                let mut filled = stack(named_id("filled_map"), 1);
                set_map_id(&mut filled, id);
                if !self.player.mode.is_creative() {
                    self.player.inventory[slot].shrink(1);
                }
                if self.player.inventory[slot].is_empty() {
                    self.player.inventory[slot] = filled;
                } else {
                    let overflow = self.insert(filled, wire, ctx.ids)?;
                    if !overflow.is_empty() {
                        self.return_drop(overflow, ctx);
                    }
                }
                self.sync_slot(slot, wire, ctx.ids)?;
            }
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
                let lifetime = i64::from(10 * (1 + duration));
                self.rockets
                    .push((entity, ctx.commands.game_time() + lifetime));
                if !self.player.mode.is_creative() {
                    self.player.inventory[slot].shrink(1);
                    self.sync_slot(slot, wire, ctx.ids)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn use_bucket(
        &mut self,
        slot: usize,
        item: i32,
        wire: &mut Wire,
        ctx: &PlayContext<'_>,
    ) -> io::Result<()> {
        if self.player.mode.is_block_placing_restricted() {
            return Ok(());
        }
        let empty = named_id("bucket");
        let collecting = item == empty;
        let creative = self.player.mode.is_creative();
        let eye = self
            .player
            .position
            .add(0.0, if self.sneaking { 1.27 } else { 1.62 }, 0.0);
        let direction = Vec3::direction_from_rotation(self.player.pitch, self.player.yaw);
        let hit = bucket_hit(
            &ctx.world.snapshot(),
            eye,
            direction,
            if creative { 5.0 } else { 4.5 },
            collecting,
        );
        let (position, fluid) = match hit {
            Some(BucketHit::Source(pos)) => {
                let Some(fluid) = ctx.world.take_fluid(pos, ctx.audience) else {
                    return Ok(());
                };
                (pos, fluid)
            }
            Some(BucketHit::Face { clicked, adjacent }) => {
                let fluid = if item == named_id("water_bucket") {
                    Fluid::Water
                } else {
                    Fluid::Lava
                };
                let Some(pos) = ctx
                    .world
                    .pour_bucket(clicked, adjacent, fluid, ctx.audience)
                else {
                    self.sync_slot(slot, wire, ctx.ids)?;
                    return Ok(());
                };
                (pos, fluid)
            }
            None => return Ok(()),
        };
        let product = if collecting {
            named_id(match fluid {
                Fluid::Water => "water_bucket",
                Fluid::Lava => "lava_bucket",
            })
        } else {
            empty
        };
        if !creative {
            self.player.inventory[slot].shrink(1);
            if self.player.inventory[slot].is_empty() {
                self.player.inventory[slot] = stack(product, 1);
            } else {
                let remainder = self.insert(stack(product, 1), wire, ctx.ids)?;
                if !remainder.is_empty() {
                    self.return_drop(remainder, ctx);
                }
            }
        } else if collecting
            && !self
                .player
                .inventory
                .iter()
                .any(|entry| item_id(entry) == Some(product))
        {
            let _ = self.insert(stack(product, 1), wire, ctx.ids)?;
        }
        self.sync_slot(slot, wire, ctx.ids)?;

        let sound = match (fluid, collecting) {
            (Fluid::Water, true) => "item.bucket.fill",
            (Fluid::Water, false) => "item.bucket.empty",
            (Fluid::Lava, true) => "item.bucket.fill_lava",
            (Fluid::Lava, false) => "item.bucket.empty_lava",
        };
        let center = position.get_center();
        let (x, y, z) = ClientboundSoundPacket::pack_position(center.x, center.y, center.z);
        ctx.audience.broadcast(
            ClientboundSoundPacket::new(
                Holder::direct(SoundEvent::new(Identifier::minecraft(sound), None)),
                SoundSource::Players,
                x,
                y,
                z,
                1.0,
                1.0,
                self.loot_random.next_long(),
            ),
            Some(chunk(position)),
        );
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
        let mut changed = Vec::new();
        insert_inventory(&mut self.player.inventory[..36], &mut stack, &mut changed);
        for slot in changed {
            self.sync_slot(slot, wire, ids)?;
        }
        Ok(stack)
    }

    fn throw_stack(&self, item: ItemStack, ctx: &PlayContext<'_>) {
        let direction = Vec3::direction_from_rotation(self.player.pitch, self.player.yaw);
        let pos = self.player.position.add(0.0, 1.32, 0.0);
        self.spawn_drop(item, pos, direction.scale(0.3), PickupDelay::Thrown, ctx);
    }

    pub fn drop_block(&self, pos: BlockPos, item: i32, ctx: &PlayContext<'_>) {
        let center = pos.get_bottom_center().add(0.0, 0.35, 0.0);
        self.spawn_drop(stack(item, 1), center, Vec3::ZERO, PickupDelay::Loot, ctx);
    }

    fn spawn_drop(
        &self,
        item: ItemStack,
        pos: Vec3,
        velocity: Vec3,
        delay: PickupDelay,
        ctx: &PlayContext<'_>,
    ) {
        let id = ctx.entity_ids.fetch_add(1, Ordering::Relaxed);
        ctx.drops
            .spawn(id, item, pos, velocity, ctx.commands.game_time(), delay);
    }

    pub fn tick(
        &mut self,
        wire: &mut Wire,
        ctx: &PlayContext<'_>,
        loaded: &HashSet<(i32, i32)>,
        subscription: &Subscription<'_>,
    ) -> io::Result<()> {
        if self.last_tick.elapsed() < TICK {
            return Ok(());
        }
        self.last_tick = Instant::now();
        let now = ctx.commands.game_time();
        let advanced = self.last_game_tick != now;
        self.last_game_tick = now;
        if advanced {
            self.tick_mining(now, wire, ctx)?;
        }
        for slot in [self.player.selection, OFFHAND] {
            if let Some(id) = map_id(&self.player.inventory[slot]) {
                if self.shown_maps.insert(id) {
                    if let Some(packet) = ctx.maps.render(id, ctx.world) {
                        wire.send(ctx.ids, &packet)?;
                    }
                }
            }
        }

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

        let mut changed = Vec::new();
        let collector =
            if !advanced || self.player.mode == GameType::Spectator || self.player.health <= 0.0 {
                0
            } else {
                self.entity_id
            };
        self.drops.sync(
            ctx,
            loaded,
            self.player.position,
            collector,
            now,
            |item| {
                insert_inventory(&mut self.player.inventory[..36], item, &mut changed);
            },
            wire,
            subscription,
        )?;
        changed.sort_unstable();
        changed.dedup();
        for slot in changed {
            self.sync_slot(slot, wire, ctx.ids)?;
        }
        if !removed.is_empty() {
            wire.send(ctx.ids, &ClientboundRemoveEntitiesPacket::new(removed))?;
        }
        Ok(())
    }
}

pub fn insert_inventory(
    inventory: &mut [ItemStack],
    stack: &mut ItemStack,
    changed: &mut Vec<usize>,
) {
    for (slot, current) in inventory.iter_mut().enumerate() {
        if stack.is_empty() {
            return;
        }
        if !current.is_empty() && ItemStack::is_same_item_same_components(current, stack) {
            let amount = (current.get_max_stack_size() - current.count)
                .max(0)
                .min(stack.count);
            if amount > 0 {
                current.grow(amount);
                stack.shrink(amount);
                changed.push(slot);
            }
        }
    }
    for (slot, current) in inventory.iter_mut().enumerate() {
        if stack.is_empty() {
            return;
        }
        if current.is_empty() {
            *current = stack.split(stack.get_max_stack_size());
            changed.push(slot);
        }
    }
}

fn is_bucket(item: i32) -> bool {
    [
        named_id("bucket"),
        named_id("water_bucket"),
        named_id("lava_bucket"),
    ]
    .contains(&item)
}

pub fn item_data(
    id: i32,
    item: ItemStack,
    burning: Option<bool>,
) -> ClientboundSetEntityDataPacket {
    let mut values = vec![data(8, 7, item)];
    if let Some(burning) = burning {
        values.push(flags(false, burning));
    }
    ClientboundSetEntityDataPacket::new(id, values)
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
