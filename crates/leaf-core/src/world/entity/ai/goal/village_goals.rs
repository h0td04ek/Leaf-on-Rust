//! Перенос деревенских целей `net.minecraft.world.entity.ai.goal` —
//! `MoveThroughVillageGoal`, `MoveBackToVillageGoal`,
//! `StrollThroughVillageGoal`, `GolemRandomStrollInVillageGoal`.
//!
//! POI-резолвинг (`getPoiManager().find/getInRange`) — через
//! `LevelReader::village_poi_near`/`poi_in_section` хуки;
//! `sectionsToVillage`/`isCloseToVillage`/`findSectionClosestToVillage`
//! — через соответствующие LevelReader-методы.

use crate::core::block_pos::BlockPos;
use crate::core::section_pos::SectionPos;
use crate::world::entity::ai::goal::goal::{Goal, GoalFlag, GoalFlagSet};
use crate::world::entity::ai::goal::movement_look::RandomStrollGoal;
use crate::world::entity::ai::util::{default_random_pos, land_random_pos};
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::Mob;
use crate::world::level::pathfinder::path::Path;
use crate::world::phys::vec3::Vec3;

/// `MoveThroughVillageGoal` — голем/житель идёт к ближайшему
/// непосещённому village-POI через `createPath`; при двери на
/// пути пересчитывает путь до двери (canDealWithDoors).
pub struct MoveThroughVillageGoal {
    /// `speedModifier`.
    pub speed_modifier: f64,
    /// `path`.
    pub path: Option<Path>,
    /// `poiPos`.
    pub poi_pos: Option<BlockPos>,
    /// `onlyAtNight`.
    pub only_at_day_night: bool,
    /// `visited` — последние ≤16 POI (как `Lists.newArrayList`+trim15).
    pub visited: Vec<BlockPos>,
    /// `distanceToPoi`.
    pub distance_to_poi: i32,
    /// `canDealWithDoors`.
    pub can_deal_with_doors: bool,
}

impl MoveThroughVillageGoal {
    /// `new(mob, speedModifier, onlyAtNight, distanceToPoi,
    /// canDealWithDoors)` — MOVE-флаг; требует ground-навигацию
    /// (`GoalUtils.hasGroundPathNavigation`).
    pub fn new(
        speed_modifier: f64,
        only_at_night: bool,
        distance_to_poi: i32,
        can_deal_with_doors: bool,
    ) -> Self {
        Self {
            speed_modifier,
            path: None,
            poi_pos: None,
            only_at_day_night: only_at_night,
            visited: Vec::new(),
            distance_to_poi,
            can_deal_with_doors,
        }
    }

    /// `hasNotVisited(pos)` — pos не в `visited`.
    fn has_not_visited(&self, pos: BlockPos) -> bool {
        !self.visited.contains(&pos)
    }

    /// `updateVisited()` — храним ≤16 (Java: size>15 → remove(0)).
    fn update_visited(&mut self) {
        if self.visited.len() > 15 {
            self.visited.remove(0);
        }
    }
}

