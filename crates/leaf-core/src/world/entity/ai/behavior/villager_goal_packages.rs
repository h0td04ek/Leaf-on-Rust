//! Перенос `net.minecraft.world.entity.ai.behavior.VillagerGoalPackages`
//! — регистрация brain-активностей villager'а (core/work/meet/
//! rest/idle/panic/pre_raid/raid/hide/play) с приоритетами
//! и memory-условиями ванили 1.21.11 (Purpur-ветки
//! `clericsFarmWarts`/`villagerDisplayTradeItem` — через хуки
//! `LevelReader`).
//!
//! Вызывается из `Mob.register_brain_goals` через колбэк
//! `goal_packages` (см. `villager::init_villager`).

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::entity::schedule::activity::{self, Activity};
use crate::tags::block_tags::BlockTags;
use crate::tags::is_block_in_named_tag;
use crate::world::entity::ai::behavior::behavior::{
    cond_set, BehaviorControl, MemoryStatusPair,
};
use crate::world::entity::ai::behavior::gate::{
    run_one, trigger_one_shuffled, GateBehavior, OrderPolicy, RunningPolicy,
};
use crate::world::entity::ai::behavior::poi_behaviors::{
    acquire_poi, assign_profession_from_job_site, go_to_potential_job_site,
    reset_profession, set_closest_home_as_walk_target,
};
use crate::world::entity::ai::behavior::sinks::{
    inside_brownian_walk, set_entity_look_target_type, set_hidden_state,
    DoNothing, LookAtTargetSink, MoveToTargetSink, Swim,
};
use crate::world::entity::ai::behavior::simple::{
    conds, reset_raid_status, react_to_bell, set_look_and_interact,
    set_raid_status, set_walk_target_from_look_target, stroll_to_poi,
    update_activity_from_schedule, villager_calm_down, wake_up,
};
use crate::world::entity::ai::behavior::villager_extra::{
    harvest_farmland, interact_with_door, jump_on_bed,
    look_and_follow_trading_player_sink, ring_bell,
    set_walk_target_from_block_memory, sleep_in_bed, socialize_at_bell,
    trade_with_villager, use_bonemeal, villager_make_love,
    villager_panic_trigger, work_at_composter, work_at_poi, yield_job_site,
};
use crate::world::entity::ai::behavior::villager_final::{
    can_breed_living, celebrate_villagers_survived_raid, give_gift_to_hero,
    interact_with_filters, interact_with_type, poi_competitor_scan,
    sequence_with_point, set_entity_look_target_category, show_trades_to_player,
    stroll_to_poi_list,
};
use crate::world::entity::ai::behavior::walk_behaviors::{
    go_to_closest_village, go_to_wanted_item, move_to_sky_seeing_spot,
    stroll_around_poi, village_bound_random_stroll, village_bound_random_stroll_dist,
};
use crate::world::entity::ai::memory::memory::{keys, MemoryModuleType, MemoryStatus};
use crate::world::entity::ai::village::poi::poi_record::holder_is;
use crate::world::entity::ai::village::poi::poi_type::PoiType;
use crate::world::entity::ai::village::poi::poi_types;
use crate::world::entity::mob::Mob;
use crate::world::entity::mob_category::MobCategory;
use crate::world::entity::npc::villager::villager_data;
use crate::world::entity::npc::villager::villager_profession::{
    self, PoiPredicate, VillagerProfession,
};
use crate::world::entity::living_entity::LivingEntity;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::level_reader::LevelReader;

/// `INTERACT_DIST_SQR`/`INTERACT_WALKUP_DIST`/`INTERACT_SPEED_MODIFIER`.
pub const INTERACT_DIST_SQR: i32 = 5;
pub const INTERACT_WALKUP_DIST: i32 = 2;
pub const INTERACT_SPEED_MODIFIER: f32 = 0.5;
const STROLL_SPEED_MODIFIER: f32 = 0.4;

fn act(name: &str) -> Arc<Activity> {
    activity::activity_by_name(name)
        .or_else(|| activity::activity_by_name("idle"))
        .expect("activity registry not bootstrapped")
}

