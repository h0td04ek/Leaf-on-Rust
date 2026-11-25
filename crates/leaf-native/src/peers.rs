use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use leaf_core::authlib::game_profile::GameProfile;
use leaf_core::core::{block_pos::BlockPos, holder::Holder};
use leaf_core::entity::{
    equipment_slot::EquipmentSlot, pose::Pose, position_move_rotation::PositionMoveRotation,
};
use leaf_core::network::protocol::game::{
    clientbound_add_entity_packet::ClientboundAddEntityPacket,
    clientbound_animate_packet::ClientboundAnimatePacket,
    clientbound_entity_position_sync_packet::ClientboundEntityPositionSyncPacket,
    clientbound_hurt_animation_packet::ClientboundHurtAnimationPacket,
    clientbound_player_info_remove_packet::ClientboundPlayerInfoRemovePacket,
    clientbound_player_info_update_packet::{
        ClientboundPlayerInfoUpdatePacket, PlayerInfoEntry, PlayerInfoUpdateAction,
    },
    clientbound_remove_entities_packet::ClientboundRemoveEntitiesPacket,
    clientbound_rotate_head_packet::ClientboundRotateHeadPacket,
    clientbound_set_entity_data_packet::ClientboundSetEntityDataPacket,
    clientbound_set_equipment_packet::ClientboundSetEquipmentPacket,
};
use leaf_core::network::syncher::synched_entity_data::DataValue;
use leaf_core::util::{mth, uuid::Uuid};
use leaf_core::world::{item::item_stack::ItemStack, level::game_type::GameType, phys::vec3::Vec3};

use crate::mining::enchantment_level;
use crate::player_environment::{Environment, Hazards, Rules};
use crate::wire::{PacketIds, Wire};
use crate::world::{entity_bounds, Fluid, NativeWorld, AIR};

const PLAYER_ENTITY_TYPE: i32 = 155; // Minecraft Java 1.21.11 entity_type registry.
const ATTACK_COOLDOWN: Duration = Duration::from_millis(500);
const HURT_COOLDOWN: i64 = 10;

#[derive(Clone, PartialEq)]
pub struct View {
    pub motion: PositionMoveRotation,
    pub on_ground: bool,
    pub gliding: bool,
    pub sneaking: bool,
    pub equipment: Vec<(EquipmentSlot, ItemStack)>,
    pub mode: GameType,
}

#[derive(Clone)]
struct Player {
    profile: GameProfile,
    view: View,
    health: f32,
    swings: u64,
    last_swing: i32,
    hurt: u64,
    burning: bool,
}

struct OnlinePlayer {
    player: Player,
    last_attack: Option<Instant>,
    last_hurt: Option<i64>,
    hazards: Hazards,
    pending_hit: Option<Hit>,
    pending_mode: Option<GameType>,
}

#[derive(Default)]
pub struct Peers(Mutex<HashMap<i32, OnlinePlayer>>);

pub struct Hit {
    pub health: f32,
    pub velocity: Option<Vec3>,
    pub death_message: String,
}

#[derive(Clone, Copy)]
pub struct MobTarget {
    pub id: i32,
    pub position: Vec3,
    pub vulnerable: bool,
}