impl Goal for MoveThroughVillageGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let Some(level) = mob.level() else {
            return false;
        };
        if !crate::world::entity::ai::util::goal_utils::has_ground_path_navigation(mob) {
            return false;
        }
        self.update_visited();
        if self.only_at_day_night && level.is_bright_outside() {
            return false;
        }
        let block_pos = mob.living.entity.block_position();
        if !level.is_close_to_village(block_pos, 6) {
            return false;
        }
        // LandRandomPos.getPos(mob, 15, 7, scorer) — scorer:
        // !isVillage → -INF; иначе -distSqr к ближайшему
        // непосещённому VILLAGE-POI (range 10 от кандидата).
        let visited = self.visited.clone();
        let scorer_mob_pos = block_pos;
        let pos = land_random_pos::get_pos_scored(mob, 15, 7, move |m: &Mob, candidate| {
            let lvl = match m.level() {
                Some(l) => l,
                None => return f64::NEG_INFINITY,
            };
            if !lvl.is_village_at(candidate) {
                return f64::NEG_INFINITY;
            }
            let poi = lvl.village_poi_near(candidate, 10, &visited);
            match poi {
                Some(p) => {
                    let dx = (p.x - scorer_mob_pos.x) as f64;
                    let dy = (p.y - scorer_mob_pos.y) as f64;
                    let dz = (p.z - scorer_mob_pos.z) as f64;
                    -(dx * dx + dy * dy + dz * dz)
                }
                None => f64::NEG_INFINITY,
            }
        });
        let Some(pos) = pos else {
            return false;
        };
        let containing = BlockPos::containing(pos.x, pos.y, pos.z);
        let Some(level2) = mob.level() else {
            return false;
        };
        let visited2 = self.visited.clone();
        let Some(poi) = level2.village_poi_near(containing, 10, &visited2) else {
            return false;
        };
        self.poi_pos = Some(poi);
        let nav = mob.get_navigation();
        let Some(nav) = nav else {
            return false;
        };
        {
            let mut n = nav.borrow_mut();
            n.set_can_open_doors(self.can_deal_with_doors);
            self.path = n.create_path_pos(mob, poi, 0);
            n.set_can_open_doors(true);
        }
        if self.path.is_none() {
            let pos_towards = default_random_pos::get_pos_towards(
                mob,
                10,
                7,
                Vec3::new(poi.x as f64 + 0.5, poi.y as f64, poi.z as f64 + 0.5),
                std::f32::consts::FRAC_PI_2 as f64,
            );
            let Some(pos_towards) = pos_towards else {
                return false;
            };
            let mut n = nav.borrow_mut();
            n.set_can_open_doors(self.can_deal_with_doors);
            self.path = n.create_path(mob, pos_towards.x, pos_towards.y, pos_towards.z, 0);
            n.set_can_open_doors(true);
            if self.path.is_none() {
                return false;
            }
        }
        // Door-scan: первая деревянная дверь на пути →
        // createPath до её ноды.
        if let Some(path) = &self.path {
            for i in 0..path.node_count() {
                let node = path.nodes[i].borrow();
                let door_pos = BlockPos::new(node.x, node.y + 1, node.z);
                drop(node);
                if level.is_wooden_door(door_pos) {
                    let mut n = nav.borrow_mut();
                    self.path = n.create_path(mob, door_pos.x as f64, door_pos.y as f64, door_pos.z as f64, 0);
                    break;
                }
            }
        }
        self.path.is_some()
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        let Some(poi) = self.poi_pos else {
            return false;
        };
        let nav_done = mob
            .get_navigation()
            .as_ref()
            .map(|n| n.borrow().is_done())
            .unwrap_or(true);
        !nav_done
            && !poi.closer_to_center_than(
                mob.living.entity.position(),
                mob.living.get_bb_width() as f64 + self.distance_to_poi as f64,
            )
    }

    fn start(&mut self, mob: &mut Mob) {
        if let (Some(nav), Some(path)) = (mob.get_navigation(), self.path.take()) {
            nav.borrow_mut().move_to_path(mob, Some(path), self.speed_modifier);
        }
    }

    fn stop(&mut self, mob: &mut Mob) {
        let nav_done = mob
            .get_navigation()
            .as_ref()
            .map(|n| n.borrow().is_done())
            .unwrap_or(true);
        if let Some(poi) = self.poi_pos {
            if nav_done
                || poi.closer_to_center_than(
                    mob.living.entity.position(),
                    self.distance_to_poi as f64,
                )
            {
                self.visited.push(poi);
            }
        }
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move].into_iter().collect()
    }
}

/// `MoveBackToVillageGoal` — `RandomStrollGoal`-вариант: только
/// вне деревни; цель — случайная точка по направлению к
/// ближайшей village-секции (radius 2).
pub struct MoveBackToVillageGoal {
    /// Встроенный `RandomStrollGoal(mob, speed, 10, checkNoAction)`.
    pub inner: RandomStrollGoal,
}

/// `MAX_XZ_DIST`/`MAX_Y_DIST` — радиус getPosTowards.
const MAX_XZ_DIST: i32 = 10;
const MAX_Y_DIST: i32 = 7;

impl MoveBackToVillageGoal {
    /// `new(mob, speedModifier, checkNoActionTime)` —
    /// `super(mob, speed, 10, checkNoAction)`.
    pub fn new(speed_modifier: f64, check_no_action_time: bool) -> Self {
        Self {
            inner: RandomStrollGoal::full(speed_modifier, 10, check_no_action_time),
        }
    }
}

impl MoveBackToVillageGoal {
    /// Java `getPosition()` — цель по направлению к ближайшей
    /// village-секции (radius 2 от секции моба).
    fn get_position(&self, mob: &Mob) -> Option<Vec3> {
        let level = mob.level()?;
        let section = SectionPos::of(&mob.living.entity.block_position());
        let closest = level.find_section_closest_to_village(section, 2)?;
        if closest == section {
            return None;
        }
        let center = closest.center();
        default_random_pos::get_pos_towards(
            mob,
            MAX_XZ_DIST,
            MAX_Y_DIST,
            Vec3::new(center.x as f64 + 0.5, center.y as f64, center.z as f64 + 0.5),
            std::f32::consts::FRAC_PI_2 as f64,
        )
    }
}