/// `Holder<VillagerProfession>` → `(heldJobSite, acquirableJobSite)`.
fn job_site_predicates(mob: &Mob) -> (PoiPredicate, PoiPredicate) {
    let data = mob.get_villager_data();
    let prof: Option<VillagerProfession> = match &data.profession {
        Holder::Direct(p) => Some(p.clone()),
        Holder::Key(k) => villager_profession::get(k),
        Holder::Reference(_) => None,
    };
    match prof {
        Some(p) => (p.held_job_site.clone(), p.acquirable_job_site.clone()),
        None => (Arc::new(|_| false), Arc::new(|_| false)),
    }
}

/// `poiType -> poiType.is(key)` — Holder-обёртка в `Rc`-предикат.
fn poi_is(
    key: crate::resources::resource_key::ResourceKey<PoiType>,
) -> Rc<dyn Fn(&Holder<PoiType>) -> bool + Send + Sync> {
    Rc::new(move |h| holder_is(h, &key))
}

/// PoiPredicate (Arc) → Rc для `validate_nearby_poi`/`acquire_poi`.
fn pred_rc(
    pred: &PoiPredicate,
) -> Rc<dyn Fn(&Holder<PoiType>) -> bool + Send + Sync> {
    let p = pred.clone();
    Rc::new(move |h| p(h))
}

/// `VillagerGoalPackages::validateBedPoi`.
fn validate_bed_poi(level: &Arc<dyn LevelReader>, pos: BlockPos) -> bool {
    let Some(state) = level.get_block_state(pos) else {
        return false;
    };
    is_bed_and_not_occupied(&state)
}

/// `state.is(BEDS) && !state.getValue(OC CUPIED)`.
fn is_bed_and_not_occupied(state: &BlockState) -> bool {
    if !is_block_in_named_tag(&state.block.id, &BlockTags::beds()) {
        return false;
    }
    match state.values.get_by_name("occupied") {
        Some(crate::world::level::block::state::property::StateValue::Bool(b)) => !b,
        _ => true,
    }
}

/// `getCorePackage(profession, speedModifier)`.
pub fn get_core_package(
    held_job_site: &PoiPredicate,
    acquirable_job_site: &PoiPredicate,
    speed_modifier: f32,
) -> Vec<(i32, Box<dyn BehaviorControl>)> {
    vec![
        (0, Box::new(Swim::new(0.8)) as Box<dyn BehaviorControl>),
        (0, Box::new(interact_with_door()) as Box<dyn BehaviorControl>),
        (0, Box::new(LookAtTargetSink::new(45, 90)) as Box<dyn BehaviorControl>),
        (0, Box::new(villager_panic_trigger()) as Box<dyn BehaviorControl>),
        (0, Box::new(wake_up()) as Box<dyn BehaviorControl>),
        (0, Box::new(react_to_bell()) as Box<dyn BehaviorControl>),
        (0, Box::new(set_raid_status()) as Box<dyn BehaviorControl>),
        (
            0,
            Box::new(crate::world::entity::ai::behavior::villager_extra::validate_nearby_poi(
                pred_rc(held_job_site),
                keys::job_site(),
            )) as Box<dyn BehaviorControl>,
        ),
        (
            0,
            Box::new(crate::world::entity::ai::behavior::villager_extra::validate_nearby_poi(
                pred_rc(acquirable_job_site),
                keys::potential_job_site(),
            )) as Box<dyn BehaviorControl>,
        ),
        (1, Box::new(MoveToTargetSink::new()) as Box<dyn BehaviorControl>),
        (2, Box::new(poi_competitor_scan()) as Box<dyn BehaviorControl>),
        (
            3,
            Box::new(look_and_follow_trading_player_sink(speed_modifier))
                as Box<dyn BehaviorControl>,
        ),
        (
            5,
            Box::new(go_to_wanted_item(
                |v| !v.living.is_sleeping,
                speed_modifier,
                false,
                4,
            )) as Box<dyn BehaviorControl>,
        ),
        (
            6,
            Box::new(acquire_poi(
                pred_rc(acquirable_job_site),
                keys::potential_job_site(),
                keys::job_site(),
                true,
                None,
                Rc::new(|_level, _pos| true),
            )) as Box<dyn BehaviorControl>,
        ),
        (
            7,
            Box::new(go_to_potential_job_site(speed_modifier)) as Box<dyn BehaviorControl>,
        ),
        (8, Box::new(yield_job_site(speed_modifier)) as Box<dyn BehaviorControl>),
        (
            10,
            Box::new(acquire_poi(
                poi_is(poi_types::home()),
                keys::home(),
                keys::home(),
                false,
                Some(14),
                Rc::new(validate_bed_poi),
            )) as Box<dyn BehaviorControl>,
        ),
        (
            10,
            Box::new(acquire_poi(
                poi_is(poi_types::meeting()),
                keys::meeting_point(),
                keys::meeting_point(),
                true,
                Some(14),
                Rc::new(|_level, _pos| true),
            )) as Box<dyn BehaviorControl>,
        ),
        (
            10,
            Box::new(assign_profession_from_job_site()) as Box<dyn BehaviorControl>,
        ),
        (10, Box::new(reset_profession()) as Box<dyn BehaviorControl>),
    ]
}

