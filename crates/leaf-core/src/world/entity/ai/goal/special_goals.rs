//! Перенос `TemptGoal` (+`ForNonPathfinders`) и `MoveToBlockGoal`
//! из `net.minecraft.world.entity.ai.goal`.

use crate::core::block_pos::BlockPos;
use crate::entity::ai::attributes::attributes as attribute_ids;
use crate::world::entity::ai::goal::goal::{Goal, GoalFlag, GoalFlagSet};
use crate::world::entity::ai::targeting::targeting_conditions::TargetingConditions;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::Mob;
use crate::world::item::item_stack::ItemStack;
use crate::world::level::level_reader::LevelReader;
use std::sync::Arc;

/// `TEMPT_TARGETING` — `forNonCombat().ignoreLineOfSight()`.
fn tempt_targeting() -> TargetingConditions {
    let mut t = TargetingConditions::for_non_combat();
    t.ignore_line_of_sight();
    t
}

/// `TemptGoal` — моб следует за игроком сtempt-предметом
/// (мясо для волков, семена для кур и т.д.).
pub struct TemptGoal {
    targeting_conditions: TargetingConditions,
    speed_modifier: f64,
    px: f64,
    py: f64,
    pz: f64,
    p_rot_x: f64,
    p_rot_y: f64,
    /// `player` — цели-провокаторы (CraftBukkit-поле; у нас
    /// `Option<LivingEntity>`).
    pub(crate) player: Option<LivingEntity>,
    calm_down: i32,
    is_running: bool,
    items: Arc<dyn Fn(&ItemStack) -> bool + Send + Sync>,
    can_scare: bool,
    stop_distance: f64,
}

impl TemptGoal {
    /// `new(mob, speedModifier, items, canScare)` — stopDistance 2.5.
    pub fn new(
        _mob: &Mob,
        speed_modifier: f64,
        items: impl Fn(&ItemStack) -> bool + Send + Sync + 'static,
        can_scare: bool,
    ) -> Self {
        Self::with_stop_distance(_mob, speed_modifier, items, can_scare, 2.5)
    }

    /// `new(mob, speedModifier, items, canScare, stopDistance)`.
    pub fn with_stop_distance(
        _mob: &Mob,
        speed_modifier: f64,
        items: impl Fn(&ItemStack) -> bool + Send + Sync + 'static,
        can_scare: bool,
        stop_distance: f64,
    ) -> Self {
        // Paper `GlobalTemptationLookup` — оптимизация поиска;
        // у нас selector-ветка (`shouldFollow`), эквивалентная
        // `globalTemptationLookupIndex == -1`.
        let items_check: Arc<dyn Fn(&ItemStack) -> bool + Send + Sync> = Arc::new(items);
        let mut conditions = tempt_targeting();
        let items_sel = items_check.clone();
        conditions.selector(Arc::new(move |le: &LivingEntity, _level: &dyn LevelReader| {
            // `shouldFollow`: предмет в основной или второй руке.
            // (Villager-sleeping проверка — у нас нет Villager-API.)
            items_sel(le.get_main_hand_item()) || items_sel(le.get_offhand_item())
        }));
        Self {
            targeting_conditions: conditions,
            speed_modifier,
            px: 0.0,
            py: 0.0,
            pz: 0.0,
            p_rot_x: 0.0,
            p_rot_y: 0.0,
            player: None,
            calm_down: 0,
            is_running: false,
            items: items_check,
            can_scare,
            stop_distance,
        }
    }

    /// `shouldFollow(entity)` — предмет в основной или второй
    /// руке (без Villager-проверки — Villager ещё не портирован).
    pub fn should_follow(&self, entity: &LivingEntity) -> bool {
        (self.items)(entity.get_main_hand_item())
            || (self.items)(entity.get_offhand_item())
    }

    /// `canScare()`.
    pub fn can_scare(&self) -> bool {
        self.can_scare
    }

