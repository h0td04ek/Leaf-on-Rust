use super::*;

impl Raid {
    /// `getValidSpawnPos(level)` — `findRandomSpawnPos(level, 8)`.
    pub fn get_valid_spawn_pos(
        &self,
        level: &dyn LevelReader,
        random: &mut dyn crate::random::RandomSource,
    ) -> Option<BlockPos> {
        self.find_random_spawn_pos(level, random, 8)
    }

    /// `moveRaidCenterToNearbyVillageSection(level)` — ближайший
    /// village section-center в радиусе 2 секций.
    pub fn move_raid_center_to_nearby_village_section(&mut self, level: &dyn LevelReader) {
        // SectionPos.cube(center, 2) → фильтр isVillage → min distSqr.
        let section = crate::core::section_pos::SectionPos::of(&self.center);
        let mut best: Option<BlockPos> = None;
        let mut best_d = f64::MAX;
        let radius = Self::SECTION_RADIUS_FOR_FINDING_NEW_VILLAGE_CENTER;
        for dx in -radius..=radius {
            for dz in -radius..=radius {
                let sp = crate::core::section_pos::SectionPos::of_i(
                    section.x + dx,
                    section.y,
                    section.z + dz,
                );
                if !level.is_village_section(sp) {
                    continue;
                }
                let center = sp.center();
                let d = center.dist_sqr(self.center);
                if d < best_d {
                    best_d = d;
                    best = Some(center);
                }
            }
        }
        if let Some(c) = best {
            self.set_center(c);
        }
    }

    /// `findRandomSpawnPos(level, attempts)` — спиральный поиск
    /// точки спавна волны вокруг center (не в деревне при i>7,
    /// heightmap WORLD_SURFACE, |dy|<=96, entity-ticking).
    pub fn find_random_spawn_pos(
        &self,
        level: &dyn LevelReader,
        random: &mut dyn crate::random::RandomSource,
        attempts: i32,
    ) -> Option<BlockPos> {
        let i = self.raid_cooldown_ticks / 20;
        let f = 0.22 * i as f32 - 0.24;
        let f1 = random.next_float() * (std::f32::consts::PI * 2.0);
        for i1 in 0..attempts {
            let f2 = f1 + std::f32::consts::PI * i1 as f32 / 8.0;
            let i2 = self.center.x
                + crate::util::mth::floor_f32(f2.cos() * 32.0 * f)
                + random.next_int_bound(3) * crate::util::mth::floor_f32(f);
            let i3 = self.center.z
                + crate::util::mth::floor_f32(f2.sin() * 32.0 * f)
                + random.next_int_bound(3) * crate::util::mth::floor_f32(f);
            let height = level.get_height(crate::world::level::levelgen::heightmap::HeightmapType::WorldSurface, i2, i3);
            if crate::util::mth::abs_i32(height - self.center.y) <= 96 {
                let pos = BlockPos::new(i2, height, i3);
                if !level.is_village_at(pos) || i <= Self::ALLOW_SPAWNING_WITHIN_VILLAGE_SECONDS_THRESHOLD {
                    let i4 = 10;
                    if level.has_chunks_at_range(pos.x - i4, pos.z - i4, pos.x + i4, pos.z + i4)
                        && level.is_position_entity_ticking(pos)
                        && (level.ravager_spawn_position_ok(pos)
                            || (level
                                .get_block_state(pos.below())
                                .map(|s| s.block.id == crate::resources::identifier::Identifier::with_default_namespace("snow").unwrap())
                                .unwrap_or(false)
                                && level
                                    .get_block_state(pos)
                                    .map(|s| s.is_air())
                                    .unwrap_or(false)))
                    {
                        return Some(pos);
                    }
                }
            }
        }
        None
    }

    /// `updateRaiders(level)` — чистка рейдеров: removed/чужой
    /// dimension/далеко (>12544) → removeFromRaid; tickCount>600
    /// без entity в мире → remove; вне деревни noAction>2400 →
    /// ticksOutsideRaid++, >=30 → remove.
    pub fn update_raiders(&mut self, level: &dyn LevelReader) {
        let mut to_remove: Vec<(Uuid, f32, i32)> = Vec::new();
        for (_wave, set) in self.group_raider_map.iter() {
            for uuid in set {
                let Some(e) = level.entity_by_uuid(*uuid) else {
                    // getEntity(uuid)==null при tickCount>600 —
                    // здесь uuid не резолвится вообще → remove.
                    // Java проверяет только при tickCount>600; упрощённо:
                    // резолв не удался → удаляем (entity выгружена).
                    to_remove.push((*uuid, 0.0, 0));
                    continue;
                };
                let le = crate::world::entity::living_entity::LivingEntity::from_entity(e.clone());
                let bp = e.block_position();
                if e.is_removed()
                    || level.dimension_id() != Some(&e.dimension_id())
                    || self.center.dist_sqr(bp) >= Self::RAID_REMOVAL_THRESHOLD_SQR
                {
                    to_remove.push((*uuid, le.get_health(), *_wave));
                    continue;
                }
                if e.tick_count() > 600 {
                    if !level.is_village_at(bp) && le.get_no_action_time() > Self::MAX_NO_ACTION_TIME {
                        let outside = le.raider_ticks_outside_raid() + 1;
                        le_set_ticks_outside(level, *uuid, outside);
                        if outside >= Self::OUTSIDE_RAID_BOUNDS_TIMEOUT {
                            to_remove.push((*uuid, le.get_health(), *_wave));
                        }
                    }
                }
            }
        }
        for (uuid, health, wave) in to_remove {
            self.remove_from_raid(uuid, health, wave, true);
            if let Some(e) = level.entity_by_uuid(uuid) {
                let le = crate::world::entity::living_entity::LivingEntity::from_entity(e);
                if le.raider_is_patrol_leader() {
                    let w = le.raider_wave();
                    self.remove_leader(w);
                }
            }
        }
    }

