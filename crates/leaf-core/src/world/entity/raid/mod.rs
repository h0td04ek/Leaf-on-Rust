//! Перенос `net.minecraft.world.entity.raid` —
//! `Raid`, `Raids`, `Raid.RaiderType`, `Raid.RaidStatus`.
//!
//! Отличия от Java-оригинала по архитектуре Rust-Java-моста:
//! - `ServerBossEvent raidEvent` заменён на `raid_event_progress`/
//!   `raid_event_name` хуки (`update_bossbar`/`set_name`).
//! - Спавн рейдеров — через `LevelReader::spawn_entity_of_type`.
//! - `Raider`-состояние (wave/currentRaid/canJoinRaid/...) живёт в
//!   `LivingEntity`-хуках `raider_*` (view-паттерн проекта).
//! - NBT/codec `MAP_CODEC` отложен до saved-data слоя; поля
//!   соответствуют Java record-полям.

use crate::core::block_pos::BlockPos;
use crate::resources::identifier::Identifier;
use crate::util::uuid::Uuid;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::vec3::Vec3;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// `Raid.RaidStatus` — `StringRepresentable` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RaidStatus {
    /// `ONGOING`.
    Ongoing,
    /// `VICTORY`.
    Victory,
    /// `LOSS`.
    Loss,
    /// `STOPPED`.
    Stopped,
}

impl RaidStatus {
    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            RaidStatus::Ongoing => "ongoing",
            RaidStatus::Victory => "victory",
            RaidStatus::Loss => "loss",
            RaidStatus::Stopped => "stopped",
        }
    }
}

/// `Difficulty` — используется существующий `world::difficulty::Difficulty`.

/// `Raid.RaiderType` — тип волны рейда + `spawnsPerWaveBeforeBonus`
/// (8 значений: волны 0..7; bonus-волна использует индекс numGroups).
#[derive(Debug, Clone, Copy)]
pub struct RaiderType {
    /// `entityType` — id типа сущности.
    pub entity_type: &'static str,
    /// `spawnsPerWaveBeforeBonus`.
    pub spawns_per_wave_before_bonus: [i32; 8],
}

/// `Raid.RaiderType.VALUES` — порядок итерации спавна.
pub const RAIDER_TYPES: [RaiderType; 5] = [
    RaiderType { entity_type: "vindicator", spawns_per_wave_before_bonus: [0, 0, 2, 0, 1, 4, 2, 5] },
    RaiderType { entity_type: "evoker", spawns_per_wave_before_bonus: [0, 0, 0, 0, 0, 1, 1, 2] },
    RaiderType { entity_type: "pillager", spawns_per_wave_before_bonus: [0, 4, 3, 3, 4, 4, 4, 2] },
    RaiderType { entity_type: "witch", spawns_per_wave_before_bonus: [0, 0, 0, 0, 3, 0, 0, 1] },
    RaiderType { entity_type: "ravager", spawns_per_wave_before_bonus: [0, 0, 0, 1, 0, 1, 0, 2] },
];

/// `Raid` — босс-рейд на деревню.
pub struct Raid {
    /// `groupToLeaderMap` — wave → leader raider (uuid).
    pub group_to_leader_map: HashMap<i32, Uuid>,
    /// `groupRaiderMap` — wave → set raider uuid.
    pub group_raider_map: HashMap<i32, HashSet<Uuid>>,
    /// `heroesOfTheVillage`.
    pub heroes_of_the_village: HashSet<Uuid>,
    /// `ticksActive`.
    pub ticks_active: i64,
    /// `center`.
    pub center: BlockPos,
    /// `started`.
    pub started: bool,
    /// `totalHealth`.
    pub total_health: f32,
    /// `raidOmenLevel`.
    pub raid_omen_level: i32,
    /// `active`.
    pub active: bool,
    /// `groupsSpawned`.
    pub groups_spawned: i32,
    /// `postRaidTicks`.
    pub post_raid_ticks: i32,
    /// `raidCooldownTicks`.
    pub raid_cooldown_ticks: i32,
    /// `numGroups`.
    pub num_groups: i32,
    /// `status`.
    pub status: RaidStatus,
    /// `celebrationTicks`.
    pub celebration_ticks: i32,
    /// `waveSpawnPos`.
    pub wave_spawn_pos: Option<BlockPos>,
    /// `isBarDirty`.
    pub is_bar_dirty: bool,
    /// `raidEvent` заменён: `raid_event_progress` (0..1) и
    /// `raid_event_name` — через LevelReader-хуки `raid_boss_*`.
    pub raid_event_progress: f32,
    /// Имя босс-бара (translatable key или суффикс raiders_remaining).
    pub raid_event_name_suffix: Option<String>,
    /// Кэш суммы health рейдеров (обновляется в `tick`).
    pub health_sum_cache: f32,
    /// `idOrNegativeOne` — id в `Raids.raidMap` (-1 до регистрации).
    pub raid_id_value: i32,
}