    /// `isRunning()`.
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// `stopNavigation()` — останавливает навигацию.
    pub fn stop_navigation(&self, mob: &mut Mob) {
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().stop(mob);
        }
    }

    /// `navigateTowards(player)` — движение к цели.
    pub fn navigate_towards(&self, mob: &mut Mob, player: &LivingEntity) {
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().move_to_entity(mob, player, self.speed_modifier);
        }
    }

    /// `getNearestPlayer(targetingConditions, mob)` — ближайший
    /// игрок, проходящий условия (скан `entities_in_box` +
    /// `TargetingConditions.test`, min distance — как
    /// `ServerLevel.getNearestPlayer`).
    fn nearest_player_matching(
        &self,
        mob: &mut Mob,
        conditions: &TargetingConditions,
    ) -> Option<LivingEntity> {
        let level = mob.level()?;
        let range = conditions.range_value();
        let box_ = mob.living.entity.bounding_box.inflate(range, range, range);
        let mut best: Option<LivingEntity> = None;
        let mut best_d = f64::MAX;
        for e in level.entities_in_box(&box_) {
            if !e.is_player() || !e.is_living_entity() {
                continue;
            }
            let le = LivingEntity::from_entity(e);
            if !le.entity.is_alive() {
                continue;
            }
            let d = le.entity.distance_to_sqr(&mob.living.entity);
            if d >= best_d {
                continue;
            }
            if !conditions.test(&level, Some(&mut *mob), &le) {
                continue;
            }
            best_d = d;
            best = Some(le);
        }
        best
    }
}

impl Goal for TemptGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if self.calm_down > 0 {
            self.calm_down -= 1;
            return false;
        }
        let tempt_range = mob
            .living
            .get_attribute_value_by_id_ro(&attribute_ids::tempt_range());
        let mut conditions = self.targeting_conditions.copy();
        conditions.range(tempt_range);
        self.player = self.nearest_player_matching(mob, &conditions);
        self.player.is_some()
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        if self.can_scare {
            let Some(player) = &self.player else { return false };
            if mob.living.entity.distance_to_sqr(&player.entity) < 36.0 {
                if player
                    .entity
                    .distance_to_sqr_pos(self.px, self.py, self.pz)
                    > 0.010000000000000002
                {
                    return false;
                }
                if (player.entity.get_x_rot() as f64 - self.p_rot_x).abs() > 5.0
                    || (player.entity.get_y_rot() as f64 - self.p_rot_y).abs() > 5.0
                {
                    return false;
                }
            } else {
                self.px = player.entity.get_x();
                self.py = player.entity.get_y();
                self.pz = player.entity.get_z();
            }
            self.p_rot_x = player.entity.get_x_rot() as f64;
            self.p_rot_y = player.entity.get_y_rot() as f64;
        }
        self.can_use(mob)
    }

    fn start(&mut self, mob: &mut Mob) {
        if let Some(player) = &self.player {
            self.px = player.entity.get_x();
            self.py = player.entity.get_y();
            self.pz = player.entity.get_z();
        }
        self.is_running = true;
    }

    fn stop(&mut self, mob: &mut Mob) {
        self.player = None;
        self.stop_navigation(mob);
        self.calm_down = Self::reduced_tick_delay_static(100);
        self.is_running = false;
    }

    fn tick(&mut self, mob: &mut Mob) {
        let Some(player) = self.player.clone() else { return };
        mob.look_control.set_look_at_entity_deltas(
            &player.entity,
            mob.get_max_head_y_rot() + 20.0,
            mob.get_max_head_x_rot(),
        );
        let d = mob.living.entity.distance_to_sqr(&player.entity);
        if d < self.stop_distance * self.stop_distance {
            self.stop_navigation(mob);
        } else {
            self.navigate_towards(mob, &player);
        }
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move, GoalFlag::Look].into_iter().collect()
    }
}

/// `TemptGoal.ForNonPathfinders` — для мобов без pathfinding:
/// движение через `MoveControl.setWantedPosition` к точке
/// перед глазами игрока.
pub struct TemptGoalForNonPathfinders {
    inner: TemptGoal,
}