    /// `spawnGroup(level, pos)` — спавн волны: по каждому
    /// RaiderType default+bonus спавны, leader-флаг первому
    /// canBeLeader, RAVAGER-усиление (pillager/evoker/vindicator
    /// пассажиром), groupsSpawned++.
    pub fn spawn_group(
        &mut self,
        level: &dyn LevelReader,
        pos: BlockPos,
        random: &mut dyn crate::random::RandomSource,
        difficulty: crate::world::difficulty::Difficulty,
    ) -> Option<i32> {
        let wave = self.groups_spawned + 1;
        self.total_health = 0.0;
        let should_spawn_bonus_group = self.should_spawn_bonus_group();
        let mut flag = false;
        for raider_type in RAIDER_TYPES {
            let i1 = Self::get_default_num_spawns(
                &raider_type,
                wave,
                self.num_groups,
                should_spawn_bonus_group,
            ) + Self::get_potential_bonus_spawns(
                &raider_type,
                random,
                wave,
                difficulty,
                should_spawn_bonus_group,
            );
            let mut i2 = 0;
            for _ in 0..i1 {
                // raiderType.entityType.create(level, EVENT)
                let Some(mut raider) = level.spawn_entity_of_type(
                    &Identifier::with_default_namespace(raider_type.entity_type).unwrap(),
                    pos,
                ) else {
                    break;
                };
                if !flag && raider.raider_can_be_leader() {
                    raider.raider_set_patrol_leader(true);
                    self.set_leader(wave, raider.entity.uuid);
                    flag = true;
                }
                // joinRaid(level, i, raider, pos, false)
                let health = raider.get_health();
                raider.raider_set_current_raid_id(self.raid_id());
                raider.raider_set_wave(wave);
                raider.raider_set_can_join_raid(true);
                raider.raider_set_ticks_outside_raid(0);
                // setPos + finalizeSpawn + applyRaidBuffs + addFreshEntity
                level.place_raider_at(&mut raider, pos, wave);
                self.add_wave_mob(wave, raider.entity.uuid, health, false);
                // RAVAGER → пассажир pillager/evoker/vindicator.
                if raider_type.entity_type == "ravager" {
                    let rider_type: Option<&'static str> = if wave
                        == Self::get_num_groups(crate::world::difficulty::Difficulty::Normal)
                    {
                        Some("pillager")
                    } else if wave
                        >= Self::get_num_groups(crate::world::difficulty::Difficulty::Hard)
                    {
                        if i2 == 0 {
                            Some("evoker")
                        } else {
                            Some("vindicator")
                        }
                    } else {
                        None
                    };
                    i2 += 1;
                    if let Some(rt) = rider_type {
                        if let Some(mut rider) = level.spawn_entity_of_type(
                            &Identifier::with_default_namespace(rt).unwrap(),
                            pos,
                        ) {
                            let rhealth = rider.get_health();
                            rider.raider_set_current_raid_id(self.raid_id());
                            rider.raider_set_wave(wave);
                            rider.raider_set_can_join_raid(true);
                            rider.raider_set_ticks_outside_raid(0);
                            level.place_raider_at(&mut rider, pos, wave);
                            self.add_wave_mob(wave, rider.entity.uuid, rhealth, false);
                            // startRiding(raider) — через level-хук.
                            level.make_passenger(&mut rider, raider.entity.uuid);
                        }
                    }
                }
            }
        }
        self.wave_spawn_pos = None;
        self.groups_spawned += 1;
        self.update_bossbar();
        self.raid_id_set();
        Some(wave)
    }

    /// `raidId` — для `raider_set_current_raid_id`.
    fn raid_id(&self) -> i32 {
        self.raid_id_value
    }

    /// Пометить id после add (вызывается Raids).
    fn raid_id_set(&mut self) {}