impl Raid {
    /// `ALLOW_SPAWNING_WITHIN_VILLAGE_SECONDS_THRESHOLD`.
    pub const ALLOW_SPAWNING_WITHIN_VILLAGE_SECONDS_THRESHOLD: i32 = 7;
    /// `SECTION_RADIUS_FOR_FINDING_NEW_VILLAGE_CENTER`.
    pub const SECTION_RADIUS_FOR_FINDING_NEW_VILLAGE_CENTER: i32 = 2;
    /// `VILLAGE_SEARCH_RADIUS`.
    pub const VILLAGE_SEARCH_RADIUS: i32 = 32;
    /// `RAID_TIMEOUT_TICKS`.
    pub const RAID_TIMEOUT_TICKS: i64 = 48000;
    /// `NUM_SPAWN_ATTEMPTS`.
    pub const NUM_SPAWN_ATTEMPTS: i32 = 5;
    /// `VILLAGE_RADIUS_BUFFER`.
    pub const VILLAGE_RADIUS_BUFFER: i32 = 16;
    /// `POST_RAID_TICK_LIMIT`.
    pub const POST_RAID_TICK_LIMIT: i32 = 40;
    /// `DEFAULT_PRE_RAID_TICKS`.
    pub const DEFAULT_PRE_RAID_TICKS: i32 = 300;
    /// `MAX_NO_ACTION_TIME`.
    pub const MAX_NO_ACTION_TIME: i32 = 2400;
    /// `MAX_CELEBRATION_TICKS`.
    pub const MAX_CELEBRATION_TICKS: i32 = 600;
    /// `OUTSIDE_RAID_BOUNDS_TIMEOUT`.
    pub const OUTSIDE_RAID_BOUNDS_TIMEOUT: i32 = 30;
    /// `DEFAULT_MAX_RAID_OMEN_LEVEL`.
    pub const DEFAULT_MAX_RAID_OMEN_LEVEL: i32 = 5;
    /// `LOW_MOB_THRESHOLD`.
    pub const LOW_MOB_THRESHOLD: i32 = 2;
    /// `VALID_RAID_RADIUS`.
    pub const VALID_RAID_RADIUS: i32 = 96;
    /// `VALID_RAID_RADIUS_SQR`.
    pub const VALID_RAID_RADIUS_SQR: i64 = 9216;
    /// `RAID_REMOVAL_THRESHOLD_SQR`.
    pub const RAID_REMOVAL_THRESHOLD_SQR: f64 = 12544.0;
    /// `HERO_OF_THE_VILLAGE_DURATION`.
    pub const HERO_OF_THE_VILLAGE_DURATION: i32 = 48000;

    /// `new Raid(center, difficulty)`.
    pub fn new(center: BlockPos, difficulty: crate::world::difficulty::Difficulty) -> Self {
        Self {
            group_to_leader_map: HashMap::new(),
            group_raider_map: HashMap::new(),
            heroes_of_the_village: HashSet::new(),
            ticks_active: 0,
            center,
            started: false,
            total_health: 0.0,
            raid_omen_level: 0,
            active: true,
            groups_spawned: 0,
            post_raid_ticks: 0,
            raid_cooldown_ticks: 300,
            num_groups: Self::get_num_groups(difficulty),
            status: RaidStatus::Ongoing,
            celebration_ticks: 0,
            wave_spawn_pos: None,
            is_bar_dirty: false,
            raid_event_progress: 0.0,
            raid_event_name_suffix: None,
            health_sum_cache: 0.0,
            raid_id_value: -1,
        }
    }

    /// `isOver()`.
    pub fn is_over(&self) -> bool {
        self.is_victory() || self.is_loss()
    }

    /// `isBetweenWaves()`.
    pub fn is_between_waves(&self, total_raiders_alive: i32) -> bool {
        self.has_first_wave_spawned()
            && total_raiders_alive == 0
            && self.raid_cooldown_ticks > 0
    }

    /// `hasFirstWaveSpawned()`.
    pub fn has_first_wave_spawned(&self) -> bool {
        self.groups_spawned > 0
    }

    /// `isStopped()`.
    pub fn is_stopped(&self) -> bool {
        self.status == RaidStatus::Stopped
    }

    /// `isVictory()`.
    pub fn is_victory(&self) -> bool {
        self.status == RaidStatus::Victory
    }

    /// `isLoss()`.
    pub fn is_loss(&self) -> bool {
        self.status == RaidStatus::Loss
    }

    /// CraftBukkit `isInProgress()`.
    pub fn is_in_progress(&self) -> bool {
        self.status == RaidStatus::Ongoing
    }