impl Peers {
    pub fn mob_targets(&self) -> Vec<MobTarget> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .filter_map(|(&id, online)| {
                let player = &online.player;
                (player.health > 0.0 && player.view.mode != GameType::Spectator).then(|| {
                    MobTarget {
                        id,
                        position: player.view.motion.position,
                        vulnerable: player.view.mode.is_survival(),
                    }
                })
            })
            .collect()
    }

    pub fn tick_environment(&self, tick: i64, rules: Rules, world: &NativeWorld) {
        let mut players = self.0.lock().unwrap();
        if players.is_empty() {
            return;
        }
        let blocks = world.snapshot();
        for online in players.values_mut() {
            let player = &mut online.player;
            if player.health == 0.0 || !player.view.mode.is_survival() {
                online.hazards.clear();
                player.burning = false;
                continue;
            }

            let view = &player.view;
            let position = view.motion.position;
            let (height, eye_height) = if view.gliding {
                (0.6, 0.4)
            } else if view.sneaking {
                (1.5, 1.27)
            } else {
                (1.8, 1.62)
            };
            let underwater =
                blocks.fluid_at(position.add(0.0, eye_height, 0.0)) == Some(Fluid::Water);
            let respiration = if underwater {
                view.equipment
                    .iter()
                    .find(|(slot, _)| *slot == EquipmentSlot::Head)
                    .map_or(0, |(_, item)| enchantment_level(item, "respiration"))
            } else {
                0
            };
            let (in_water, in_lava) = blocks.touching_fluids(entity_bounds(position, 0.6, height));
            let hazard = online
                .hazards
                .tick(underwater, in_water, in_lava, respiration, rules);
            player.burning = online.hazards.environment().burning;
            if let Some(hazard) = hazard.filter(|_| can_hurt(online, tick)) {
                let message = format!("{} {}", online.player.profile.name, hazard.death());
                apply_hit(online, hazard.damage(), None, message, tick);
            }
        }
    }

    pub fn hurt(&self, id: i32, damage: f32, velocity: Vec3, source: String, tick: i64) {
        let mut players = self.0.lock().unwrap();
        let Some(target) = players.get_mut(&id) else {
            return;
        };
        if !can_hurt(target, tick) {
            return;
        }
        let name = &target.player.profile.name;
        let message = if source == "explosion" {
            format!("{name} blew up")
        } else {
            format!("{name} was slain by {source}")
        };
        apply_hit(target, damage, Some(velocity), message, tick);
    }

    pub fn join(&self, id: i32, profile: GameProfile, view: View, health: f32) -> Presence<'_> {
        let profile_seed = profile.id.least_significant_bits;
        let player = Player {
            profile,
            view,
            health,
            swings: 0,
            last_swing: 0,
            hurt: 0,
            burning: false,
        };
        self.0.lock().unwrap().insert(
            id,
            OnlinePlayer {
                player,
                last_attack: None,
                last_hurt: None,
                hazards: Hazards::new(profile_seed),
                pending_hit: None,
                pending_mode: None,
            },
        );
        Presence { peers: self, id }
    }

    pub fn set_mode(&self, name: &str, mode: GameType) -> bool {
        let mut online = self.0.lock().unwrap();
        let Some(player) = online.values_mut().find(|p| p.player.profile.name == name) else {
            return false;
        };
        player.pending_mode = Some(mode);
        true
    }

    fn snapshot(&self, exclude: i32) -> HashMap<i32, Player> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .filter(|(id, _)| **id != exclude)
            .map(|(&id, player)| (id, player.player.clone()))
            .collect()
    }
}

pub struct Presence<'a> {
    peers: &'a Peers,
    pub id: i32,
}

impl Presence<'_> {
    fn update(&self, change: impl FnOnce(&mut OnlinePlayer)) {
        if let Some(player) = self.peers.0.lock().unwrap().get_mut(&self.id) {
            change(player);
        }
    }

    pub fn publish(&self, view: View) {
        self.update(|player| player.player.view = view);
    }

    pub fn swing(&self, action: i32) {
        self.update(|player| {
            player.player.swings = player.player.swings.wrapping_add(1);
            player.player.last_swing = action;
        });
    }

    pub fn take_mode(&self) -> Option<GameType> {
        let mut mode = None;
        self.update(|player| mode = player.pending_mode.take());
        mode
    }

    pub fn take_updates(&self) -> (Option<Hit>, Environment) {
        let mut updates = (None, Environment::default());
        self.update(|player| updates = (player.pending_hit.take(), player.hazards.environment()));
        updates
    }

    pub fn health(&self) -> f32 {
        self.peers
            .0
            .lock()
            .unwrap()
            .get(&self.id)
            .map_or(20.0, |player| player.player.health)
    }

    pub fn respawn(&self, view: View) -> bool {
        let mut respawned = false;
        self.update(|online| {
            if online.player.health > 0.0 {
                return;
            }
            online.player.health = 20.0;
            online.player.view = view;
            online.player.burning = false;
            online.hazards.clear();
            online.last_hurt = None;
            online.last_attack = None;
            online.pending_hit = None;
            respawned = true;
        });
        respawned
    }

    pub fn leave(self) -> f32 {
        self.peers
            .0
            .lock()
            .unwrap()
            .remove(&self.id)
            .map_or(20.0, |player| player.player.health)
    }

    pub fn attack(&self, target_id: i32, world: &NativeWorld, tick: i64) {
        if self.id == target_id {
            return;
        }
        let mut players = self.peers.0.lock().unwrap();
        let now = Instant::now();
        let Some(attacker) = players.get(&self.id) else {
            return;
        };
        let source = &attacker.player;
        if source.health == 0.0
            || source.view.mode == GameType::Spectator
            || attacker
                .last_attack
                .is_some_and(|last| now.duration_since(last) < ATTACK_COOLDOWN)
        {
            return;
        }
        let Some(target) = players.get(&target_id) else {
            return;
        };
        let destination = &target.player;
        if !can_hurt(target, tick) {
            return;
        }
        let from = source.view.motion.position;
        let to = destination.view.motion.position;
        let reach = if source.view.mode.is_creative() {
            5.0
        } else {
            3.2
        };
        if from.distance_to_sqr(&to) > reach * reach
            || !clear_sight(world, from.add(0.0, 1.62, 0.0), to.add(0.0, 1.1, 0.0))
        {
            return;
        }
        let damage = source
            .view
            .equipment
            .iter()
            .find(|(slot, _)| *slot == EquipmentSlot::Mainhand)
            .map_or(1.0, |(_, item)| attack_damage(item));
        let direction = Vec3::new(to.x - from.x, 0.0, to.z - from.z).normalize();
        let velocity = direction.scale(0.4).add(0.0, 0.36, 0.0);
        let message = format!(
            "{} was slain by {}",
            destination.profile.name, source.profile.name
        );
        players.get_mut(&self.id).unwrap().last_attack = Some(now);
        apply_hit(
            players.get_mut(&target_id).unwrap(),
            damage,
            Some(velocity),
            message,
            tick,
        );
    }
}