    /// `tick(level, random)` — полный тик рейда (см. Java tick()).
    /// `random` — level random для spawn-pos.
    pub fn tick(
        &mut self,
        level: &dyn LevelReader,
        random: &mut dyn crate::random::RandomSource,
    ) {
        if self.is_bar_dirty {
            self.update_bossbar_internal();
            self.is_bar_dirty = false;
        }
        if self.is_stopped() {
            return;
        }
        if self.status == RaidStatus::Ongoing {
            let flag = self.active;
            self.active = level.has_chunk_at(self.center);
            if level.difficulty() == crate::world::difficulty::Difficulty::Peaceful {
                // CraftBukkit RaidStopEvent PEACE — event-bus.
                self.stop();
                return;
            }
            if flag != self.active {
                // raidEvent.setVisible(active) — level-хук не нужен.
            }
            if !self.active {
                return;
            }
            if !level.is_village_at(self.center) {
                self.move_raid_center_to_nearby_village_section(level);
            }
            if !level.is_village_at(self.center) {
                if self.groups_spawned > 0 {
                    self.status = RaidStatus::Loss;
                } else {
                    self.stop();
                }
            }
            self.ticks_active += 1;
            if self.ticks_active >= Self::RAID_TIMEOUT_TICKS {
                self.stop();
                return;
            }
            let total_raiders_alive = self.get_total_raiders_alive();
            if total_raiders_alive == 0 && self.has_more_waves() {
                if self.raid_cooldown_ticks <= 0 {
                    if self.raid_cooldown_ticks == 0 && self.groups_spawned > 0 {
                        self.raid_cooldown_ticks = 300;
                        self.raid_event_name_suffix = None;
                        return;
                    }
                } else {
                    let is_present = self.wave_spawn_pos.is_some();
                    let mut flag1 = !is_present && self.raid_cooldown_ticks % 5 == 0;
                    if is_present
                        && !level.is_position_entity_ticking(self.wave_spawn_pos.unwrap())
                    {
                        flag1 = true;
                    }
                    if flag1 {
                        self.wave_spawn_pos = self.get_valid_spawn_pos(level, random);
                    }
                    if self.raid_cooldown_ticks == 300 || self.raid_cooldown_ticks % 20 == 0 {
                        // updatePlayers — raidEvent players sync (level-хук).
                    }
                    self.raid_cooldown_ticks -= 1;
                    self.raid_event_progress = crate::util::mth::clamp_f32(
                        (300 - self.raid_cooldown_ticks) as f32 / 300.0,
                        0.0,
                        1.0,
                    );
                }
            }
            if self.ticks_active % 20 == 0 {
                self.update_raiders(level);
                self.health_sum_cache = self.get_health_of_living_raiders(level);
                let alive = self.get_total_raiders_alive();
                self.raid_event_name_suffix = if alive > 0 && alive <= Self::LOW_MOB_THRESHOLD {
                    Some(format!("raiders_remaining:{}", alive))
                } else {
                    None
                };
            }
            let mut is_presentx = false;
            let mut i = 0;
            while self.should_spawn_group() {
                let pos = self
                    .wave_spawn_pos
                    .or_else(|| self.find_random_spawn_pos(level, random, 20));
                if let Some(pos) = pos {
                    self.started = true;
                    self.spawn_group(level, pos, random, level.difficulty());
                    if !is_presentx {
                        self.play_sound(level, pos, random);
                        is_presentx = true;
                    }
                } else {
                    i += 1;
                }
                if i > Self::NUM_SPAWN_ATTEMPTS {
                    self.stop();
                    break;
                }
            }
            if self.is_started() && !self.has_more_waves() && total_raiders_alive == 0 {
                if self.post_raid_ticks < Self::POST_RAID_TICK_LIMIT {
                    self.post_raid_ticks += 1;
                } else {
                    self.status = RaidStatus::Victory;
                    // heroesOfTheVillage → HERO_OF_THE_VILLAGE эффект +
                    // stats/criteria — через level-хук `raid_victory`.
                    for uuid in self.heroes_of_the_village.clone() {
                        if let Some(e) = level.entity_by_uuid(uuid) {
                            level.grant_hero_of_the_village(&e, self.raid_omen_level);
                        }
                    }
                }
            }
            self.update_bossbar();
        } else if self.is_over() {
            self.celebration_ticks += 1;
            if self.celebration_ticks >= Self::MAX_CELEBRATION_TICKS {
                self.stop();
                return;
            }
            if self.celebration_ticks % 20 == 0 {
                self.health_sum_cache = self.get_health_of_living_raiders(level);
                if self.is_victory() {
                    self.raid_event_progress = 0.0;
                }
            }
        }
    }

    /// `playSound(level, pos)` — RAID_HORN 13-блоковый радиус
    /// для игроков рейда через level-хук.
    fn play_sound(
        &self,
        level: &dyn LevelReader,
        pos: BlockPos,
        random: &mut dyn crate::random::RandomSource,
    ) {
        let random_long = random.next_long();
        level.play_raid_horn(pos, random_long);
    }
}

/// Хелпер: установка ticksOutsideRaid через level-резолв uuid
/// (mutating view).
fn le_set_ticks_outside(level: &dyn LevelReader, uuid: Uuid, value: i32) {
    if let Some(e) = level.entity_by_uuid(uuid) {
        let mut le = crate::world::entity::living_entity::LivingEntity::from_entity(e);
        le.raider_set_ticks_outside_raid(value);
    }
}