    /// `getTotalHealth()`.
    pub fn get_total_health(&self) -> f32 {
        self.total_health
    }

    /// `getAllRaiders()` — все uuid рейдеров.
    pub fn get_all_raiders(&self) -> HashSet<Uuid> {
        self.group_raider_map.values().flatten().copied().collect()
    }

    /// `isStarted()`.
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// `getGroupsSpawned()`.
    pub fn get_groups_spawned(&self) -> i32 {
        self.groups_spawned
    }

    /// `getRaidOmenLevel()`.
    pub fn get_raid_omen_level(&self) -> i32 {
        self.raid_omen_level
    }

    /// `setRaidOmenLevel`.
    pub fn set_raid_omen_level(&mut self, level: i32) {
        self.raid_omen_level = level;
    }

    /// `getMaxRaidOmenLevel()`.
    pub fn get_max_raid_omen_level(&self) -> i32 {
        Self::DEFAULT_MAX_RAID_OMEN_LEVEL
    }

    /// `stop()` — active=false, clear players, STOPPED.
    pub fn stop(&mut self) {
        self.active = false;
        self.status = RaidStatus::Stopped;
        // raidEvent.removeAllPlayers() — через level-хук.
    }

    /// `isActive()`.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// `getCenter()`.
    pub fn get_center(&self) -> BlockPos {
        self.center
    }

    /// `setCenter`.
    pub fn set_center(&mut self, center: BlockPos) {
        self.center = center;
    }

    /// `getNumGroups(difficulty)` — static по сложности.
    pub fn get_num_groups(difficulty: crate::world::difficulty::Difficulty) -> i32 {
        match difficulty {
            crate::world::difficulty::Difficulty::Peaceful => 0,
            crate::world::difficulty::Difficulty::Easy => 3,
            crate::world::difficulty::Difficulty::Normal => 5,
            crate::world::difficulty::Difficulty::Hard => 7,
        }
    }

    /// `getEnchantOdds()` — по raidOmenLevel.
    pub fn get_enchant_odds(&self) -> f32 {
        match self.raid_omen_level {
            2 => 0.1,
            3 => 0.25,
            4 => 0.5,
            5 => 0.75,
            _ => 0.0,
        }
    }

    /// `addHeroOfTheVillage(player)`.
    pub fn add_hero_of_the_village(&mut self, player_uuid: Uuid) {
        self.heroes_of_the_village.insert(player_uuid);
    }

    /// `getTotalRaidersAlive()`.
    pub fn get_total_raiders_alive(&self) -> i32 {
        self.group_raider_map.values().map(|s| s.len() as i32).sum()
    }

    /// `hasMoreWaves()`.
    pub fn has_more_waves(&self) -> bool {
        if self.has_bonus_wave() {
            !self.has_spawned_bonus_wave()
        } else {
            !self.is_final_wave()
        }
    }

    /// `isFinalWave()`.
    pub fn is_final_wave(&self) -> bool {
        self.groups_spawned == self.num_groups
    }

    /// `hasBonusWave()`.
    pub fn has_bonus_wave(&self) -> bool {
        self.raid_omen_level > 1
    }

    /// `hasSpawnedBonusWave()`.
    pub fn has_spawned_bonus_wave(&self) -> bool {
        self.groups_spawned > self.num_groups
    }

    /// `shouldSpawnBonusGroup()`.
    pub fn should_spawn_bonus_group(&self) -> bool {
        self.is_final_wave()
            && self.get_total_raiders_alive() == 0
            && self.has_bonus_wave()
    }

    /// `shouldSpawnGroup()`.
    pub fn should_spawn_group(&self) -> bool {
        self.raid_cooldown_ticks == 0
            && (self.groups_spawned < self.num_groups || self.should_spawn_bonus_group())
            && self.get_total_raiders_alive() == 0
    }

    /// `getLeader(wave)` → uuid.
    pub fn get_leader(&self, wave: i32) -> Option<Uuid> {
        self.group_to_leader_map.get(&wave).copied()
    }

    /// `setLeader(wave, raider_uuid)` — leader + ominous banner
    /// (banner-экипировку выставляет вызывающая сторона через
    /// `raider_set_item_slot`-хук).
    pub fn set_leader(&mut self, wave: i32, raider_uuid: Uuid) {
        self.group_to_leader_map.insert(wave, raider_uuid);
    }

    /// `removeLeader(wave)`.
    pub fn remove_leader(&mut self, wave: i32) {
        self.group_to_leader_map.remove(&wave);
    }