impl Drop for Presence<'_> {
    fn drop(&mut self) {
        self.peers.0.lock().unwrap().remove(&self.id);
    }
}

fn can_hurt(target: &OnlinePlayer, tick: i64) -> bool {
    target.player.health > 0.0
        && target.player.view.mode.is_survival()
        && target
            .last_hurt
            .is_none_or(|last| tick.saturating_sub(last) >= HURT_COOLDOWN)
}

fn apply_hit(
    target: &mut OnlinePlayer,
    damage: f32,
    velocity: Option<Vec3>,
    death_message: String,
    tick: i64,
) {
    target.last_hurt = Some(tick);
    let health = (target.player.health - damage).max(0.0);
    target.player.health = health;
    target.player.hurt = target.player.hurt.wrapping_add(1);
    target.pending_hit = Some(Hit {
        health,
        velocity,
        death_message,
    });
}

pub fn attack_damage(item: &ItemStack) -> f32 {
    item.item_identifier()
        .map_or(1.0, |id| match id.get_path() {
            "wooden_sword" | "golden_sword" => 4.0,
            "stone_sword" => 5.0,
            "iron_sword" => 6.0,
            "diamond_sword" => 7.0,
            "netherite_sword" => 8.0,
            _ => 1.0,
        })
}

pub fn clear_sight(world: &NativeWorld, from: Vec3, to: Vec3) -> bool {
    clear_sight_with(from, to, |pos| world.block(pos))
}

pub fn clear_sight_with(from: Vec3, to: Vec3, block_at: impl Fn(BlockPos) -> i32) -> bool {
    let direction = Vec3::new(to.x - from.x, to.y - from.y, to.z - from.z);
    let steps = (from.distance_to_sqr(&to).sqrt() * 4.0).ceil() as i32;
    (1..steps).all(|step| {
        let offset = direction.scale(f64::from(step) / f64::from(steps));
        let position = from.add(offset.x, offset.y, offset.z);
        block_at(BlockPos::new(
            position.x.floor() as i32,
            position.y.floor() as i32,
            position.z.floor() as i32,
        )) == AIR
    })
}

#[derive(Default)]
pub struct PeerTracker {
    profiles: HashMap<i32, (Uuid, GameType)>,
    visible: HashMap<i32, Player>,
}

impl PeerTracker {
    pub fn sees(&self, entity: i32) -> bool {
        self.visible.contains_key(&entity)
    }