impl TemptGoalForNonPathfinders {
    /// `new(mob, speedModifier, items, canScare, stopDistance)`.
    pub fn new(
        mob: &Mob,
        speed_modifier: f64,
        items: impl Fn(&ItemStack) -> bool + Send + Sync + 'static,
        can_scare: bool,
        stop_distance: f64,
    ) -> Self {
        Self {
            inner: TemptGoal::with_stop_distance(
                mob,
                speed_modifier,
                items,
                can_scare,
                stop_distance,
            ),
        }
    }
}

impl Goal for TemptGoalForNonPathfinders {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        self.inner.can_use(mob)
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

    fn tick(&mut self, mob: &mut Mob) {
        let Some(player) = self.inner.player.clone() else {
            return;
        };
        mob.look_control.set_look_at_entity_deltas(
            &player.entity,
            mob.get_max_head_y_rot() + 20.0,
            mob.get_max_head_x_rot(),
        );
        let d = mob.living.entity.distance_to_sqr(&player.entity);
        if d < self.inner.stop_distance * self.inner.stop_distance {
            // `stopNavigation` → `moveControl.setWait()`.
            mob.move_control.set_wait();
        } else {
            // `navigateTowards`: точка между mob и глазом игрока
            // со случайным множителем.
            let eye = player.entity.get_eye_position();
            let mob_pos = mob.living.entity.position();
            let delta = eye.subtract_vec(&mob_pos);
            let t = mob.random.borrow_mut().next_double();
            let vec3 = delta.scale(t).add_vec(&mob_pos);
            mob.move_control.set_wanted_position(
                vec3.x,
                vec3.y,
                vec3.z,
                self.inner.speed_modifier,
            );
        }
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move, GoalFlag::Look].into_iter().collect()
    }
}

/// `MoveToBlockGoal` — абстрактная цель «найти и встать на блок»
/// (гнездование, поедание блоков и т.п.). `is_valid_target` —
/// абстрактный метод Java; здесь — callback.
pub struct MoveToBlockGoal {
    speed_modifier: f64,
    next_start_tick: i32,
    try_ticks: i32,
    max_stay_ticks: i32,
    block_pos: BlockPos,
    reached_target: bool,
    search_range: i32,
    vertical_search_range: i32,
    pub(crate) vertical_search_start: i32,
    is_valid_target: Box<dyn Fn(&dyn LevelReader, BlockPos) -> bool>,
    /// `nextStartTick(creature)`-переопределение наследников
    /// (CatLieOnBedGoal → 40 вместо random 200+rand(200)).
    pub(crate) next_start_tick_override: Option<i32>,
}

impl MoveToBlockGoal {
    /// `new(mob, speedModifier, searchRange)` — verticalSearchRange 1.
    pub fn new(
        mob: &Mob,
        speed_modifier: f64,
        search_range: i32,
        is_valid_target: impl Fn(&dyn LevelReader, BlockPos) -> bool + 'static,
    ) -> Self {
        Self::with_vertical(mob, speed_modifier, search_range, 1, is_valid_target)
    }

    /// `new(mob, speedModifier, searchRange, verticalSearchRange)`.
    pub fn with_vertical(
        _mob: &Mob,
        speed_modifier: f64,
        search_range: i32,
        vertical_search_range: i32,
        is_valid_target: impl Fn(&dyn LevelReader, BlockPos) -> bool + 'static,
    ) -> Self {
        Self {
            speed_modifier,
            next_start_tick: 0,
            try_ticks: 0,
            max_stay_ticks: 0,
            block_pos: BlockPos::ZERO,
            reached_target: false,
            search_range,
            vertical_search_range,
            vertical_search_start: 0,
            is_valid_target: Box::new(is_valid_target),
            next_start_tick_override: None,
        }
    }

    /// `acceptedDistance()` — 1.0.
    pub fn accepted_distance(&self) -> f64 {
        1.0
    }

    /// `getMoveToTarget()` — `blockPos.above()`.
    pub fn get_move_to_target(&self) -> BlockPos {
        self.block_pos.above()
    }