impl Goal for MoveBackToVillageGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        // Java: `!isVillage(blockPos) && super.canUse()` — где
        // super.canUse использует переопределённый getPosition.
        let in_village = mob
            .level()
            .map(|l| l.is_village_at(mob.living.entity.block_position()))
            .unwrap_or(false);
        if in_village || mob.has_controlling_passenger() {
            return false;
        }
        if !self.inner.force_trigger {
            if self.inner.check_no_action_time && mob.get_no_action_time() >= 100 {
                return false;
            }
            let delay = crate::util::mth::positive_ceil_div(self.inner.interval, 2);
            if mob.random.borrow_mut().next_int_bound(delay) != 0 {
                return false;
            }
        }
        let Some(position) = self.get_position(mob) else {
            return false;
        };
        self.inner.wanted_x = position.x;
        self.inner.wanted_y = position.y;
        self.inner.wanted_z = position.z;
        self.inner.force_trigger = false;
        true
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        self.inner.can_continue_to_use(mob)
    }

    fn start(&mut self, mob: &mut Mob) {
        self.inner.start(mob);
    }

    fn stop(&mut self, mob: &mut Mob) {
        self.inner.stop(mob);
    }

    fn get_flags(&self) -> GoalFlagSet {
        self.inner.get_flags()
    }

    fn requires_update_every_tick(&self) -> bool {
        self.inner.requires_update_every_tick()
    }
}

/// `StrollThroughVillageGoal` — ночная прогулка внутри деревни:
/// `LandRandomPos.getPos(15,7, scorer=-sectionsToVillage)` →
/// wantedPos; tick — подкрадывается к wantedPos через heightmap
/// MOTION_BLOCKING_NO_LEAVES.
pub struct StrollThroughVillageGoal {
    /// `interval` (reducedTickDelay).
    pub interval: i32,
    /// `wantedPos`.
    pub wanted_pos: Option<BlockPos>,
}

/// `DISTANCE_THRESHOLD`.
const DISTANCE_THRESHOLD: f64 = 10.0;

impl StrollThroughVillageGoal {
    /// `new(mob, interval)` — MOVE-флаг.
    pub fn new(interval: i32) -> Self {
        Self {
            interval: Self::reduced_tick_delay_static(interval),
            wanted_pos: None,
        }
    }

    /// `moveRandomly()` — случайная точка в 16×16 вокруг моба
    /// на heightmap MOTION_BLOCKING_NO_LEAVES.
    fn move_randomly(&self, mob: &mut Mob) {
        let Some(level) = mob.level() else {
            return;
        };
        let base = mob.living.entity.block_position();
        let mut r = mob.random.borrow_mut();
        let offset = BlockPos::new(
            base.x - 8 + r.next_int_bound(16),
            base.y,
            base.z - 8 + r.next_int_bound(16),
        );
        drop(r);
        let heightmap_pos = level.get_heightmap_pos(
            crate::world::level::levelgen::heightmap::HeightmapType::MotionBlockingNoLeaves,
            offset,
        );
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().move_to_pos(
                mob,
                heightmap_pos.x as f64,
                heightmap_pos.y as f64,
                heightmap_pos.z as f64,
                1.0,
            );
        }
    }
}