/// `getWorkPackage(profession, speedModifier, clericsFarmWarts)`.
#[allow(clippy::too_many_arguments)]
pub fn get_work_package(
    speed_modifier: f32,
    is_farmer: bool,
    clerics_farm_warts: bool,
    is_cleric: bool,
) -> Vec<(i32, Box<dyn BehaviorControl>)> {
    let work_at_poi_behavior: Box<dyn BehaviorControl> =
        if is_farmer || (clerics_farm_warts && is_cleric) {
            Box::new(work_at_composter())
        } else {
            Box::new(work_at_poi(None))
        };
    let harvest_weight = if is_farmer { 2 } else { 5 };
    let bonemeal_weight = if is_farmer { 4 } else { 7 };
    vec![
        get_minimal_look_behavior(),
        (
            5,
            Box::new(run_one(
                HashMap::new(),
                vec![
                    (work_at_poi_behavior, 7),
                    (
                        Box::new(stroll_around_poi(keys::job_site(), STROLL_SPEED_MODIFIER, 4))
                            as Box<dyn BehaviorControl>,
                        2,
                    ),
                    (
                        Box::new(stroll_to_poi(keys::job_site(), STROLL_SPEED_MODIFIER, 1, 10))
                            as Box<dyn BehaviorControl>,
                        5,
                    ),
                    (
                        Box::new(stroll_to_poi_list(
                            keys::secondary_job_site(),
                            speed_modifier,
                            1,
                            6,
                            keys::job_site(),
                        )) as Box<dyn BehaviorControl>,
                        5,
                    ),
                    (
                        Box::new(harvest_farmland()) as Box<dyn BehaviorControl>,
                        harvest_weight,
                    ),
                    (
                        Box::new(use_bonemeal()) as Box<dyn BehaviorControl>,
                        bonemeal_weight,
                    ),
                ],
            )) as Box<dyn BehaviorControl>,
        ),
        (
            10,
            Box::new(show_trades_to_player(400, 1600)) as Box<dyn BehaviorControl>,
        ),
        (
            10,
            Box::new(set_look_and_interact("player", 4)) as Box<dyn BehaviorControl>,
        ),
        (
            2,
            Box::new(set_walk_target_from_block_memory(
                keys::job_site(),
                speed_modifier,
                9,
                100,
                1200,
            )) as Box<dyn BehaviorControl>,
        ),
        (3, Box::new(give_gift_to_hero(100)) as Box<dyn BehaviorControl>),
        (
            99,
            Box::new(update_activity_from_schedule()) as Box<dyn BehaviorControl>,
        ),
    ]
}

/// `getPlayPackage(speedModifier)`.
pub fn get_play_package(speed_modifier: f32) -> Vec<(i32, Box<dyn BehaviorControl>)> {
    vec![
        (
            0,
            Box::new(MoveToTargetSink::with_duration(80, 120)) as Box<dyn BehaviorControl>,
        ),
        get_full_look_behavior(),
        (
            5,
            Box::new(crate::world::entity::ai::behavior::animal_behaviors::play_tag_with_other_kids())
                as Box<dyn BehaviorControl>,
        ),
        (
            5,
            Box::new(run_one(
                conds(&[(
                    keys::visible_villager_babies(),
                    MemoryStatus::ValueAbsent,
                )]),
                vec![
                    (
                        Box::new(interact_with_type(
                            "villager",
                            8,
                            keys::interaction_target(),
                            speed_modifier,
                            2,
                        )) as Box<dyn BehaviorControl>,
                        2,
                    ),
                    (
                        Box::new(interact_with_type(
                            "cat",
                            8,
                            keys::interaction_target(),
                            speed_modifier,
                            2,
                        )) as Box<dyn BehaviorControl>,
                        1,
                    ),
                    (
                        Box::new(village_bound_random_stroll(speed_modifier))
                            as Box<dyn BehaviorControl>,
                        1,
                    ),
                    (
                        Box::new(set_walk_target_from_look_target(speed_modifier, 2))
                            as Box<dyn BehaviorControl>,
                        1,
                    ),
                    (
                        Box::new(jump_on_bed(speed_modifier)) as Box<dyn BehaviorControl>,
                        2,
                    ),
                    (
                        Box::new(DoNothing::new(20, 40)) as Box<dyn BehaviorControl>,
                        2,
                    ),
                ],
            )) as Box<dyn BehaviorControl>,
        ),
        (
            99,
            Box::new(update_activity_from_schedule()) as Box<dyn BehaviorControl>,
        ),
    ]
}