    /// `isReachedTarget()`.
    pub fn is_reached_target(&self) -> bool {
        self.reached_target
    }

    /// `shouldRecalculatePath()` — каждые 40 тиков.
    pub fn should_recalculate_path(&self) -> bool {
        self.try_ticks % 40 == 0
    }

    /// `findNearestBlock()` — спиральный поиск вокруг моба.
    fn find_nearest_block(&mut self, mob: &mut Mob) -> bool {
        let Some(level) = mob.level() else { return false };
        let level_ref: &dyn LevelReader = level.as_ref();
        let i = self.search_range;
        let i1 = self.vertical_search_range;
        let block_pos = mob.living.entity.block_position();

        let mut i2 = self.vertical_search_start;
        while i2 <= i1 {
            let mut i3 = 0;
            while i3 < i {
                let mut i4 = 0;
                while i4 <= i3 {
                    let mut i5 = if i4 < i3 && i4 > -i3 { i3 } else { 0 };
                    while i5 <= i3 {
                        let candidate = BlockPos::new(
                            block_pos.x + i4,
                            block_pos.y + i2 - 1,
                            block_pos.z + i5,
                        );
                        if mob.is_within_home(candidate)
                            && (self.is_valid_target)(level_ref, candidate)
                        {
                            self.block_pos = candidate;
                            mob.moving_target = if candidate == BlockPos::ZERO {
                                None
                            } else {
                                Some(candidate)
                            };
                            return true;
                        }
                        i5 = if i5 > 0 { -i5 } else { 1 - i5 };
                    }
                    i4 = if i4 > 0 { -i4 } else { 1 - i4 };
                }
                i3 += 1;
            }
            i2 = if i2 > 0 { -i2 } else { 1 - i2 };
        }
        false
    }
}

impl Goal for MoveToBlockGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        if self.next_start_tick > 0 {
            self.next_start_tick -= 1;
            return false;
        }
        self.next_start_tick = self.next_start_tick_override.unwrap_or_else(|| {
            Self::reduced_tick_delay_static(200)
                + mob.random.borrow_mut().next_int_bound(200)
        });
        self.find_nearest_block(mob)
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        let Some(level) = mob.level() else { return false };
        let level_ref: &dyn LevelReader = level.as_ref();
        self.try_ticks >= -self.max_stay_ticks
            && self.try_ticks <= 1200
            && (self.is_valid_target)(level_ref, self.block_pos)
    }

    fn start(&mut self, mob: &mut Mob) {
        // `moveMobToBlock`.
        if let Some(nav) = mob.get_navigation() {
            nav.borrow_mut().move_to_pos(
                mob,
                self.block_pos.x as f64 + 0.5,
                self.block_pos.y as f64 + 1.0,
                self.block_pos.z as f64 + 0.5,
                self.speed_modifier,
            );
        }
        self.try_ticks = 0;
        self.max_stay_ticks = {
            let mut r = mob.random.borrow_mut();
            let inner = r.next_int_bound(1200);
            r.next_int_bound(inner + 1200) + 1200
        };
    }

    fn stop(&mut self, mob: &mut Mob) {
        mob.moving_target = None;
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &mut Mob) {
        let move_to_target = self.get_move_to_target();
        if !move_to_target
            .closer_to_center_than(mob.living.entity.position(), self.accepted_distance())
        {
            self.reached_target = false;
            self.try_ticks += 1;
            if self.should_recalculate_path() {
                if let Some(nav) = mob.get_navigation() {
                    nav.borrow_mut().move_to_pos(
                        mob,
                        move_to_target.x as f64 + 0.5,
                        move_to_target.y as f64,
                        move_to_target.z as f64 + 0.5,
                        self.speed_modifier,
                    );
                }
            }
        } else {
            self.reached_target = true;
            self.try_ticks -= 1;
        }
    }

    fn get_flags(&self) -> GoalFlagSet {
        [GoalFlag::Move, GoalFlag::Jump].into_iter().collect()
    }
}


