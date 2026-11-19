use super::*;

/// `Raids` — saved-data менеджер рейдов уровня
/// (`Int2ObjectMap<Raid> raidMap`, `nextId`, `tick`,
/// Purpur `playerCooldowns`).
pub struct Raids {
    /// `raidMap` — id → Raid (Rc<RefCell> для разделяемого доступа
    /// из goal'ов и level).
    pub raid_map: HashMap<i32, Rc<RefCell<Raid>>>,
    /// `nextId`.
    pub next_id: i32,
    /// `tick`.
    pub tick: i32,
    /// Purpur `playerCooldowns` — uuid → секунд до следующего рейда.
    pub player_cooldowns: HashMap<Uuid, i32>,
    /// raid cooldown seconds (Purpur config; 0 = выкл).
    pub raid_cooldown_seconds: i32,
}

impl Raids {
    /// `new()` — пустой менеджер.
    pub fn new() -> Self {
        Self {
            raid_map: HashMap::new(),
            next_id: 1,
            tick: 0,
            player_cooldowns: HashMap::new(),
            raid_cooldown_seconds: 0,
        }
    }

    /// `get(id)`.
    pub fn get(&self, id: i32) -> Option<Rc<RefCell<Raid>>> {
        self.raid_map.get(&id).cloned()
    }

    /// `getId(raid)` — поиск id по ptr_eq.
    pub fn get_id(&self, raid: &Rc<RefCell<Raid>>) -> Option<i32> {
        self.raid_map
            .iter()
            .find(|(_, v)| Rc::ptr_eq(v, raid))
            .map(|(k, _)| *k)
    }

    /// `tick(level, random)` — tick всех рейдов + Purpur cooldowns.
    pub fn tick(
        &mut self,
        level: &dyn LevelReader,
        random: &mut dyn crate::random::RandomSource,
    ) {
        self.tick += 1;
        if self.raid_cooldown_seconds != 0 && self.tick % 20 == 0 {
            let keys: Vec<Uuid> = self.player_cooldowns.keys().copied().collect();
            for uuid in keys {
                let i = self.player_cooldowns.get(&uuid).copied().unwrap_or(0);
                if i < 1 {
                    self.player_cooldowns.remove(&uuid);
                } else {
                    self.player_cooldowns.insert(uuid, i - 1);
                }
            }
        }
        let raids_enabled = level.raids_enabled();
        let ids: Vec<i32> = self.raid_map.keys().copied().collect();
        for id in ids {
            let Some(raid_rc) = self.raid_map.get(&id).cloned() else {
                continue;
            };
            {
                let mut raid = raid_rc.borrow_mut();
                if !raids_enabled {
                    raid.stop();
                }
                if raid.is_stopped() {
                    continue;
                }
                raid.tick(level, random);
            }
            if self
                .raid_map
                .get(&id)
                .map(|r| r.borrow().is_stopped())
                .unwrap_or(false)
            {
                self.raid_map.remove(&id);
            }
        }
        if self.tick % 200 == 0 {
            // setDirty — saved-data флаг (уровень пометит).
        }
    }

    /// `canJoinRaid(raider)` — жив, canJoinRaid, noActionTime<=2400.
    pub fn can_join_raid(raider: &crate::world::entity::living_entity::LivingEntity) -> bool {
        raider.is_alive() && raider.raider_can_join_raid() && raider.get_no_action_time() <= Raid::MAX_NO_ACTION_TIME
    }

    /// `getNearbyRaid(pos, distance)` — ближайший активный рейд.
    pub fn get_nearby_raid(&self, pos: BlockPos, distance: i32) -> Option<Rc<RefCell<Raid>>> {
        let mut raid: Option<Rc<RefCell<Raid>>> = None;
        let mut d = distance as f64;
        for r in self.raid_map.values() {
            let center = r.borrow().get_center();
            let d1 = center.dist_sqr(pos);
            if r.borrow().is_active() && d1 < d {
                raid = Some(r.clone());
                d = d1;
            }
        }
        raid
    }

    /// `getRaidCentersInChunk(chunkPos)` — центры рейдов в чанке.
    pub fn get_raid_centers_in_chunk(&self, chunk_pos: &crate::world::level::chunk_pos::ChunkPos) -> Vec<BlockPos> {
        self.raid_map
            .values()
            .map(|r| r.borrow().get_center())
            .filter(|pos| {
                pos.x >> 4 == chunk_pos.x && pos.z >> 4 == chunk_pos.z
            })
            .collect()
    }

    /// `getOrCreateRaid(level, pos)` — рейд в pos или новый.
    fn get_or_create_raid(
        &mut self,
        level: &dyn LevelReader,
        pos: BlockPos,
    ) -> Rc<RefCell<Raid>> {
        if let Some(raid) = level.raid_at(pos) {
            return raid;
        }
        let difficulty = level.difficulty();
        let raid = Rc::new(RefCell::new(Raid::new(pos, difficulty)));
        raid.borrow_mut().raid_id_value = self.next_id;
        self.next_id += 1;
        let id = raid.borrow().raid_id_value;
        self.raid_map.insert(id, raid.clone());
        raid
    }

    /// `createOrExtendRaid(player, pos)` — Bad Omen → рейд:
    /// village-POI центризация, absorbRaidOmen, дедупликация.
    /// `player_uuid`, `player_omen_level`, `player_is_spectator`.
    pub fn create_or_extend_raid(
        &mut self,
        level: &dyn LevelReader,
        player_uuid: Uuid,
        player_omen_level: i32,
        player_is_spectator: bool,
        pos: BlockPos,
    ) -> Option<Rc<RefCell<Raid>>> {
        if player_is_spectator || !level.raids_enabled() {
            return None;
        }
        if !level.environment_attribute_can_start_raid(pos) {
            return None;
        }
        // POI village-центризация (getPoiManager VILLAGE, 64).
        let center = level.village_center_near(pos).unwrap_or(pos);
        let raid = self.get_or_create_raid(level, center);
        let (started, in_progress, omen_level, max_omen) = {
            let r = raid.borrow();
            (r.is_started(), r.is_in_progress(), r.get_raid_omen_level(), r.get_max_raid_omen_level())
        };
        if !started || (in_progress && omen_level < max_omen) {
            if self.raid_cooldown_seconds != 0
                && self.player_cooldowns.contains_key(&player_uuid)
            {
                return None;
            }
            // CraftBukkit callRaidTriggerEvent — event-bus уровень.
            {
                let mut r = raid.borrow_mut();
                r.absorb_raid_omen(player_omen_level);
            }
            if self.raid_cooldown_seconds != 0 {
                self.player_cooldowns.insert(player_uuid, self.raid_cooldown_seconds);
            }
        }
        Some(raid)
    }
}

impl Default for Raids {
    fn default() -> Self {
        Self::new()
    }
}