    pub fn reset_entities(&mut self, wire: &mut Wire, ids: &PacketIds) -> io::Result<()> {
        if !self.visible.is_empty() {
            wire.send(
                ids,
                &ClientboundRemoveEntitiesPacket::new(self.visible.keys().copied().collect()),
            )?;
            self.visible.clear();
        }
        Ok(())
    }
    pub fn sync(
        &mut self,
        presence: &Presence<'_>,
        loaded: &HashSet<(i32, i32)>,
        wire: &mut Wire,
        ids: &PacketIds,
    ) -> io::Result<()> {
        let others = presence.peers.snapshot(presence.id);
        let mut removed_entities = Vec::new();
        self.visible.retain(|id, _| {
            let remains = others.get(id).is_some_and(|player| visible(player, loaded));
            if !remains {
                removed_entities.push(*id);
            }
            remains
        });
        if !removed_entities.is_empty() {
            wire.send(ids, &ClientboundRemoveEntitiesPacket::new(removed_entities))?;
        }

        let mut removed_profiles = Vec::new();
        self.profiles.retain(|id, (uuid, _)| {
            let remains = others.contains_key(id);
            if !remains {
                removed_profiles.push(*uuid);
            }
            remains
        });
        if !removed_profiles.is_empty() {
            wire.send(
                ids,
                &ClientboundPlayerInfoRemovePacket::new(removed_profiles),
            )?;
        }

        for (id, player) in others {
            let known_mode = self.profiles.get(&id).map(|(_, mode)| *mode);
            if known_mode != Some(player.view.mode) {
                let mut entry = PlayerInfoEntry::new_minimal(player.profile.id, true);
                entry.game_mode = player.view.mode;
                if known_mode.is_none() {
                    entry.profile = Some(player.profile.clone());
                    wire.send(
                        ids,
                        &ClientboundPlayerInfoUpdatePacket::create_player_initializing(vec![entry]),
                    )?;
                } else {
                    wire.send(
                        ids,
                        &ClientboundPlayerInfoUpdatePacket::new(
                            [PlayerInfoUpdateAction::UpdateGameMode]
                                .into_iter()
                                .collect(),
                            vec![entry],
                        ),
                    )?;
                }
                self.profiles
                    .insert(id, (player.profile.id, player.view.mode));
            }
            if visible(&player, loaded) {
                self.update_entity(id, player, wire, ids)?;
            }
        }
        Ok(())
    }

    fn update_entity(
        &mut self,
        id: i32,
        player: Player,
        wire: &mut Wire,
        ids: &PacketIds,
    ) -> io::Result<()> {
        let previous = self.visible.get(&id);
        let view = &player.view;
        let motion = view.motion;
        if previous.is_none() {
            wire.send(
                ids,
                &ClientboundAddEntityPacket::new(
                    id,
                    player.profile.id,
                    Holder::Reference(PLAYER_ENTITY_TYPE),
                    motion.position.x,
                    motion.position.y,
                    motion.position.z,
                    motion.delta_movement,
                    motion.x_rot,
                    motion.y_rot,
                    f64::from(motion.y_rot),
                    0,
                ),
            )?;
        } else if previous
            .is_some_and(|old| old.view.motion != motion || old.view.on_ground != view.on_ground)
        {
            wire.send(
                ids,
                &ClientboundEntityPositionSyncPacket::new(id, motion, view.on_ground),
            )?;
            wire.send(
                ids,
                &ClientboundRotateHeadPacket::new(id, mth::pack_degrees(motion.y_rot)),
            )?;
        }
        if previous
            .is_none_or(|old| old.view.gliding != view.gliding || old.burning != player.burning)
        {
            wire.send(ids, &gliding_packet(id, view.gliding, player.burning))?;
        }
        if previous.is_none_or(|old| old.view.equipment != view.equipment) {
            wire.send(
                ids,
                &ClientboundSetEquipmentPacket::new(id, view.equipment.clone()),
            )?;
        }
        if previous.is_some_and(|old| old.swings != player.swings) {
            wire.send(ids, &ClientboundAnimatePacket::new(id, player.last_swing))?;
        }
        if previous.is_some_and(|old| old.hurt != player.hurt) {
            wire.send(ids, &ClientboundHurtAnimationPacket::new(id, 0.0))?;
        }
        self.visible.insert(id, player);
        Ok(())
    }
}

pub fn flags(gliding: bool, burning: bool) -> DataValue {
    DataValue {
        id: 0,
        serializer: 0,
        value: Box::new((if gliding { -128_i8 } else { 0 }) | i8::from(burning)),
    }
}

pub fn gliding_packet(id: i32, gliding: bool, burning: bool) -> ClientboundSetEntityDataPacket {
    ClientboundSetEntityDataPacket::new(
        id,
        vec![
            flags(gliding, burning),
            DataValue {
                id: 6,
                serializer: 20,
                value: Box::new(if gliding {
                    Pose::FallFlying
                } else {
                    Pose::Standing
                }),
            },
        ],
    )
}

fn visible(player: &Player, loaded: &HashSet<(i32, i32)>) -> bool {
    let position = player.view.motion.position;
    player.health > 0.0
        && player.view.mode != GameType::Spectator
        && loaded.contains(&(
            position.x.floor() as i32 >> 4,
            position.z.floor() as i32 >> 4,
        ))
}