/// `getRestPackage(profession, speedModifier)`.
pub fn get_rest_package(speed_modifier: f32) -> Vec<(i32, Box<dyn BehaviorControl>)> {
    vec![
        (
            2,
            Box::new(set_walk_target_from_block_memory(
                keys::home(),
                speed_modifier,
                1,
                150,
                1200,
            )) as Box<dyn BehaviorControl>,
        ),
        (
            3,
            Box::new(crate::world::entity::ai::behavior::villager_extra::validate_nearby_poi(
                poi_is(poi_types::home()),
                keys::home(),
            )) as Box<dyn BehaviorControl>,
        ),
        (3, Box::new(sleep_in_bed()) as Box<dyn BehaviorControl>),
        (
            5,
            Box::new(run_one(
                conds(&[(keys::home(), MemoryStatus::ValueAbsent)]),
                vec![
                    (
                        Box::new(set_closest_home_as_walk_target(speed_modifier))
                            as Box<dyn BehaviorControl>,
                        1,
                    ),
                    (
                        Box::new(inside_brownian_walk(speed_modifier))
                            as Box<dyn BehaviorControl>,
                        4,
                    ),
                    (
                        Box::new(go_to_closest_village(speed_modifier, 4))
                            as Box<dyn BehaviorControl>,
                        2,
                    ),
                    (
                        Box::new(DoNothing::new(20, 40)) as Box<dyn BehaviorControl>,
                        2,
                    ),
                ],
            )) as Box<dyn BehaviorControl>,
        ),
        get_minimal_look_behavior(),
        (
            99,
            Box::new(update_activity_from_schedule()) as Box<dyn BehaviorControl>,
        ),
    ]
}

/// `AgeableMob::canBreed` — фильтр для self (`&Mob`).
fn can_breed_mob(m: &Mob) -> bool {
    m.living.entity.is_alive()
        && !m.living.entity.is_baby()
        && m.living.entity.is_in_love()
}

/// `AgeableMob::canBreed` — фильтр для target (`&LivingEntity`).
fn can_breed_target(e: &LivingEntity) -> bool {
    can_breed_living(e)
}

/// `getMeetPackage(profession, speedModifier)`.
pub fn get_meet_package(speed_modifier: f32) -> Vec<(i32, Box<dyn BehaviorControl>)> {
    vec![
        (
            2,
            Box::new(trigger_one_shuffled(vec![
                (
                    stroll_around_poi(keys::meeting_point(), 0.4, 40).trigger_rc(),
                    2,
                ),
                (socialize_at_bell().trigger_rc(), 2),
            ])) as Box<dyn BehaviorControl>,
        ),
        (
            10,
            Box::new(show_trades_to_player(400, 1600)) as Box<dyn BehaviorControl>,
        ),
        (
            10,
            Box::new(set_look_and_interact("player", 4)) as Box<dyn BehaviorControl>,
        ),
        (
            2,
            Box::new(set_walk_target_from_block_memory(
                keys::meeting_point(),
                speed_modifier,
                6,
                100,
                200,
            )) as Box<dyn BehaviorControl>,
        ),
        (3, Box::new(give_gift_to_hero(100)) as Box<dyn BehaviorControl>),
        (
            3,
            Box::new(crate::world::entity::ai::behavior::villager_extra::validate_nearby_poi(
                poi_is(poi_types::meeting()),
                keys::meeting_point(),
            )) as Box<dyn BehaviorControl>,
        ),
        (
            3,
            Box::new(GateBehavior::new(
                HashMap::new(),
                HashSet::from([keys::interaction_target()]),
                OrderPolicy::Ordered,
                RunningPolicy::RunOne,
                vec![(Box::new(trade_with_villager()) as Box<dyn BehaviorControl>, 1)],
            )) as Box<dyn BehaviorControl>,
        ),
        get_full_look_behavior(),
        (
            99,
            Box::new(update_activity_from_schedule()) as Box<dyn BehaviorControl>,
        ),
    ]
}