impl Goal for StrollThroughVillageGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if mob.has_controlling_passenger() {
            return false;
        }
        let Some(level) = mob.level() else {
            return false;
        };
        if level.is_bright_outside() {
            return false;
        }
        if mob.random.borrow_mut().next_int_bound(self.interval) != 0 {
            return false;
        }
        let block_pos = mob.living.entity.block_position();
        if !level.is_close_to_village(block_pos, 6) {
            return false;
        }
        // scorer = -sectionsToVillage(SectionPos.of(candidate)).
        let pos = land_random_pos::get_pos_scored(mob, 15, 7, |m: &Mob, candidate| {
            m.level()
                .map(|l| {
                    -(l.sections_to_village(SectionPos::of(&candidate)) as f64)
                })
                .unwrap_or(f64::NEG_INFINITY)
        });
        self.wanted_pos = pos.map(|p| BlockPos::containing(p.x, p.y, p.z));
        self.wanted_pos.is_some()
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        let Some(wanted) = self.wanted_pos else {
            return false;
        };
        let nav = mob.get_navigation();
        let Some(nav) = nav else {
            return false;
        };
        let n = nav.borrow();
        !n.is_done()
            && n.target_pos() == Some(wanted)
    }

    fn tick(&mut self, mob: &mut Mob) {
        let Some(wanted) = self.wanted_pos else {
            return;
        };
        let nav = mob.get_navigation();
        let Some(nav) = nav else {
            return;
        };
        let nav_done = nav.borrow().is_done();
        if !nav_done || wanted.closer_to_center_than(mob.living.entity.position(), DISTANCE_THRESHOLD) {
            return;
        }
        // vec3 = atBottomCenterOf(wanted); vec32 = pos - vec3;
        // vec3 += vec32.scale(0.4); vec33 = vec3-vec31 normalize*10+vec31;
        let w = Vec3::new(wanted.x as f64 + 0.5, wanted.y as f64, wanted.z as f64 + 0.5);
        let p = mob.living.entity.position();
        let diff = p.subtract_vec(&w);
        let adjusted = w.add_vec(&diff.scale(0.4));
        let toward = adjusted.subtract_vec(&p);
        let len = toward.length();
        if len <= 0.0 {
            return;
        }
        let target = p.add_vec(&toward.scale(DISTANCE_THRESHOLD / len));
        let mut bp = BlockPos::containing(target.x, target.y, target.z);
        if let Some(level) = mob.level() {
            bp = level.get_heightmap_pos(
                crate::world::level::levelgen::heightmap::HeightmapType::MotionBlockingNoLeaves,
                bp,
            );
        }
        let moved = {
            let mut n = nav.borrow_mut();
            n.move_to_pos(mob, bp.x as f64, bp.y as f64, bp.z as f64, 1.0)
        };
        if !moved {
            self.move_randomly(mob);
        }
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move].into_iter().collect()
    }
}

/// `GolemRandomStrollInVillageGoal` — голем гуляет по деревне:
/// 30% — куда угодно; иначе 70/30 villager-who-wants-golem vs
/// village-POI (по убыванию). `RandomStrollGoal(mob, speed, 240,
/// checkNoAction=false)`.
pub struct GolemRandomStrollInVillageGoal {
    /// Встроенный `RandomStrollGoal`.
    pub inner: RandomStrollGoal,
}

impl GolemRandomStrollInVillageGoal {
    /// `POI_SECTION_SCAN_RADIUS`/`VILLAGER_SCAN_RADIUS`/
    /// `RANDOM_POS_*` константы Java.
    pub const POI_SECTION_SCAN_RADIUS: i32 = 2;
    pub const VILLAGER_SCAN_RADIUS: i32 = 32;
    pub const RANDOM_POS_XY_DISTANCE: i32 = 10;
    pub const RANDOM_POS_Y_DISTANCE: i32 = 7;

    /// `new(mob, speedModifier)` — interval=240, checkNoAction=false.
    pub fn new(speed_modifier: f64) -> Self {
        Self {
            inner: RandomStrollGoal::full(speed_modifier, 240, false),
        }
    }

    /// `getPositionTowardsAnywhere()`.
    fn position_towards_anywhere(&self, mob: &Mob) -> Option<Vec3> {
        land_random_pos::get_pos(mob, Self::RANDOM_POS_XY_DISTANCE, Self::RANDOM_POS_Y_DISTANCE)
    }

    /// `getPositionTowardsVillagerWhoWantsGolem()` — случайный
    /// villager в 32 блоках с `wantsToSpawnGolem(gameTime)`.
    fn position_towards_villager_who_wants_golem(&self, mob: &Mob) -> Option<Vec3> {
        let level = mob.level()?;
        let game_time = level.game_time();
        let box_ = mob.living.entity.get_bounding_box().inflate(32.0, 32.0, 32.0);
        let villager_id = crate::resources::identifier::Identifier::with_default_namespace("villager").ok()?;
        let villagers: Vec<LivingEntity> = level
            .entities_in_box(&box_)
            .into_iter()
            .filter(|e| e.is_living_entity())
            .filter(|e| {
                crate::world::entity::entity_type::holder_type_id(&e.entity_type)
                    .map(|id| id == villager_id)
                    .unwrap_or(false)
            })
            .map(LivingEntity::from_entity)
            .filter(|le| le.villager_wants_to_spawn_golem(game_time))
            .collect();
        if villagers.is_empty() {
            return None;
        }
        let idx = {
            let mut r = mob.random.borrow_mut();
            r.next_int_bound(villagers.len() as i32) as usize
        };
        let villager = villagers.get(idx)?.clone();
        let pos = villager.position();
        land_random_pos::get_pos_towards(
            mob,
            Self::RANDOM_POS_XY_DISTANCE,
            Self::RANDOM_POS_Y_DISTANCE,
            pos,
        )
    }