/// `RemoveBlockGoal` — моб (зомби-вариант) целенаправленно
/// разрушает целевой блок: extends `MoveToBlockGoal`
/// (searchRange=24, verticalSearchRange=ctor-arg), mobGriefing
/// проверка, egg-bounce анимация при подходе, через 60 тиков
/// `removeBlock` + 20 POOF-частиц.
pub struct RemoveBlockGoal {
    /// Встроенный `MoveToBlockGoal`.
    goal: MoveToBlockGoal,
    /// `blockToRemove` — id блока-цели.
    pub block_to_remove: crate::resources::identifier::Identifier,
    /// `ticksSinceReachedGoal`.
    pub ticks_since_reached_goal: i32,
}

impl RemoveBlockGoal {
    /// `new(blockToRemove, removerMob, speedModifier, searchRange)` —
    /// `super(mob, speed, 24, searchRange)`.
    pub fn new(
        block_to_remove: crate::resources::identifier::Identifier,
        mob: &Mob,
        speed_modifier: f64,
        search_range: i32,
    ) -> Self {
        let id = block_to_remove.clone();
        let goal = MoveToBlockGoal::with_vertical(
            mob,
            speed_modifier,
            24,
            search_range,
            move |level: &dyn LevelReader, pos: BlockPos| {
                Self::is_valid_target_static(level, pos, &id)
            },
        );
        Self { goal, block_to_remove, ticks_since_reached_goal: 0 }
    }

    /// `isValidTarget(level, pos)` — блок совпадает,
    /// `above()` и `above(2)` — воздух.
    fn is_valid_target_static(
        level: &dyn LevelReader,
        pos: BlockPos,
        block_to_remove: &crate::resources::identifier::Identifier,
    ) -> bool {
        let Some(state) = level.get_block_state(pos) else {
            return false;
        };
        if state.block.id != *block_to_remove {
            return false;
        }
        level
            .get_block_state(pos.above())
            .map(|s| s.is_air())
            .unwrap_or(false)
            && level
                .get_block_state(pos.above().above())
                .map(|s| s.is_air())
                .unwrap_or(false)
    }

    /// `getPosWithBlock(pos, level)` — сам pos или сосед
    /// (below/west/east/north/south/below²) с целевым блоком.
    fn get_pos_with_block(
        &self,
        pos: BlockPos,
        level: &dyn LevelReader,
    ) -> Option<BlockPos> {
        let is_target = |p: BlockPos| {
            level
                .get_block_state(p)
                .map(|s| s.block.id == self.block_to_remove)
                .unwrap_or(false)
        };
        if is_target(pos) {
            return Some(pos);
        }
        let below = pos.below();
        for candidate in [
            below,
            pos.west(),
            pos.east(),
            pos.north(),
            pos.south(),
            below.below(),
        ] {
            if is_target(candidate) {
                return Some(candidate);
            }
        }
        None
    }

    /// `playDestroyProgressSound` / `playBreakSound` —
    /// в Java пустые override'ы; звуки добавляются наследниками.
    pub fn play_destroy_progress_sound(&self, _level: &dyn LevelReader, _pos: BlockPos) {}

    /// `playBreakSound`.
    pub fn play_break_sound(&self, _level: &dyn LevelReader, _pos: BlockPos) {}
}

impl Goal for RemoveBlockGoal {
    fn can_use(&mut self, mob: &mut Mob) -> bool {
        let mob_griefing = mob.level().map(|l| l.mob_griefing()).unwrap_or(true);
        if !mob_griefing {
            return false;
        }
        if self.goal.next_start_tick > 0 {
            self.goal.next_start_tick -= 1;
            return false;
        }
        if self.goal.find_nearest_block(mob) {
            self.goal.next_start_tick = Self::reduced_tick_delay_static(20);
            return true;
        }
        self.goal.next_start_tick = Self::reduced_tick_delay_static(200)
            + mob.random.borrow_mut().next_int_bound(200);
        false
    }

    fn can_continue_to_use(&mut self, mob: &mut Mob) -> bool {
        Goal::can_continue_to_use(&mut self.goal, mob)
    }

    fn is_interruptable(&self) -> bool {
        self.goal.is_interruptable()
    }