/// `getIdlePackage(profession, speedModifier)`.
pub fn get_idle_package(speed_modifier: f32) -> Vec<(i32, Box<dyn BehaviorControl>)> {
    vec![
        (
            2,
            Box::new(run_one(
                HashMap::new(),
                vec![
                    (
                        Box::new(interact_with_type(
                            "villager",
                            8,
                            keys::interaction_target(),
                            speed_modifier,
                            2,
                        )) as Box<dyn BehaviorControl>,
                        2,
                    ),
                    (
                        Box::new(interact_with_filters(
                            "villager",
                            8,
                            Arc::new(can_breed_mob),
                            Arc::new(can_breed_target),
                            keys::breed_target(),
                            speed_modifier,
                            2,
                        )) as Box<dyn BehaviorControl>,
                        1,
                    ),
                    (
                        Box::new(interact_with_type(
                            "cat",
                            8,
                            keys::interaction_target(),
                            speed_modifier,
                            2,
                        )) as Box<dyn BehaviorControl>,
                        1,
                    ),
                    (
                        Box::new(village_bound_random_stroll(speed_modifier))
                            as Box<dyn BehaviorControl>,
                        1,
                    ),
                    (
                        Box::new(set_walk_target_from_look_target(speed_modifier, 2))
                            as Box<dyn BehaviorControl>,
                        1,
                    ),
                    (
                        Box::new(jump_on_bed(speed_modifier)) as Box<dyn BehaviorControl>,
                        1,
                    ),
                    (
                        Box::new(DoNothing::new(30, 60)) as Box<dyn BehaviorControl>,
                        1,
                    ),
                ],
            )) as Box<dyn BehaviorControl>,
        ),
        (3, Box::new(give_gift_to_hero(100)) as Box<dyn BehaviorControl>),
        (
            3,
            Box::new(set_look_and_interact("player", 4)) as Box<dyn BehaviorControl>,
        ),
        (
            3,
            Box::new(show_trades_to_player(400, 1600)) as Box<dyn BehaviorControl>,
        ),
        (
            3,
            Box::new(GateBehavior::new(
                HashMap::new(),
                HashSet::from([keys::interaction_target()]),
                OrderPolicy::Ordered,
                RunningPolicy::RunOne,
                vec![(Box::new(trade_with_villager()) as Box<dyn BehaviorControl>, 1)],
            )) as Box<dyn BehaviorControl>,
        ),
        (
            3,
            Box::new(GateBehavior::new(
                HashMap::new(),
                HashSet::from([keys::breed_target()]),
                OrderPolicy::Ordered,
                RunningPolicy::RunOne,
                vec![(Box::new(villager_make_love()) as Box<dyn BehaviorControl>, 1)],
            )) as Box<dyn BehaviorControl>,
        ),
        get_full_look_behavior(),
        (
            99,
            Box::new(update_activity_from_schedule()) as Box<dyn BehaviorControl>,
        ),
    ]
}

/// `getPanicPackage(profession, speedModifier)`.
pub fn get_panic_package(speed_modifier: f32) -> Vec<(i32, Box<dyn BehaviorControl>)> {
    let f = speed_modifier * 1.5;
    vec![
        (0, Box::new(villager_calm_down()) as Box<dyn BehaviorControl>),
        (
            1,
            Box::new(crate::world::entity::ai::behavior::animal_behaviors::set_walk_target_away_from_entity(
                keys::nearest_hostile(),
                f,
                6,
                false,
            )) as Box<dyn BehaviorControl>,
        ),
        (
            1,
            Box::new(crate::world::entity::ai::behavior::animal_behaviors::set_walk_target_away_from_entity(
                keys::hurt_by_entity(),
                f,
                6,
                false,
            )) as Box<dyn BehaviorControl>,
        ),
        (
            3,
            Box::new(village_bound_random_stroll_dist(f, 2, 2))
                as Box<dyn BehaviorControl>,
        ),
        get_minimal_look_behavior(),
    ]
}