    /// `getRandomVillageSection()` — случайная village-секция
    /// в радиусе 2 секций от моба.
    fn random_village_section(&self, mob: &Mob) -> Option<SectionPos> {
        let level = mob.level()?;
        let center = SectionPos::of(&mob.living.entity.block_position());
        let mut list = Vec::new();
        for dx in -Self::POI_SECTION_SCAN_RADIUS..=Self::POI_SECTION_SCAN_RADIUS {
            for dz in -Self::POI_SECTION_SCAN_RADIUS..=Self::POI_SECTION_SCAN_RADIUS {
                let sp = SectionPos::of_i(center.x + dx, center.y, center.z + dz);
                if level.sections_to_village(sp) == 0 {
                    list.push(sp);
                }
            }
        }
        if list.is_empty() {
            return None;
        }
        let idx = {
            let mut r = mob.random.borrow_mut();
            r.next_int_bound(list.len() as i32) as usize
        };
        list.get(idx).copied()
    }

    /// `getRandomPoiWithinSection(section)` — случайный POI
    /// (radius 8 от section center) через level-хук.
    fn random_poi_within_section(&self, mob: &Mob, section: SectionPos) -> Option<BlockPos> {
        let level = mob.level()?;
        let center = section.center();
        let pois = level.poi_in_section(center, 8)?;
        if pois.is_empty() {
            return None;
        }
        let idx = {
            let mut r = mob.random.borrow_mut();
            r.next_int_bound(pois.len() as i32) as usize
        };
        pois.get(idx).copied()
    }

    /// `getPositionTowardsPoi()`.
    fn position_towards_poi(&self, mob: &Mob) -> Option<Vec3> {
        let section = self.random_village_section(mob)?;
        let poi = self.random_poi_within_section(mob, section)?;
        land_random_pos::get_pos_towards(
            mob,
            Self::RANDOM_POS_XY_DISTANCE,
            Self::RANDOM_POS_Y_DISTANCE,
            Vec3::new(poi.x as f64 + 0.5, poi.y as f64, poi.z as f64 + 0.5),
        )
    }

    /// Java `getPosition()` — выбор ветки по randomFloat.
    pub fn get_position(&self, mob: &mut Mob) -> Option<Vec3> {
        let random_float = {
            let mut r = mob.random.borrow_mut();
            r.next_float()
        };
        if mob.random.borrow_mut().next_float() < 0.3 {
            return self.position_towards_anywhere(mob);
        }
        let towards_villager = || self.position_towards_villager_who_wants_golem(mob);
        let towards_poi = || self.position_towards_poi(mob);
        let result = if random_float < 0.7 {
            towards_villager().or_else(|| towards_poi())
        } else {
            towards_poi().or_else(|| towards_villager())
        };
        result.or_else(|| self.position_towards_anywhere(mob))
    }
}

impl Goal for GolemRandomStrollInVillageGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        // Повторяем RandomStrollGoal.canUse с нашим getPosition.
        if mob.has_controlling_passenger() {
            return false;
        }
        if !self.inner.force_trigger {
            if self.inner.check_no_action_time && mob.get_no_action_time() >= 100 {
                return false;
            }
            let delay = crate::util::mth::positive_ceil_div(self.inner.interval, 2);
            if mob.random.borrow_mut().next_int_bound(delay) != 0 {
                return false;
            }
        }
        let Some(position) = self.get_position(mob) else {
            return false;
        };
        self.inner.wanted_x = position.x;
        self.inner.wanted_y = position.y;
        self.inner.wanted_z = position.z;
        self.inner.force_trigger = false;
        true
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        self.inner.can_continue_to_use(mob)
    }

    fn start(&mut self, mob: &mut Mob) {
        self.inner.start(mob);
    }

    fn stop(&mut self, mob: &mut Mob) {
        self.inner.stop(mob);
    }

    fn get_flags(&self) -> GoalFlagSet {
        self.inner.get_flags()
    }

    fn requires_update_every_tick(&self) -> bool {
        self.inner.requires_update_every_tick()
    }
}