    fn start(&mut self, mob: &mut Mob) {
        Goal::start(&mut self.goal, mob);
        self.ticks_since_reached_goal = 0;
    }

    fn stop(&mut self, mob: &mut Mob) {
        Goal::stop(&mut self.goal, mob);
        mob.living.entity.fall_distance = 1.0;
    }

    fn requires_update_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &mut Mob) {
        Goal::tick(&mut self.goal, mob);
        let Some(level) = mob.level() else {
            return;
        };
        let block_pos = mob.living.entity.block_position();
        let level_ref: &dyn LevelReader = level.as_ref();
        if !self.goal.is_reached_target() {
            return;
        }
        let Some(pos_with_block) = self.get_pos_with_block(block_pos, level_ref) else {
            return;
        };
        if self.ticks_since_reached_goal > 0 {
            // Egg-bounce: подброс вверх каждые 2 тика, вниз на чётных.
            let delta = mob.living.entity.get_delta_movement();
            mob.living.entity.set_delta_movement(delta.x, 0.3, delta.z);
            // ITEM(EGG)-частицы — только серверная сторона.
            let egg = crate::resources::identifier::Identifier::with_default_namespace("egg")
                .ok()
                .and_then(|id| crate::world::item::item::item_by_identifier(&id))
                .map(|item| {
                    crate::world::item::item_stack::ItemStack::new(
                        crate::core::holder::Holder::Direct(item.as_ref().clone()),
                        1,
                    )
                });
            if let Some(egg_stack) = egg {
                if let Some(particle_type) =
                    crate::core::particles::particle_types::particle_type_by_identifier(
                        &crate::resources::identifier::Identifier::with_default_namespace("item")
                            .unwrap(),
                    )
                {
                    let options = crate::core::particles::particle_options::ParticleOptions {
                        type_: particle_type,
                        data: crate::core::particles::particle_options::ParticleOptionsData::Item(
                            egg_stack,
                        ),
                    };
                    let mut r = mob.random.borrow_mut();
                    level.send_particles(
                        &options,
                        pos_with_block.x as f64 + 0.5,
                        pos_with_block.y as f64 + 0.7,
                        pos_with_block.z as f64 + 0.5,
                        3,
                        (r.next_float() - 0.5) as f64 * 0.08,
                        (r.next_float() - 0.5) as f64 * 0.08,
                        (r.next_float() - 0.5) as f64 * 0.08,
                        0.15,
                    );
                }
            }
        }
        if self.ticks_since_reached_goal % 2 == 0 {
            let delta = mob.living.entity.get_delta_movement();
            mob.living.entity.set_delta_movement(delta.x, -0.3, delta.z);
            if self.ticks_since_reached_goal % 6 == 0 {
                self.play_destroy_progress_sound(level_ref, self.goal.block_pos);
            }
        }
        if self.ticks_since_reached_goal > 60 {
            // CraftBukkit EntityInteractEvent — на уровне event-bus'а.
            level.destroy_block(pos_with_block, false);
            // 20 POOF-частиц.
            if let Some(particle_type) =
                crate::core::particles::particle_types::particle_type_by_identifier(
                    &crate::resources::identifier::Identifier::with_default_namespace("poof")
                        .unwrap(),
                )
            {
                let options =
                    crate::core::particles::particle_options::ParticleOptions::simple(particle_type);
                let mut r = mob.random.borrow_mut();
                for _ in 0..20 {
                    level.send_particles(
                        &options,
                        pos_with_block.x as f64 + 0.5,
                        pos_with_block.y as f64,
                        pos_with_block.z as f64 + 0.5,
                        1,
                        r.next_gaussian() * 0.02,
                        r.next_gaussian() * 0.02,
                        r.next_gaussian() * 0.02,
                        0.15,
                    );
                }
            }
            self.play_break_sound(level_ref, pos_with_block);
        }
        self.ticks_since_reached_goal += 1;
    }

    fn get_flags(&self) -> GoalFlagSet {
        self.goal.get_flags()
    }
}