/// `getPreRaidPackage(profession, speedModifier)`.
pub fn get_pre_raid_package(speed_modifier: f32) -> Vec<(i32, Box<dyn BehaviorControl>)> {
    vec![
        (0, Box::new(ring_bell()) as Box<dyn BehaviorControl>),
        (
            0,
            Box::new(trigger_one_shuffled(vec![
                (
                    set_walk_target_from_block_memory(
                        keys::meeting_point(),
                        speed_modifier * 1.5,
                        2,
                        150,
                        200,
                    )
                    .trigger_rc(),
                    6,
                ),
                (
                    village_bound_random_stroll(speed_modifier * 1.5).trigger_rc(),
                    2,
                ),
            ])) as Box<dyn BehaviorControl>,
        ),
        get_minimal_look_behavior(),
        (99, Box::new(reset_raid_status()) as Box<dyn BehaviorControl>),
    ]
}

/// `raidExistsAndActive`.
fn raid_exists_and_active(level: &Arc<dyn LevelReader>, mob: &Mob) -> bool {
    let pos = mob.living.entity.block_position();
    level
        .raid_at(pos)
        .map(|r| {
            let r = r.borrow();
            r.is_active() && !r.is_victory() && !r.is_loss()
        })
        .unwrap_or(false)
}

/// `raidExistsAndNotVictory`.
fn raid_exists_and_not_victory(level: &Arc<dyn LevelReader>, mob: &Mob) -> bool {
    let pos = mob.living.entity.block_position();
    level
        .raid_at(pos)
        .map(|r| r.borrow().is_victory())
        .unwrap_or(false)
}

/// `getRaidPackage(profession, speedModifier)`.
pub fn get_raid_package(speed_modifier: f32) -> Vec<(i32, Box<dyn BehaviorControl>)> {
    let pred_active = Rc::new(raid_exists_and_active);
    let pred_not_victory = Rc::new(raid_exists_and_not_victory);
    let pa = pred_active.clone();
    let pnv = pred_not_victory.clone();
    let inner1 = trigger_one_shuffled(vec![
        (move_to_sky_seeing_spot(speed_modifier).trigger_rc(), 5),
        (
            village_bound_random_stroll(speed_modifier * 1.1).trigger_rc(),
            2,
        ),
    ]);
    let seq1 = sequence_with_point(
        move |level, mob| pnv(level, mob),
        &inner1,
    );
    let inner2 = crate::world::entity::ai::behavior::animal_behaviors::locate_hiding_place(
        24,
        speed_modifier * 1.4,
        1,
    );
    let seq2 = sequence_with_point(
        move |level, mob| pa(level, mob),
        &inner2,
    );
    vec![
        (0, Box::new(seq1) as Box<dyn BehaviorControl>),
        (
            0,
            Box::new(celebrate_villagers_survived_raid(600, 600))
                as Box<dyn BehaviorControl>,
        ),
        (2, Box::new(seq2) as Box<dyn BehaviorControl>),
        get_minimal_look_behavior(),
        (99, Box::new(reset_raid_status()) as Box<dyn BehaviorControl>),
    ]
}

/// `getHidePackage(profession, speedModifier)`.
pub fn get_hide_package(speed_modifier: f32) -> Vec<(i32, Box<dyn BehaviorControl>)> {
    vec![
        (0, Box::new(set_hidden_state(15, 3)) as Box<dyn BehaviorControl>),
        (
            1,
            Box::new(crate::world::entity::ai::behavior::animal_behaviors::locate_hiding_place(
                32,
                speed_modifier * 1.25,
                2,
            )) as Box<dyn BehaviorControl>,
        ),
        get_minimal_look_behavior(),
    ]
}