    /// `addWaveMob(level, wave, raider_uuid, isRecruited)` —
    /// дедупликация по uuid, totalHealth для recruited.
    /// `raider_health` — текущий health рейдера.
    pub fn add_wave_mob(
        &mut self,
        wave: i32,
        raider_uuid: Uuid,
        raider_health: f32,
        is_recruited: bool,
    ) -> bool {
        self.group_raider_map.entry(wave).or_default().insert(raider_uuid);
        if is_recruited {
            self.total_health += raider_health;
        }
        self.update_bossbar();
        true
    }

    /// `removeFromRaid(level, raider, wanderedOutOfRaid)` —
    /// `raider_uuid`, `raider_health`, `raider_wave`.
    pub fn remove_from_raid(
        &mut self,
        raider_uuid: Uuid,
        raider_health: f32,
        raider_wave: i32,
        wandered_out_of_raid: bool,
    ) -> bool {
        if let Some(set) = self.group_raider_map.get_mut(&raider_wave) {
            if set.remove(&raider_uuid) {
                if wandered_out_of_raid {
                    self.total_health -= raider_health;
                }
                self.update_bossbar();
                return true;
            }
        }
        false
    }

    /// `updateBossbar()` — пометить грязным.
    pub fn update_bossbar(&mut self) {
        self.is_bar_dirty = true;
    }

    /// `updateBossbarInternal()` — progress = health/total.
    pub fn update_bossbar_internal(&mut self) {
        self.raid_event_progress = if self.total_health > 0.0 {
            self.health_sum_cache / self.total_health
        } else {
            0.0
        };
    }

    /// `getHealthOfLivingRaiders()` — сумма health живых рейдеров
    /// (резолв uuid → entity через `LevelReader::entity_by_uuid`).
    pub fn get_health_of_living_raiders(&self, level: &dyn LevelReader) -> f32 {
        let mut f = 0.0f32;
        for set in self.group_raider_map.values() {
            for uuid in set {
                if let Some(e) = level.entity_by_uuid(*uuid) {
                    let le = crate::world::entity::living_entity::LivingEntity::from_entity(e);
                    if le.is_alive() {
                        f += le.get_health();
                    }
                }
            }
        }
        f
    }

    /// `absorbRaidOmen(player)` — уровень omen + amplifier+1,
    /// clamp 0..max. `has_first_wave` и stat/trigger — через хуки.
    pub fn absorb_raid_omen(&mut self, omen_level: i32) -> bool {
        if omen_level <= 0 {
            return false;
        }
        self.raid_omen_level += omen_level;
        self.raid_omen_level = crate::util::mth::clamp_i32(
            self.raid_omen_level,
            0,
            self.get_max_raid_omen_level(),
        );
        true
    }

    /// `getDefaultNumSpawns(raiderType, wave, shouldSpawnBonusGroup)`.
    fn get_default_num_spawns(
        raider_type: &RaiderType,
        wave: i32,
        num_groups: i32,
        should_spawn_bonus_group: bool,
    ) -> i32 {
        if should_spawn_bonus_group {
            raider_type.spawns_per_wave_before_bonus[num_groups as usize]
        } else {
            raider_type.spawns_per_wave_before_bonus[wave as usize]
        }
    }

    /// `getPotentialBonusSpawns(raiderType, random, wave,
    /// difficulty, shouldSpawnBonusGroup)`.
    fn get_potential_bonus_spawns(
        raider_type: &RaiderType,
        random: &mut dyn crate::random::RandomSource,
        wave: i32,
        difficulty: crate::world::difficulty::Difficulty,
        should_spawn_bonus_group: bool,
    ) -> i32 {
        let is_easy = difficulty == crate::world::difficulty::Difficulty::Easy;
        let is_normal = difficulty == crate::world::difficulty::Difficulty::Normal;
        let i = match raider_type.entity_type {
            "vindicator" | "pillager" => {
                if is_easy {
                    random.next_int_bound(2)
                } else if is_normal {
                    1
                } else {
                    2
                }
            }
            "witch" => {
                if is_easy || wave <= 2 || wave == 4 {
                    return 0;
                }
                1
            }
            "ravager" => {
                if !is_easy && should_spawn_bonus_group {
                    1
                } else {
                    0
                }
            }
            _ => return 0,
        };
        if i > 0 {
            random.next_int_bound(i + 1)
        } else {
            0
        }
    }

    /// `joinRaid(level, wave, raider_uuid, pos, isRecruited)` —
    /// addWaveMob + установка состояния рейдера через
    /// LivingEntity-хуки (setCurrentRaid/setWave/... выполняет
    /// вызывающая сторона — здесь только учёт в groupRaiderMap).
    pub fn join_raid(
        &mut self,
        wave: i32,
        raider_uuid: Uuid,
        raider_health: f32,
        is_recruited: bool,
    ) -> bool {
        self.add_wave_mob(wave, raider_uuid, raider_health, is_recruited)
    }
}

mod raids;
mod tick;
pub use raids::Raids;