/// `getFullLookBehavior()` — Pair(5, RunOne(...)).
pub fn get_full_look_behavior() -> (i32, Box<dyn BehaviorControl>) {
    (
        5,
        Box::new(run_one(
            HashMap::new(),
            vec![
                (
                    Box::new(set_entity_look_target_type_pub("cat", 8.0))
                        as Box<dyn BehaviorControl>,
                    8,
                ),
                (
                    Box::new(set_entity_look_target_type_pub("villager", 8.0))
                        as Box<dyn BehaviorControl>,
                    2,
                ),
                (
                    Box::new(set_entity_look_target_type_pub("player", 8.0))
                        as Box<dyn BehaviorControl>,
                    2,
                ),
                (
                    Box::new(set_entity_look_target_category(MobCategory::Creature, 8.0))
                        as Box<dyn BehaviorControl>,
                    1,
                ),
                (
                    Box::new(set_entity_look_target_category(MobCategory::WaterCreature, 8.0))
                        as Box<dyn BehaviorControl>,
                    1,
                ),
                (
                    Box::new(set_entity_look_target_category(MobCategory::Axolotls, 8.0))
                        as Box<dyn BehaviorControl>,
                    1,
                ),
                (
                    Box::new(set_entity_look_target_category(
                        MobCategory::UndergroundWaterCreature,
                        8.0,
                    )) as Box<dyn BehaviorControl>,
                    1,
                ),
                (
                    Box::new(set_entity_look_target_category(MobCategory::WaterAmbient, 8.0))
                        as Box<dyn BehaviorControl>,
                    1,
                ),
                (
                    Box::new(set_entity_look_target_category(MobCategory::Monster, 8.0))
                        as Box<dyn BehaviorControl>,
                    1,
                ),
                (
                    Box::new(DoNothing::new(30, 60)) as Box<dyn BehaviorControl>,
                    2,
                ),
            ],
        )) as Box<dyn BehaviorControl>,
    )
}

/// `SetEntityLookTarget.create(EntityType, dist)` — sinks-обёртка.
fn set_entity_look_target_type_pub(
    entity_type_id: &str,
    max_dist: f32,
) -> crate::world::entity::ai::behavior::simple::PointBehavior {
    set_entity_look_target_type(entity_type_id, max_dist)
}

/// `getMinimalLookBehavior()` — Pair(5, RunOne(...)).
pub fn get_minimal_look_behavior() -> (i32, Box<dyn BehaviorControl>) {
    (
        5,
        Box::new(run_one(
            HashMap::new(),
            vec![
                (
                    Box::new(set_entity_look_target_type_pub("villager", 8.0))
                        as Box<dyn BehaviorControl>,
                    2,
                ),
                (
                    Box::new(set_entity_look_target_type_pub("player", 8.0))
                        as Box<dyn BehaviorControl>,
                    2,
                ),
                (
                    Box::new(DoNothing::new(30, 60)) as Box<dyn BehaviorControl>,
                    8,
                ),
            ],
        )) as Box<dyn BehaviorControl>,
    )
}

/// `Villager.registerBrainGoals` — activities-часть.
/// Вызывается через `Mob.goal_packages` из
/// `Mob::register_brain_goals` (schedule/core-activities
/// выставляет сам `register_brain_goals`).
pub fn register_villager_packages(
    level: &Arc<dyn LevelReader>,
    mob: &mut Mob,
    brain: &crate::world::entity::ai::brain::Brain,
) {
    let baby = mob.living.entity.is_baby();
    let speed = 0.5f32;
    let (held, acquirable) = job_site_predicates(mob);

    // Purpur `villagerClericsFarmWarts` — дефолт false.
    let clerics_farm_warts = level.purpur_villager_clerics_farm_warts();
    let profession = mob.get_villager_data().profession.clone();
    let is_farmer = villager_data::profession_is(&profession, &villager_profession::farmer());
    let is_cleric = villager_data::profession_is(&profession, &villager_profession::cleric());

    if baby {
        brain.add_activity_pairs_list(act("play"), get_play_package(speed));
    } else {
        brain.add_activity_with_conditions_pairs(
            act("work"),
            get_work_package(speed, is_farmer, clerics_farm_warts, is_cleric),
            cond_set(&[(keys::job_site(), MemoryStatus::ValuePresent)]),
        );
    }
    brain.add_activity_pairs_list(
        act("core"),
        get_core_package(&held, &acquirable, speed),
    );
    brain.add_activity_with_conditions_pairs(
        act("meet"),
        get_meet_package(speed),
        cond_set(&[(keys::meeting_point(), MemoryStatus::ValuePresent)]),
    );
    brain.add_activity_pairs_list(act("rest"), get_rest_package(speed));
    brain.add_activity_pairs_list(act("idle"), get_idle_package(speed));
    brain.add_activity_pairs_list(act("panic"), get_panic_package(speed));
    brain.add_activity_pairs_list(act("pre_raid"), get_pre_raid_package(speed));
    brain.add_activity_pairs_list(act("raid"), get_raid_package(speed));
    brain.add_activity_pairs_list(act("hide"), get_hide_package(speed));
}
