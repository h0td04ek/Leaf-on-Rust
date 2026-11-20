//! Финальная партия villager/AI behavior-классов:
//! `CelebrateVillagersSurvivedRaid`, `GiveGiftToHero`,
//! `GoAndGiveItemsToTarget`, `ShowTradesToPlayer`,
//! `TransportItemsBetweenContainers`, `PoiCompetitorScan`,
//! `StrollToPoiList`, `BehaviorBuilder.sequence`/`triggerIf`
//! и `InteractWith`-обёртки для `VillagerGoalPackages`.
//!
//! Ограничения порта (как в `villager_extra`): memory хранит
//! клоны `LivingEntity`, поэтому villager-поля *других* сущностей
//! (профессия/xp) недоступны; `PoiCompetitorScan` деградирует до
//! проверки типа+JOB_SITE (tie → поздний конкурент выигрывает,
//! как в ванили при равном xp).

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::core::global_pos::GlobalPos;
use crate::core::holder::Holder;
use crate::entity::equipment_slot::EquipmentSlot;
use crate::resources::identifier::Identifier;
use crate::world::entity::ai::behavior::behavior::{
    BehaviorBase, BehaviorControl, BehaviorStatus,
};
use crate::world::entity::ai::behavior::behavior_utils;
use crate::world::entity::ai::behavior::simple::{conds, PointBehavior, TriggerFn};
use crate::world::entity::ai::memory::memory::{
    self, keys, BlockPosTracker, EntityTracker, MemoryModuleType, MemoryStatus,
    NearestVisibleLivingEntities, PositionTracker, WalkTarget,
};
use crate::world::entity::ai::village::poi::poi_type::PoiType;
use crate::world::entity::entity::Entity;
use crate::world::entity::entity_type;
use crate::world::entity::item_entity::ItemEntity;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::Mob;
use crate::world::entity::mob_category::MobCategory;
use crate::world::entity::npc::villager::villager;
use crate::world::entity::npc::villager::villager_profession;
use crate::world::item::component::firework_explosion::{
    FireworkExplosion, FireworkExplosionShape,
};
use crate::world::item::component::fireworks::Fireworks;
use crate::world::item::dye_color::DyeColor;
use crate::world::item::item_stack::ItemStack;
use crate::world::item::trading::merchant_offer::MerchantOffer;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::heightmap::HeightmapType;
use crate::world::level::pathfinder::path::Path;
use crate::world::phys::aabb::Aabb;
use crate::world::phys::vec3::Vec3;

// ===================== helpers =====================

fn brain_of(mob: &Mob) -> Option<Rc<crate::world::entity::ai::brain::Brain>> {
    mob.living.brain.clone()
}

/// `EntityType`-id сущности (`minecraft:villager` → "villager").
fn entity_type_path(e: &Entity) -> String {
    entity_type::holder_type_id(&e.entity_type)
        .map(|id| id.path.to_string())
        .unwrap_or_default()
}

/// `GlobalPos` текущего измерения уровня.
fn global_pos_of(level: &Arc<dyn LevelReader>, pos: BlockPos) -> GlobalPos {
    GlobalPos::of(
        crate::resources::ResourceKey::<crate::world::level::Level>::create_with_registry_name(
            crate::core::registry::registries::dimension(),
            level
                .dimension_id()
                .cloned()
                .unwrap_or_else(|| Identifier::with_default_namespace("overworld").unwrap()),
        ),
        pos,
    )
}

/// `MoveToSkySeeingSpot.hasNoBlocksAbove(level, entity, pos)` —
/// `canSeeSky(pos) && heightmap(MOTION_BLOCKING,pos).y <= entity.y`.
pub fn has_no_blocks_above(level: &Arc<dyn LevelReader>, mob: &Mob, pos: BlockPos) -> bool {
    level.can_see_sky(pos)
        && level
            .get_heightmap_pos(HeightmapType::MotionBlocking, pos)
            .y
            <= mob.living.entity.pos.y as i32
}

/// `ItemStack` из registry-id предмета.
fn item_stack_of(id: &Identifier, count: i32) -> ItemStack {
    crate::world::item::item::item_by_identifier(id)
        .map(|item| {
            ItemStack::new(
                Holder::Direct(item.as_ref().clone()),
                count,
            )
        })
        .unwrap_or_else(ItemStack::empty)
}

/// `BehaviorUtils.throwItem(entity, stack, offset)` —
/// спавн `ItemEntity` с импульсом к `target_pos`
/// (speedMultiplier=(0.3,0.3,0.3), yOffset=0.3).
pub fn throw_item(entity: &LivingEntity, stack: ItemStack, target_pos: Vec3) {
    throw_item_with(entity, stack, target_pos, Vec3::new(0.3, 0.3, 0.3), 0.3);
}

/// `BehaviorUtils.throwItem(entity, stack, offset, speedMultiplier, yOffset)`.
pub fn throw_item_with(
    entity: &LivingEntity,
    stack: ItemStack,
    target_pos: Vec3,
    speed_multiplier: Vec3,
    y_offset: f64,
) {
    if stack.is_empty() {
        return;
    }
    let Some(level) = entity.entity.level.clone() else {
        return;
    };
    let d = entity.entity.get_eye_y() - y_offset;
    let mut item_entity = ItemEntity::new(
        Some(level.clone()),
        entity.entity.pos.x,
        d,
        entity.entity.pos.z,
        stack,
    );
    item_entity.thrower = Some(entity.entity.uuid);
    let vec3 = target_pos
        .subtract_vec(&entity.entity.position())
        .normalize()
        .multiply(speed_multiplier.x, speed_multiplier.y, speed_multiplier.z);
    item_entity
        .entity
        .set_delta_movement(vec3.x, vec3.y, vec3.z);
    item_entity.set_default_pick_up_delay();
    level.add_fresh_item_entity(item_entity);
}

/// `Villager.isHeroOfTheVillage(player)`-проверка memory-цели.
fn is_hero_of_the_village(entity: &LivingEntity) -> bool {
    entity.has_effect(&crate::world::effect::mob_effects::hero_of_the_village())
}

/// `GiveGiftToHero.getNearestTargetableHero`.
fn nearest_hero(mob: &Mob) -> Option<LivingEntity> {
    mob.living
        .brain_get::<LivingEntity>(&keys::nearest_visible_player())
        .filter(|p| is_hero_of_the_village(p))
}

/// `InteractWith.of(type, range, memory, speed, maxDist)` —
/// фильтры `entity -> true`.
pub fn interact_with_type(
    type_id: &'static str,
    interaction_range: i32,
    memory: MemoryModuleType,
    speed_modifier: f32,
    max_dist: i32,
) -> PointBehavior {
    interact_with_filters(
        type_id,
        interaction_range,
        Arc::new(|_| true),
        Arc::new(|_| true),
        memory,
        speed_modifier,
        max_dist,
    )
}

/// `InteractWith.of(type, range, selfFilter, targetFilter, memory,
/// speed, maxDist)` — полная форма.
pub fn interact_with_filters(
    type_id: &'static str,
    interaction_range: i32,
    self_filter: Arc<dyn Fn(&Mob) -> bool + Send + Sync>,
    target_filter: Arc<dyn Fn(&LivingEntity) -> bool + Send + Sync>,
    memory: MemoryModuleType,
    speed_modifier: f32,
    max_dist: i32,
) -> PointBehavior {
    super::villager_extra::interact_with(
        type_id,
        interaction_range,
        self_filter,
        target_filter,
        memory,
        speed_modifier,
        max_dist,
    )
}

/// `AgeableMob::canBreed` — `isAlive && !isBaby && isInLove`.
pub fn can_breed_living(e: &LivingEntity) -> bool {
    e.entity.is_alive() && !e.entity.is_baby() && e.entity.is_in_love()
}

/// `SetEntityLookTarget.create(MobCategory, maxDist)`.
pub fn set_entity_look_target_category(
    category: MobCategory,
    max_dist: f32,
) -> PointBehavior {
    super::sinks::set_entity_look_target(
        move |e| {
            entity_type::holder_type_category(&e.entity.entity_type)
                == Some(category)
        },
        max_dist,
    )
}

/// `BehaviorBuilder.triggerIf(predicate)` — point, true пока
/// предикат выполняется.
pub fn trigger_if<F>(pred: F) -> PointBehavior
where
    F: Fn(&Arc<dyn LevelReader>, &mut Mob) -> bool + 'static,
{
    PointBehavior::point(move |level, mob, _t| pred(level, mob))
}

/// `BehaviorBuilder.sequence(predicateTrigger, trigger)` —
/// OneShot: latch при первом `true` предиката, далее делегирует
/// внутреннему trigger (`ifTriggered`-семантика).
pub fn sequence_point<F, G>(pred: F, mut inner: G) -> PointBehavior
where
    F: Fn(&Arc<dyn LevelReader>, &mut Mob) -> bool + 'static,
    G: FnMut(&Arc<dyn LevelReader>, &mut Mob, i64) -> bool + 'static,
{
    let latched = Rc::new(RefCell::new(false));
    let l2 = latched.clone();
    PointBehavior::point(move |level, mob, game_time| {
        if !*l2.borrow() && pred(level, mob) {
            *l2.borrow_mut() = true;
        }
        if *l2.borrow() {
            // Внутренний trigger вызывается как Trigger.trigger —
            // без tryStart-обёртки PointBehavior.
            // `inner` уже замкнут на нужную логику.
            // NB: вызываем напрямую — см. `sequence_trigger` ниже
            // для PointBehavior-варианта.
            inner(level, mob, game_time)
        } else {
            false
        }
    })
}

/// `BehaviorBuilder.sequence(triggerIf(pred), oneShot)` —
/// вариант, где внутренний — `PointBehavior` (его trigger
/// вызывается напрямую, как `Trigger.trigger` в Java).
pub fn sequence_with_point<F>(pred: F, inner: &PointBehavior) -> PointBehavior
where
    F: Fn(&Arc<dyn LevelReader>, &mut Mob) -> bool + 'static,
{
    let latched = Rc::new(RefCell::new(false));
    let l2 = latched.clone();
    let trigger = inner.trigger_rc();
    PointBehavior::point(move |level, mob, game_time| {
        if !*l2.borrow() && pred(level, mob) {
            *l2.borrow_mut() = true;
        }
        if *l2.borrow() {
            (trigger.borrow_mut())(level, mob, game_time)
        } else {
            false
        }
    })
}

// ===================== CelebrateVillagersSurvivedRaid =====================

/// `CelebrateVillagersSurvivedRaid(minDuration, maxDuration)` —
/// жители празднуют после победы над рейдом: звуки и
/// фейерверки в открытом небе.
pub fn celebrate_villagers_survived_raid(
    min_duration: i32,
    max_duration: i32,
) -> impl BehaviorControl {
    struct CelebrateVillagersSurvivedRaid {
        base: BehaviorBase,
        /// `currentRaid`.
        current_raid: Option<Rc<RefCell<crate::world::entity::raid::Raid>>>,
    }
    impl BehaviorControl for CelebrateVillagersSurvivedRaid {
        fn status(&self) -> BehaviorStatus {
            self.base.status
        }
        fn set_status(&mut self, s: BehaviorStatus) {
            self.base.status = s;
        }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) {
            (self.base.min_duration, self.base.max_duration)
        }
        fn end_timestamp(&self) -> i64 {
            self.base.end_timestamp
        }
        fn set_end_timestamp(&mut self, t: i64) {
            self.base.end_timestamp = t;
        }
        fn debug_string(&self) -> String {
            "CelebrateVillagersSurvivedRaid".to_string()
        }

        fn check_extra_start_conditions(
            &mut self,
            level: &Arc<dyn LevelReader>,
            mob: &mut Mob,
        ) -> bool {
            let pos = mob.living.entity.block_position();
            self.current_raid = level.raid_at(pos);
            self.current_raid
                .as_ref()
                .map(|r| {
                    r.borrow().is_victory()
                        && has_no_blocks_above(level, mob, pos)
                })
                .unwrap_or(false)
        }

        fn can_still_use(
            &mut self,
            _level: &Arc<dyn LevelReader>,
            _mob: &mut Mob,
            _game_time: i64,
        ) -> bool {
            self.current_raid
                .as_ref()
                .map(|r| !r.borrow().is_stopped())
                .unwrap_or(false)
        }

        fn start(&mut self, _level: &Arc<dyn LevelReader>, _mob: &mut Mob, _game_time: i64) {}

        fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            let mut r = mob.random.borrow_mut();
            if r.next_int_bound(100) == 0 {
                // `playCelebrateSound` — VILLAGER_CELEBRATE.
                level.play_sound(
                    mob.living.entity.position(),
                    &Identifier::minecraft("entity.villager.celebrate"),
                    1.0,
                    1.0,
                );
            }
            if r.next_int_bound(200) == 0
                && has_no_blocks_above(level, mob, mob.living.entity.block_position())
            {
                let color = DyeColor::VALUES[r.next_int_bound(16) as usize];
                let flight_time = r.next_int_bound(3);
                let firework = get_firework(color, flight_time);
                // `Projectile.spawnProjectile(FireworkRocketEntity)`.
                let mut fw = crate::world::entity::firework_rocket_entity::FireworkRocketEntity::new(
                    Some(level.clone()),
                    Some(&mob.living.entity),
                    mob.living.entity.pos.x,
                    mob.living.entity.get_eye_y(),
                    mob.living.entity.pos.z,
                    firework,
                );
                fw.entity
                    .set_delta_movement(0.0, 0.0, 0.0);
                level.add_fresh_firework_rocket(fw);
            }
        }

        fn stop(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            self.current_raid = None;
            if let Some(brain) = brain_of(mob) {
                brain.update_activity_from_schedule(
                    level.day_time(),
                    mob.living.entity.position(),
                    level.as_ref(),
                );
            }
        }
    }
    CelebrateVillagersSurvivedRaid {
        base: BehaviorBase::with_range(conds(&[]), min_duration, max_duration),
        current_raid: None,
    }
}

/// `getFirework(color, flightTime)` — `FIREWORK_ROCKET` с
/// `FIREWORKS(BURST, [color])`.
fn get_firework(color: DyeColor, flight_time: i32) -> ItemStack {
    let mut stack = item_stack_of(&Identifier::minecraft("firework_rocket"), 1);
    if stack.is_empty() {
        return stack;
    }
    stack.set_component(
        &Identifier::minecraft("fireworks"),
        Some(Fireworks::new(
            flight_time as i8 as i32,
            vec![FireworkExplosion {
                shape: FireworkExplosionShape::BURST,
                colors: vec![color.firework_color()],
                fade_colors: Vec::new(),
                has_trail: false,
                has_twinkle: false,
            }],
        )),
    );
    stack
}

// ===================== GiveGiftToHero =====================

/// `GiveGiftToHero(duration)` — герой деревни получает подарок
/// из лут-таблицы профессии.
pub fn give_gift_to_hero(duration: i32) -> impl BehaviorControl {
    struct GiveGiftToHero {
        base: BehaviorBase,
        /// `timeUntilNextGift` — старт 600.
        time_until_next_gift: i32,
        /// `giftGivenDuringThisRun`.
        gift_given: bool,
        /// `timeSinceStart`.
        time_since_start: i64,
    }
    impl BehaviorControl for GiveGiftToHero {
        fn status(&self) -> BehaviorStatus {
            self.base.status
        }
        fn set_status(&mut self, s: BehaviorStatus) {
            self.base.status = s;
        }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) {
            (self.base.min_duration, self.base.max_duration)
        }
        fn end_timestamp(&self) -> i64 {
            self.base.end_timestamp
        }
        fn set_end_timestamp(&mut self, t: i64) {
            self.base.end_timestamp = t;
        }
        fn debug_string(&self) -> String {
            "GiveGiftToHero".to_string()
        }

        fn check_extra_start_conditions(
            &mut self,
            _level: &Arc<dyn LevelReader>,
            mob: &mut Mob,
        ) -> bool {
            if nearest_hero(mob).is_none() {
                return false;
            }
            if self.time_until_next_gift > 0 {
                self.time_until_next_gift -= 1;
                return false;
            }
            true
        }

        fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
            self.gift_given = false;
            self.time_since_start = game_time;
            if let Some(player) = nearest_hero(mob) {
                mob.living.brain_set_value(&keys::interaction_target(), player.clone());
                behavior_utils::look_at_entity(mob, &player);
            }
        }

        fn can_still_use(
            &mut self,
            _level: &Arc<dyn LevelReader>,
            mob: &mut Mob,
            _game_time: i64,
        ) -> bool {
            nearest_hero(mob).is_some() && !self.gift_given
        }

        fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, game_time: i64) {
            let Some(player) = nearest_hero(mob) else {
                return;
            };
            behavior_utils::look_at_entity(mob, &player);
            let hero_pos = player.entity.block_position();
            let own_pos = mob.living.entity.block_position();
            if own_pos.get_center().distance_to(&hero_pos.get_center()) < 5.0 {
                if game_time - self.time_since_start > 20 {
                    self.throw_gift(level, mob, &player);
                    self.gift_given = true;
                }
            } else {
                behavior_utils::set_walk_and_look_target_memories_entity(
                    mob, &player, 0.5, 5,
                );
            }
        }

        fn stop(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            self.time_until_next_gift = 600 + level.level_random_next_int(6001);
            mob.living.brain_erase(&keys::interaction_target());
            mob.living.brain_erase(&keys::walk_target());
            mob.living.brain_erase(&keys::look_target());
        }
    }
    impl GiveGiftToHero {
        /// `throwGift` — дроп из gift-таблицы и бросок к герою.
        fn throw_gift(&self, level: &Arc<dyn LevelReader>, mob: &mut Mob, target: &LivingEntity) {
            let table = get_loot_table_to_throw(mob);
            // `dropFromGiftLootTable` — LootParams(ORIGIN, THIS_ENTITY).
            use crate::world::level::storage::loot::loot_context::LootParams;
            use crate::world::level::storage::loot::parameters::loot_context_params as params;
            let loot_params = LootParams::new()
                .with_parameter(params::origin(), mob.living.entity.pos)
                .with_parameter(params::this_entity(), mob.living.entity.clone());
            let items = level.generate_loot(&table, &loot_params);
            for stack in items {
                if !stack.is_empty() {
                    throw_item(&mob.living, stack, target.entity.position());
                }
            }
        }
    }
    GiveGiftToHero {
        base: BehaviorBase::with_duration(
            conds(&[
                (keys::walk_target(), MemoryStatus::Registered),
                (keys::look_target(), MemoryStatus::Registered),
                (keys::interaction_target(), MemoryStatus::Registered),
                (keys::nearest_visible_player(), MemoryStatus::ValuePresent),
            ]),
            duration,
        ),
        time_until_next_gift: 600,
        gift_given: false,
        time_since_start: 0,
    }
}

/// `getLootTableToThrow(villager)` — baby → baby_gift;
/// иначе профессия → `gameplay/hero_of_the_village/{prof}_gift`;
/// без профессии → `unemployed_gift`.
fn get_loot_table_to_throw(mob: &Mob) -> Identifier {
    if mob.living.entity.is_baby() {
        return Identifier::minecraft("gameplay/hero_of_the_village/baby_gift");
    }
    let Some(data) = mob.villager_data.as_ref() else {
        return Identifier::minecraft("gameplay/hero_of_the_village/unemployed_gift");
    };
    let Some(prof_key) = data.profession.key() else {
        return Identifier::minecraft("gameplay/hero_of_the_village/unemployed_gift");
    };
    let name = prof_key.identifier.path.to_string();
    // Все keyed-профессии → `{name}_gift` (включая nitwit —
    // в ванили есть `nitwit_gift` loot table).
    Identifier::minecraft(&format!("gameplay/hero_of_the_village/{name}_gift"))
}

// ===================== GoAndGiveItemsToTarget =====================

/// `GoAndGiveItemsToTarget(targetPositionGetter, speedModifier,
/// duration)` — донести предмет из инвентаря до цели
/// (allay/villager). `E` = `Mob` с инвентарём.
pub fn go_and_give_items_to_target(
    target_position_getter: Rc<dyn Fn(&Mob) -> Option<Rc<dyn PositionTracker>> + Send + Sync>,
    speed_modifier: f32,
    duration: i32,
) -> impl BehaviorControl {
    struct GoAndGiveItemsToTarget {
        base: BehaviorBase,
        getter: Rc<dyn Fn(&Mob) -> Option<Rc<dyn PositionTracker>> + Send + Sync>,
        speed_modifier: f32,
    }
    impl GoAndGiveItemsToTarget {
        /// `canThrowItemToTarget`.
        fn can_throw(&self, mob: &Mob) -> bool {
            let has_items = mob
                .get_inventory_ref()
                .map(|c| !c.is_empty())
                .unwrap_or(false);
            has_items && (self.getter)(mob).is_some()
        }
    }
    impl BehaviorControl for GoAndGiveItemsToTarget {
        fn status(&self) -> BehaviorStatus {
            self.base.status
        }
        fn set_status(&mut self, s: BehaviorStatus) {
            self.base.status = s;
        }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) {
            (self.base.min_duration, self.base.max_duration)
        }
        fn end_timestamp(&self) -> i64 {
            self.base.end_timestamp
        }
        fn set_end_timestamp(&mut self, t: i64) {
            self.base.end_timestamp = t;
        }
        fn debug_string(&self) -> String {
            "GoAndGiveItemsToTarget".to_string()
        }

        fn check_extra_start_conditions(
            &mut self,
            _level: &Arc<dyn LevelReader>,
            mob: &mut Mob,
        ) -> bool {
            self.can_throw(mob)
        }

        fn can_still_use(
            &mut self,
            _level: &Arc<dyn LevelReader>,
            mob: &mut Mob,
            _game_time: i64,
        ) -> bool {
            self.can_throw(mob)
        }

        fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            if let Some(pos) = (self.getter)(mob) {
                behavior_utils::set_walk_and_look_target_memories_tracker(
                    mob,
                    pos,
                    self.speed_modifier,
                    3,
                );
            }
        }

        fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            let Some(tracker) = (self.getter)(mob) else {
                return;
            };
            let d = tracker
                .current_position()
                .distance_to(&mob.living.entity.get_eye_position());
            if d < 3.0 {
                let stack = mob.get_inventory().remove_item(0, 1);
                if !stack.is_empty() {
                    // `getThrowPosition` = currentPosition + (0,1,0).
                    let throw_pos = tracker.current_position().add(0.0, 1.0, 0.0);
                    throw_item_with(
                        &mob.living,
                        stack.clone(),
                        throw_pos,
                        Vec3::new(0.2, 0.3, 0.2),
                        0.2,
                    );
                    // AllayAi.getLikedPlayer → CriteriaTriggers —
                    // allay-ветка: критерий требует ServerPlayer,
                    // недоступен в порте (как и TradeWithVillager).
                    let _ = stack;
                    mob.living
                        .brain_set_value(&keys::item_pickup_cooldown_ticks(), 60);
                }
            }
            let _ = level;
        }
    }
    GoAndGiveItemsToTarget {
        base: BehaviorBase::with_duration(
            conds(&[
                (keys::look_target(), MemoryStatus::Registered),
                (keys::walk_target(), MemoryStatus::Registered),
                (keys::item_pickup_cooldown_ticks(), MemoryStatus::Registered),
            ]),
            duration,
        ),
        getter: target_position_getter,
        speed_modifier,
    }
}

// ===================== ShowTradesToPlayer =====================

/// `ShowTradesToPlayer(minDuration, maxDuration)` — показ
/// предметов трейда в руке жителя.
pub fn show_trades_to_player(min_duration: i32, max_duration: i32) -> impl BehaviorControl {
    struct ShowTradesToPlayer {
        base: BehaviorBase,
        /// `playerItemStack`.
        player_item_stack: Option<ItemStack>,
        /// `displayItems`.
        display_items: Vec<ItemStack>,
        /// `cycleCounter`.
        cycle_counter: i32,
        /// `displayIndex`.
        display_index: usize,
        /// `lookTime`.
        look_time: i32,
    }
    impl ShowTradesToPlayer {
        /// `lookAtTarget` — LOOK_TARGET = INTERACTION_TARGET.
        fn look_at_target(&self, mob: &mut Mob) -> Option<LivingEntity> {
            let target: Option<LivingEntity> =
                mob.living.brain_get(&keys::interaction_target());
            if let Some(t) = &target {
                mob.living.brain_set_value(
                    &keys::look_target(),
                    Rc::new(EntityTracker::new(t.entity.clone(), true))
                        as Rc<dyn PositionTracker>,
                );
            }
            target
        }

        /// `checkExtraStartConditions`.
        fn extra_conditions(&self, mob: &Mob) -> bool {
            let Some(target) = mob
                .living
                .brain_get::<LivingEntity>(&keys::interaction_target())
            else {
                return false;
            };
            entity_type_path(&target.entity) == "player"
                && mob.living.entity.is_alive()
                && target.entity.is_alive()
                && !mob.living.entity.is_baby()
                && mob.living.distance_to_sqr(&target.entity) <= 17.0
        }

        /// `findItemsToDisplay`.
        fn find_items_to_display(&mut self, target: &LivingEntity, mob: &mut Mob) {
            let main_hand = target.get_main_hand_item().clone();
            let mut flag = false;
            match &self.player_item_stack {
                None => {
                    self.player_item_stack = Some(main_hand.clone());
                    flag = true;
                    self.display_items.clear();
                }
                Some(prev) => {
                    if !ItemStack::is_same_item(prev, &main_hand) {
                        self.player_item_stack = Some(main_hand.clone());
                        flag = true;
                        self.display_items.clear();
                    }
                }
            }
            if flag && self.player_item_stack.as_ref().map(|s| !s.is_empty()).unwrap_or(false) {
                self.update_display_items(mob);
                if !self.display_items.is_empty() {
                    self.look_time = 900;
                    self.display_first_item(mob);
                }
            }
        }

        /// `updateDisplayItems`.
        fn update_display_items(&mut self, mob: &Mob) {
            let Some(offers) = mob.get_offers() else {
                return;
            };
            for offer in &offers.offers {
                if !offer.is_out_of_stock()
                    && self.player_item_stack_matches_cost(offer)
                {
                    self.display_items.push(offer.assemble());
                }
            }
        }

        /// `playerItemStackMatchesCostOfOffer`.
        fn player_item_stack_matches_cost(&self, offer: &MerchantOffer) -> bool {
            let Some(stack) = &self.player_item_stack else {
                return false;
            };
            ItemStack::is_same_item(stack, &offer.get_cost_a())
                || ItemStack::is_same_item(stack, &offer.get_cost_b())
        }

        /// `displayFirstItem`.
        fn display_first_item(&self, mob: &mut Mob) {
            if let Some(first) = self.display_items.first() {
                display_as_held_item(mob, first.clone());
            }
        }

        /// `displayCyclingItems`.
        fn display_cycling_items(&mut self, mob: &mut Mob) {
            if self.display_items.len() >= 2 {
                self.cycle_counter += 1;
                if self.cycle_counter >= 40 {
                    self.cycle_counter = 0;
                    self.display_index += 1;
                    if self.display_index > self.display_items.len() - 1 {
                        self.display_index = 0;
                    }
                    if let Some(item) = self.display_items.get(self.display_index) {
                        display_as_held_item(mob, item.clone());
                    }
                }
            }
        }
    }
    impl BehaviorControl for ShowTradesToPlayer {
        fn status(&self) -> BehaviorStatus {
            self.base.status
        }
        fn set_status(&mut self, s: BehaviorStatus) {
            self.base.status = s;
        }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) {
            (self.base.min_duration, self.base.max_duration)
        }
        fn end_timestamp(&self) -> i64 {
            self.base.end_timestamp
        }
        fn set_end_timestamp(&mut self, t: i64) {
            self.base.end_timestamp = t;
        }
        fn debug_string(&self) -> String {
            "ShowTradesToPlayer".to_string()
        }

        fn check_extra_start_conditions(
            &mut self,
            _level: &Arc<dyn LevelReader>,
            mob: &mut Mob,
        ) -> bool {
            self.extra_conditions(mob)
        }

        fn can_still_use(
            &mut self,
            level: &Arc<dyn LevelReader>,
            mob: &mut Mob,
            _game_time: i64,
        ) -> bool {
            // Purpur `villagerDisplayTradeItem` — конфиг не портирован,
            // хук уровня даёт vanilla-true.
            level.purpur_villager_display_trade_item()
                && self.extra_conditions(mob)
                && self.look_time > 0
                && mob
                    .living
                    .brain_has(&keys::interaction_target())
        }

        fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            self.look_at_target(mob);
            self.cycle_counter = 0;
            self.display_index = 0;
            self.look_time = 40;
        }

        fn tick(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            let Some(target) = self.look_at_target(mob) else {
                return;
            };
            self.find_items_to_display(&target, mob);
            if !self.display_items.is_empty() {
                self.display_cycling_items(mob);
            } else {
                clear_held_item(mob);
                self.look_time = self.look_time.min(40);
            }
            self.look_time -= 1;
        }

        fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            mob.living.brain_erase(&keys::interaction_target());
            clear_held_item(mob);
            self.player_item_stack = None;
        }
    }
    ShowTradesToPlayer {
        base: BehaviorBase::with_range(
            conds(&[(keys::interaction_target(), MemoryStatus::ValuePresent)]),
            min_duration,
            max_duration,
        ),
        player_item_stack: None,
        display_items: Vec::new(),
        cycle_counter: 0,
        display_index: 0,
        look_time: 0,
    }
}

/// `clearHeldItem` — MAINHAND=EMPTY, dropChance=0.085.
fn clear_held_item(mob: &mut Mob) {
    mob.living
        .set_item_slot(EquipmentSlot::Mainhand, ItemStack::empty());
    mob.drop_chances
        .by_equipment
        .insert(EquipmentSlot::Mainhand, 0.085);
}

/// `displayAsHeldItem` — MAINHAND=item, dropChance=0.
fn display_as_held_item(mob: &mut Mob, item: ItemStack) {
    mob.living.set_item_slot(EquipmentSlot::Mainhand, item);
    mob.drop_chances.by_equipment.insert(EquipmentSlot::Mainhand, 0.0);
}

// ===================== PoiCompetitorScan =====================

/// `PoiCompetitorScan.create()` — при наличии JOB_SITE и
/// NLE сканирует соседних жителей: у всех с тем же JOB_SITE
/// и подходящей профессией остаётся один «победитель»
/// (максимальный villagerXp; при равенстве — более поздний).
pub fn poi_competitor_scan() -> PointBehavior {
    PointBehavior::new(
        conds(&[
            (keys::job_site(), MemoryStatus::ValuePresent),
            (keys::nearest_living_entities(), MemoryStatus::ValuePresent),
        ]),
        move |level, mob, _game_time| {
            let Some(gp): Option<GlobalPos> = mob.living.brain_get(&keys::job_site()) else {
                return true;
            };
            // `getTypeIfLoaded` — тип POI в точке.
            let poi_type: Option<Holder<PoiType>> = level
                .poi_manager()
                .and_then(|pm| pm.get_type(gp.pos));
            let Some(poi) = poi_type else {
                return true;
            };
            let entities: Vec<LivingEntity> = mob
                .living
                .brain_get(&keys::nearest_living_entities())
                .unwrap_or_default();
            let self_uuid = mob.living.entity.uuid;
            for other in &entities {
                if other.entity.uuid == self_uuid {
                    continue;
                }
                // `other instanceof Villager` — по типу сущности.
                if entity_type_path(&other.entity) != "villager" {
                    continue;
                }
                if !other.entity.is_alive() {
                    continue;
                }
                if !competes_for_same_jobsite(&gp, &poi, other) {
                    continue;
                }
                // `selectWinner(winner, other)`: xp недоступен
                // (поля на Mob, memory хранит LivingEntity) —
                // tie-семантика: поздний конкурент выигрывает,
                // текущий «winner» теряет JOB_SITE. Начальный
                // winner — сам моб.
                mob.living.brain_erase(&keys::job_site());
                // Если победитель — `other`, его JOB_SITE уже
                // есть; следующий конкурент снова сравнивается
                // с ним (xp-поле недоступно — tie).
                let _ = other;
            }
            true
        },
    )
}

/// `competesForSameJobsite` — JOB_SITE другого == нашему
/// и профессия совпадает (profession-проверка деградирует:
/// LivingEntity-клон не несёт villager_data — считаем
/// совпадением для villager-типа).
fn competes_for_same_jobsite(
    job_site_pos: &GlobalPos,
    _poi_type: &Holder<PoiType>,
    other: &LivingEntity,
) -> bool {
    let other_job: Option<GlobalPos> = other.brain_get(&keys::job_site());
    other_job
        .map(|gp| gp == *job_site_pos)
        .unwrap_or(false)
}

// ===================== StrollToPoiList =====================

/// `StrollToPoiList.create(poiListMemory, speedModifier,
/// closeEnoughDist, maxDistFromPoi, mustBeCloseToMemory)`.
pub fn stroll_to_poi_list(
    poi_list_memory: MemoryModuleType,
    speed_modifier: f32,
    close_enough_dist: i32,
    max_dist_from_poi: i32,
    must_be_close_to_memory: MemoryModuleType,
) -> PointBehavior {
    let last_update = Rc::new(RefCell::new(0i64));
    let lu = last_update.clone();
    PointBehavior::new(
        conds(&[
            (keys::walk_target(), MemoryStatus::Registered),
            (poi_list_memory.clone(), MemoryStatus::ValuePresent),
            (must_be_close_to_memory.clone(), MemoryStatus::ValuePresent),
        ]),
        move |level, mob, game_time| {
            let list: Vec<GlobalPos> = mob
                .living
                .brain_get(&poi_list_memory)
                .unwrap_or_default();
            let must_be_close_to: Option<GlobalPos> =
                mob.living.brain_get(&must_be_close_to_memory);
            if list.is_empty() {
                return false;
            }
            let Some(_required) = must_be_close_to else {
                return false;
            };
            let idx = level.level_random_next_int(list.len() as i32) as usize;
            let Some(gp) = list.get(idx) else {
                return false;
            };
            // `level.dimension() == gp.dimension` — ResourceKey eq.
            let dim = level
                .dimension_id()
                .cloned()
                .unwrap_or_else(|| Identifier::with_default_namespace("overworld").unwrap());
            if gp.dimension.identifier != dim {
                return false;
            }
            if !gp.pos.closer_to_center_than(
                mob.living.entity.position(),
                max_dist_from_poi as f64,
            ) {
                return false;
            }
            if game_time > *lu.borrow() {
                *lu.borrow_mut() = game_time + 100;
                mob.living.brain_set_value(
                    &keys::walk_target(),
                    WalkTarget::new_block(gp.pos, speed_modifier, close_enough_dist),
                );
            }
            true
        },
    )
}

// ===================== TransportItemsBetweenContainers =====================

/// `ContainerInteractionState`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ContainerInteractionState {
    /// `PICKUP_ITEM`.
    PickupItem,
    /// `PICKUP_NO_ITEM`.
    PickupNoItem,
    /// `PLACE_ITEM`.
    PlaceItem,
    /// `PLACE_NO_ITEM`.
    PlaceNoItem,
}

/// `TransportItemState`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportItemState {
    /// `TRAVELLING`.
    Travelling,
    /// `QUEUING`.
    Queuing,
    /// `INTERACTING`.
    Interacting,
}

/// `TransportItemTarget` record — pos + container + state.
/// `blockEntity` заменён на `container_key` (идентификатор
/// контейнера для `targetHasNotChanged` через ptr_eq).
#[derive(Clone)]
pub struct TransportItemTarget {
    /// `pos`.
    pub pos: BlockPos,
    /// `container`.
    pub container: Rc<RefCell<dyn crate::world::simple_container::Container>>,
    /// `state` — `blockEntity.getBlockState()`.
    pub state: crate::world::level::block::state::block_state::BlockState,
}

impl TransportItemTarget {
    /// `tryCreatePossibleTarget(blockEntity, level)` —
    /// контейнер уже получен из `level.container_at`.
    pub fn try_create(
        level: &Arc<dyn LevelReader>,
        pos: BlockPos,
        container: Rc<RefCell<dyn crate::world::simple_container::Container>>,
    ) -> Option<Self> {
        let state = level.get_block_state(pos)?;
        Some(Self {
            pos,
            container,
            state,
        })
    }
}

/// `TransportItemsBetweenContainers(speedModifier,
/// sourceBlockType, destinationBlockType, horizontalSearchDistance,
/// verticalSearchDistance, onTargetInteractionActions,
/// onStartTravelling, shouldQueueForTarget)`.
#[allow(clippy::too_many_arguments)]
pub fn transport_items_between_containers(
    speed_modifier: f32,
    source_block_type: Rc<dyn Fn(&crate::world::level::block::state::block_state::BlockState) -> bool + Send + Sync>,
    destination_block_type: Rc<dyn Fn(&crate::world::level::block::state::block_state::BlockState) -> bool + Send + Sync>,
    horizontal_search_distance: i32,
    vertical_search_distance: i32,
    on_target_interaction_actions: HashMap<
        ContainerInteractionState,
        Rc<dyn Fn(&mut Mob, &TransportItemTarget, i32) + Send + Sync>,
    >,
    on_start_travelling: Rc<dyn Fn(&mut Mob) + Send + Sync>,
    should_queue_for_target: Rc<dyn Fn(&TransportItemTarget) -> bool + Send + Sync>,
) -> impl BehaviorControl {
    struct TransportItemsBetweenContainers {
        base: BehaviorBase,
        speed_modifier: f32,
        horizontal_search_distance: i32,
        vertical_search_distance: i32,
        source_block_type: Rc<dyn Fn(&crate::world::level::block::state::block_state::BlockState) -> bool + Send + Sync>,
        destination_block_type: Rc<dyn Fn(&crate::world::level::block::state::block_state::BlockState) -> bool + Send + Sync>,
        should_queue_for_target: Rc<dyn Fn(&TransportItemTarget) -> bool + Send + Sync>,
        on_start_travelling: Rc<dyn Fn(&mut Mob) + Send + Sync>,
        on_target_interaction_actions: HashMap<
            ContainerInteractionState,
            Rc<dyn Fn(&mut Mob, &TransportItemTarget, i32) + Send + Sync>,
        >,
        /// `target`.
        target: Option<TransportItemTarget>,
        /// `state`.
        state: TransportItemState,
        /// `interactionState`.
        interaction_state: Option<ContainerInteractionState>,
        /// `ticksSinceReachingTarget`.
        ticks_since_reaching_target: i32,
    }

    impl TransportItemsBetweenContainers {
        /// `updateInvalidTarget`.
        fn update_invalid_target(
            &mut self,
            level: &Arc<dyn LevelReader>,
            mob: &mut Mob,
        ) -> bool {
            if !self.has_valid_target(level, mob) {
                self.stop_targeting_current_target(mob);
                if let Some(target) = self.get_transport_target(level, mob) {
                    self.target = Some(target);
                    (self.on_start_travelling)(mob);
                    if let Some(t) = &self.target {
                        self.set_visited_block_pos(mob, level, t.pos);
                    }
                    return true;
                }
                self.enter_cooldown_after_no_matching_target_found(mob);
                return true;
            }
            false
        }

        /// `onQueuingForTarget`.
        fn on_queuing_for_target(
            &mut self,
            target: &TransportItemTarget,
            level: &Arc<dyn LevelReader>,
            mob: &mut Mob,
        ) {
            if !self.is_another_mob_interacting_with_target(target, level) {
                self.resume_travelling(mob);
            }
        }

        /// `onTravelToTarget`.
        fn on_travel_to_target(
            &mut self,
            target: &TransportItemTarget,
            level: &Arc<dyn LevelReader>,
            mob: &mut Mob,
        ) {
            let center = self.get_center_pos(mob);
            if self.is_within_target_distance(3.0, target, level, mob, &center)
                && self.is_another_mob_interacting_with_target(target, level)
            {
                self.start_queuing(mob);
            } else if self.is_within_target_distance(
                get_interaction_range(mob),
                target,
                level,
                mob,
                &center,
            ) {
                self.start_on_reached_target_interaction(target.clone(), mob);
            } else {
                self.walk_towards_target(mob);
            }
        }

        /// `getCenterPos` = `setMiddleYPosition(mob, mob.position())`.
        fn get_center_pos(&self, mob: &Mob) -> Vec3 {
            set_middle_y_position(mob, mob.living.entity.position())
        }

        /// `onReachedTarget`.
        fn on_reached_target(
            &mut self,
            target: &TransportItemTarget,
            level: &Arc<dyn LevelReader>,
            mob: &mut Mob,
        ) {
            let center = self.get_center_pos(mob);
            if !self.is_within_target_distance(2.0, target, level, mob, &center) {
                (self.on_start_travelling)(mob);
                self.state = TransportItemState::Travelling;
                self.interaction_state = None;
                self.ticks_since_reaching_target = 0;
            } else {
                self.ticks_since_reaching_target += 1;
                self.on_target_interaction(target, mob);
                if self.ticks_since_reaching_target >= 60 {
                    let container = target.container.clone();
                    let state = self.probe_reached_state(mob, &container);
                    match state {
                        ContainerInteractionState::PickupItem => {
                            self.pick_up_items(mob, &container);
                        }
                        ContainerInteractionState::PickupNoItem => {
                            self.stop_targeting_current_target(mob);
                        }
                        ContainerInteractionState::PlaceItem => {
                            self.put_down_item(mob, &container);
                        }
                        ContainerInteractionState::PlaceNoItem => {
                            self.stop_targeting_current_target(mob);
                        }
                    }
                    (self.on_start_travelling)(mob);
                    self.state = TransportItemState::Travelling;
                    self.interaction_state = None;
                    self.ticks_since_reaching_target = 0;
                }
            }
        }

        /// `startQueuing`.
        fn start_queuing(&mut self, mob: &mut Mob) {
            self.stop_in_place(mob);
            self.state = TransportItemState::Queuing;
        }

        /// `resumeTravelling`.
        fn resume_travelling(&mut self, mob: &mut Mob) {
            self.state = TransportItemState::Travelling;
            self.walk_towards_target(mob);
        }

        /// `walkTowardsTarget`.
        fn walk_towards_target(&mut self, mob: &mut Mob) {
            if let Some(t) = &self.target {
                behavior_utils::set_walk_and_look_target_memories_pos(
                    mob,
                    t.pos,
                    self.speed_modifier,
                    0,
                );
            }
        }

        /// `startOnReachedTargetInteraction` — проба состояния
        /// и переход в INTERACTING.
        fn start_on_reached_target_interaction(
            &mut self,
            target: TransportItemTarget,
            mob: &mut Mob,
        ) {
            let state = self.probe_reached_state(mob, &target.container);
            self.interaction_state = Some(state);
            self.state = TransportItemState::Interacting;
            let _ = target;
        }

        /// `doReachedTargetInteraction` — решение, какое
        /// из 4 действий применить (PICKUP/PLACE × match).
        fn probe_reached_state(
            &self,
            mob: &Mob,
            container: &Rc<RefCell<dyn crate::world::simple_container::Container>>,
        ) -> ContainerInteractionState {
            if is_picking_up_items(mob) {
                if matches_getting_items_requirement(container) {
                    ContainerInteractionState::PickupItem
                } else {
                    ContainerInteractionState::PickupNoItem
                }
            } else if matches_leaving_items_requirement(mob, container) {
                ContainerInteractionState::PlaceItem
            } else {
                ContainerInteractionState::PlaceNoItem
            }
        }

        /// `onTargetInteraction` — LOOK_TARGET=BlockPosTracker,
        /// стоп на месте, вызов action по interactionState.
        fn on_target_interaction(&mut self, target: &TransportItemTarget, mob: &mut Mob) {
            mob.living.brain_set_value(
                &keys::look_target(),
                Rc::new(BlockPosTracker::new(target.pos))
                    as Rc<dyn PositionTracker>,
            );
            self.stop_in_place(mob);
            if let Some(state) = self.interaction_state {
                if let Some(f) = self.on_target_interaction_actions.get(&state) {
                    f(mob, target, self.ticks_since_reaching_target);
                }
            }
        }

        /// `hasValidTarget`.
        fn has_valid_target(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob) -> bool {
            let Some(target) = self.target.clone() else {
                return false;
            };
            let flag = self.is_wanted_block(mob, &target.state)
                && self.target_has_not_changed(level, &target);
            if flag && !self.is_target_blocked(level, &target) {
                if self.state != TransportItemState::Travelling {
                    return true;
                }
                if self.has_valid_travelling_path(level, &target, mob) {
                    return true;
                }
                self.mark_visited_block_pos_as_unreachable(mob, level, target.pos);
            }
            false
        }

        /// `hasValidTravellingPath`.
        fn has_valid_travelling_path(
            &mut self,
            level: &Arc<dyn LevelReader>,
            target: &TransportItemTarget,
            mob: &mut Mob,
        ) -> bool {
            // `path = getPath()==null ? createPath(pos,0) : getPath()`.
            let nav = mob.get_navigation();
            let existing_end: Option<Option<BlockPos>> = nav.as_ref().and_then(|n| {
                let b = n.borrow();
                b.path().map(|p| {
                    p.end_node().map(|node| node.borrow().as_block_pos())
                })
            });
            let (has_path, end_node_pos): (bool, Option<BlockPos>) = match existing_end {
                Some(end) => (true, end),
                None => {
                    if let Some(n) = &nav {
                        if let Some(p) = n.borrow_mut().create_path_pos(mob, target.pos, 0) {
                            let end = p
                                .end_node()
                                .map(|node| node.borrow().as_block_pos());
                            (true, end)
                        } else {
                            (false, None)
                        }
                    } else {
                        (false, None)
                    }
                }
            };
            let _ = level;
            let position_to_reach = self.get_position_to_reach_target_from(has_path, end_node_pos, mob);
            let within = self.is_within_target_distance(
                get_interaction_range(mob),
                target,
                level,
                mob,
                &position_to_reach,
            );
            let no_path_and_far = !has_path && !within;
            no_path_and_far
                || self.target_is_reachable_from_position(level, within, &position_to_reach, target, mob)
        }

        /// `getPositionToReachTargetFrom`.
        fn get_position_to_reach_target_from(
            &self,
            has_path: bool,
            end_node_pos: Option<BlockPos>,
            mob: &Mob,
        ) -> Vec3 {
            let vec3 = if !has_path {
                mob.living.entity.position()
            } else {
                match end_node_pos {
                    None => mob.living.entity.position(),
                    Some(bp) => Vec3::new(
                        bp.x as f64 + 0.5,
                        bp.y as f64,
                        bp.z as f64 + 0.5,
                    ),
                }
            };
            set_middle_y_position(mob, vec3)
        }

        /// `setMiddleYPosition`.
        // (функция `set_middle_y_position` ниже)

        /// `isTargetBlocked` — barrel/shulker/chest-blocked.
        /// Purpur-флаги не портированы → vanilla-ветки:
        /// chest blocked по твёрдому блоку сверху.
        fn is_target_blocked(
            &self,
            level: &Arc<dyn LevelReader>,
            target: &TransportItemTarget,
        ) -> bool {
            let id = block_state_id(&target.state);
            if id.ends_with("barrel") || id.ends_with("shulker_box") {
                return false; // Purpur-открытие не портировано — не блокируем.
            }
            // `ChestBlock.isChestBlockedAt` — твёрдый блок над сундуком.
            if id.contains("chest") {
                if let Some(above) = level.get_block_state(target.pos.above()) {
                    return block_state_is_solid(&above);
                }
            }
            false
        }

        /// `targetHasNotChanged` — blockEntity.equals(getBlockEntity).
        fn target_has_not_changed(
            &self,
            level: &Arc<dyn LevelReader>,
            target: &TransportItemTarget,
        ) -> bool {
            level
                .container_at(target.pos)
                .map(|c| Rc::ptr_eq(&c, &target.container))
                .unwrap_or(false)
        }

        /// `getConnectedTargets` — double-chest соседи.
        fn get_connected_targets(
            &self,
            target: &TransportItemTarget,
            level: &Arc<dyn LevelReader>,
        ) -> Vec<TransportItemTarget> {
            // `ChestBlock.TYPE != SINGLE` → connected pos.
            if let Some(connected_pos) = chest_connected_pos(&target.state, target.pos) {
                if let Some(container) = level.container_at(connected_pos) {
                    if let Some(t2) =
                        TransportItemTarget::try_create(level, connected_pos, container)
                    {
                        return vec![target.clone(), t2];
                    }
                }
            }
            vec![target.clone()]
        }

        /// `getTargetSearchArea`.
        fn get_target_search_area(&self, mob: &Mob) -> Aabb {
            let h = self.get_horizontal_search_distance(mob);
            block_aabb(mob.living.entity.block_position())
                .inflate(h as f64, self.get_vertical_search_distance(mob) as f64, h as f64)
        }

        /// `getHorizontalSearchDistance`.
        fn get_horizontal_search_distance(&self, mob: &Mob) -> i32 {
            if mob.living.entity.is_passenger() {
                1
            } else {
                self.horizontal_search_distance
            }
        }

        /// `getVerticalSearchDistance`.
        fn get_vertical_search_distance(&self, mob: &Mob) -> i32 {
            if mob.living.entity.is_passenger() {
                1
            } else {
                self.vertical_search_distance
            }
        }

        /// `getVisitedPositions`.
        fn visited_positions(mob: &Mob) -> HashSet<GlobalPos> {
            mob.living
                .brain_get(&keys::visited_block_positions())
                .unwrap_or_default()
        }

        /// `getUnreachablePositions`.
        fn unreachable_positions(mob: &Mob) -> HashSet<GlobalPos> {
            mob.living
                .brain_get(&keys::unreachable_transport_block_positions())
                .unwrap_or_default()
        }

        /// `isPositionAlreadyVisited`.
        fn is_position_already_visited(
            &self,
            visited: &HashSet<GlobalPos>,
            unreachable: &HashSet<GlobalPos>,
            target: &TransportItemTarget,
            level: &Arc<dyn LevelReader>,
        ) -> bool {
            self.get_connected_targets(target, level).iter().any(|t| {
                let gp = global_pos_of(level, t.pos);
                visited.contains(&gp) || unreachable.contains(&gp)
            })
        }

        /// `setVisitedBlockPos`.
        fn set_visited_block_pos(
            &mut self,
            mob: &mut Mob,
            level: &Arc<dyn LevelReader>,
            pos: BlockPos,
        ) {
            let mut set = Self::visited_positions(mob);
            set.insert(global_pos_of(level, pos));
            if set.len() > 10 {
                self.enter_cooldown_after_no_matching_target_found(mob);
            } else {
                mob.living
                    .brain_set_with_expiry(&keys::visited_block_positions(), set, 6000);
            }
        }

        /// `markVisitedBlockPosAsUnreachable`.
        fn mark_visited_block_pos_as_unreachable(
            &mut self,
            mob: &mut Mob,
            level: &Arc<dyn LevelReader>,
            pos: BlockPos,
        ) {
            let mut set = Self::visited_positions(mob);
            set.remove(&global_pos_of(level, pos));
            let mut set1 = Self::unreachable_positions(mob);
            set1.insert(global_pos_of(level, pos));
            if set1.len() > 50 {
                self.enter_cooldown_after_no_matching_target_found(mob);
            } else {
                mob.living
                    .brain_set_with_expiry(&keys::visited_block_positions(), set, 6000);
                mob.living.brain_set_with_expiry(
                    &keys::unreachable_transport_block_positions(),
                    set1,
                    6000,
                );
            }
        }

        /// `isWantedBlock`.
        fn is_wanted_block(
            &self,
            mob: &Mob,
            state: &crate::world::level::block::state::block_state::BlockState,
        ) -> bool {
            if is_picking_up_items(mob) {
                (self.source_block_type)(state)
            } else {
                (self.destination_block_type)(state)
            }
        }

        /// `isWithinTargetDistance` — collisionShape bounds
        /// аппроксимирован unit-bbox блока (collision shape
        /// не портирован).
        fn is_within_target_distance(
            &self,
            distance: f64,
            target: &TransportItemTarget,
            _level: &Arc<dyn LevelReader>,
            mob: &Mob,
            pos: &Vec3,
        ) -> bool {
            let bb = mob.living.entity.get_bounding_box();
            let aabb = Aabb::of_size(pos, bb.get_xsize(), bb.get_ysize(), bb.get_zsize());
            block_aabb(target.pos)
                .inflate(distance, 0.5, distance)
                .intersects(&aabb)
        }

        /// `targetIsReachableFromPosition`.
        fn target_is_reachable_from_position(
            &self,
            level: &Arc<dyn LevelReader>,
            within_distance: bool,
            pos: &Vec3,
            target: &TransportItemTarget,
            mob: &Mob,
        ) -> bool {
            within_distance && self.can_see_any_target_side(target, level, mob, pos)
        }

        /// `canSeeAnyTargetSide` — clip до любой стороны блока.
        fn can_see_any_target_side(
            &self,
            target: &TransportItemTarget,
            level: &Arc<dyn LevelReader>,
            _mob: &Mob,
            pos: &Vec3,
        ) -> bool {
            let center = Vec3::new(
                target.pos.x as f64 + 0.5,
                target.pos.y as f64 + 0.5,
                target.pos.z as f64 + 0.5,
            );
            Direction::VALUES.iter().any(|dir| {
                let side = Vec3::new(
                    center.x + 0.5 * dir.get_step_x() as f64,
                    center.y + 0.5 * dir.get_step_y() as f64,
                    center.z + 0.5 * dir.get_step_z() as f64,
                );
                level
                    .clip_block_hit(*pos, side)
                    .map(|hit| hit == target.pos)
                    .unwrap_or(false)
            })
        }

        /// `isAnotherMobInteractingWithTarget`.
        fn is_another_mob_interacting_with_target(
            &self,
            target: &TransportItemTarget,
            level: &Arc<dyn LevelReader>,
        ) -> bool {
            self.get_connected_targets(target, level)
                .iter()
                .any(|t| (self.should_queue_for_target)(t))
        }

        /// `getTransportTarget` — поиск ближайшего контейнера.
        /// `chunk.getBlockEntities()` заменён хуком
        /// `level.containers_in_area` (эквивалент скану чанков).
        fn get_transport_target(
            &mut self,
            level: &Arc<dyn LevelReader>,
            mob: &Mob,
        ) -> Option<TransportItemTarget> {
            let search_area = self.get_target_search_area(mob);
            let visited = Self::visited_positions(mob);
            let unreachable = Self::unreachable_positions(mob);
            let h = self.get_horizontal_search_distance(mob);
            let v = self.get_vertical_search_distance(mob);
            let center = mob.living.entity.block_position();
            let min = BlockPos::new(center.x - h, center.y - v, center.z - h);
            let max = BlockPos::new(center.x + h, center.y + v, center.z + h);
            let mut best: Option<TransportItemTarget> = None;
            let mut best_d = f64::MAX;
            for (pos, container) in level.containers_in_area(min, max) {
                // `searchArea.contains(x,y,z)`.
                if !aabb_contains_pos(&search_area, pos) {
                    continue;
                }
                let mob_pos = mob.living.entity.position();
                let center = pos.get_center();
                let dx = center.x - mob_pos.x;
                let dy = center.y - mob_pos.y;
                let dz = center.z - mob_pos.z;
                let d = dx * dx + dy * dy + dz * dz;
                if d >= best_d {
                    continue;
                }
                if let Some(t) =
                    TransportItemTarget::try_create(level, pos, container)
                {
                    if self.is_wanted_block(mob, &t.state)
                        && !self.is_position_already_visited(
                            &visited, &unreachable, &t, level,
                        )
                        && !t.container.borrow().is_locked()
                    {
                        best = Some(t);
                        best_d = d;
                    }
                }
            }
            best
        }

        /// `pickUpItems`.
        fn pick_up_items(
            &mut self,
            mob: &mut Mob,
            container: &Rc<RefCell<dyn crate::world::simple_container::Container>>,
        ) {
            let stack = pick_up_item_from_container(container);
            mob.living
                .set_item_slot(EquipmentSlot::Mainhand, stack);
            mob.drop_chances = mob
                .drop_chances
                .with_guaranteed_drop(EquipmentSlot::Mainhand);
            container.borrow_mut().set_changed();
            self.clear_memories_after_matching_target_found(mob);
        }

        /// `putDownItem`.
        fn put_down_item(
            &mut self,
            mob: &mut Mob,
            container: &Rc<RefCell<dyn crate::world::simple_container::Container>>,
        ) {
            let remaining = add_items_to_container(mob, container);
            container.borrow_mut().set_changed();
            mob.living
                .set_item_slot(EquipmentSlot::Mainhand, remaining);
            if mob
                .living
                .get_main_hand_item()
                .is_empty()
            {
                self.clear_memories_after_matching_target_found(mob);
            } else {
                self.stop_targeting_current_target(mob);
            }
        }

        /// `stopTargetingCurrentTarget`.
        fn stop_targeting_current_target(&mut self, mob: &mut Mob) {
            self.ticks_since_reaching_target = 0;
            self.target = None;
            if let Some(nav) = mob.get_navigation() {
                nav.borrow_mut().stop(mob);
            }
            mob.living.brain_erase(&keys::walk_target());
        }

        /// `clearMemoriesAfterMatchingTargetFound`.
        fn clear_memories_after_matching_target_found(&mut self, mob: &mut Mob) {
            self.stop_targeting_current_target(mob);
            mob.living.brain_erase(&keys::visited_block_positions());
            mob.living
                .brain_erase(&keys::unreachable_transport_block_positions());
        }

        /// `enterCooldownAfterNoMatchingTargetFound`.
        fn enter_cooldown_after_no_matching_target_found(&mut self, mob: &mut Mob) {
            self.stop_targeting_current_target(mob);
            mob.living.brain_set_value(
                &keys::transport_items_cooldown_ticks(),
                140,
            );
            mob.living.brain_erase(&keys::visited_block_positions());
            mob.living
                .brain_erase(&keys::unreachable_transport_block_positions());
        }

        /// `stopInPlace`.
        fn stop_in_place(&self, mob: &mut Mob) {
            if let Some(nav) = mob.get_navigation() {
                nav.borrow_mut().stop(mob);
            }
            mob.living.set_xxa(0.0);
            mob.living.set_yya(0.0);
            mob.living.set_speed(0.0);
            let dy = mob.living.entity.get_delta_movement().y;
            mob.living.entity.set_delta_movement(0.0, dy, 0.0);
        }
    }

    impl BehaviorControl for TransportItemsBetweenContainers {
        fn status(&self) -> BehaviorStatus {
            self.base.status
        }
        fn set_status(&mut self, s: BehaviorStatus) {
            self.base.status = s;
        }
        fn entry_conditions(&self) -> &HashMap<MemoryModuleType, MemoryStatus> {
            &self.base.entry_condition
        }
        fn durations(&self) -> (i32, i32) {
            (self.base.min_duration, self.base.max_duration)
        }
        fn end_timestamp(&self) -> i64 {
            self.base.end_timestamp
        }
        fn set_end_timestamp(&mut self, t: i64) {
            self.base.end_timestamp = t;
        }
        fn debug_string(&self) -> String {
            "TransportItemsBetweenContainers".to_string()
        }

        fn start(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            if let Some(nav) = mob.get_navigation() {
                nav.borrow_mut().set_can_path_to_targets_below_surface(true);
            }
        }

        fn check_extra_start_conditions(
            &mut self,
            _level: &Arc<dyn LevelReader>,
            mob: &mut Mob,
        ) -> bool {
            !mob.is_leashed()
        }

        fn can_still_use(
            &mut self,
            _level: &Arc<dyn LevelReader>,
            mob: &mut Mob,
            _game_time: i64,
        ) -> bool {
            !mob
                .living
                .brain_has(&keys::transport_items_cooldown_ticks())
                && !mob.living.brain_has(&keys::is_panicking())
                && !mob.is_leashed()
        }

        /// `timedOut` → false (бесконечное поведение).
        fn timed_out(&self, _game_time: i64) -> bool {
            false
        }

        fn tick(&mut self, level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            let flag = self.update_invalid_target(level, mob);
            if self.target.is_none() {
                // `stop` — onStartTravelling + reset nav flag.
                (self.on_start_travelling)(mob);
                self.state = TransportItemState::Travelling;
                self.interaction_state = None;
                self.ticks_since_reaching_target = 0;
                if let Some(nav) = mob.get_navigation() {
                    nav.borrow_mut().set_can_path_to_targets_below_surface(false);
                }
            } else if !flag {
                match self.state {
                    TransportItemState::Queuing => {
                        if let Some(t) = self.target.clone() {
                            self.on_queuing_for_target(&t, level, mob);
                        }
                    }
                    TransportItemState::Travelling => {
                        if let Some(t) = self.target.clone() {
                            self.on_travel_to_target(&t, level, mob);
                        }
                    }
                    TransportItemState::Interacting => {
                        if let Some(t) = self.target.clone() {
                            self.on_reached_target(&t, level, mob);
                        }
                    }
                }
            }
        }

        fn stop(&mut self, _level: &Arc<dyn LevelReader>, mob: &mut Mob, _game_time: i64) {
            (self.on_start_travelling)(mob);
            self.state = TransportItemState::Travelling;
            self.interaction_state = None;
            self.ticks_since_reaching_target = 0;
            if let Some(nav) = mob.get_navigation() {
                nav.borrow_mut().set_can_path_to_targets_below_surface(false);
            }
        }
    }

    TransportItemsBetweenContainers {
        base: BehaviorBase::new(conds(&[
            (keys::visited_block_positions(), MemoryStatus::Registered),
            (
                keys::unreachable_transport_block_positions(),
                MemoryStatus::Registered,
            ),
            (keys::transport_items_cooldown_ticks(), MemoryStatus::ValueAbsent),
            (keys::is_panicking(), MemoryStatus::ValueAbsent),
        ])),
        speed_modifier,
        horizontal_search_distance,
        vertical_search_distance,
        source_block_type,
        destination_block_type,
        should_queue_for_target,
        on_start_travelling,
        on_target_interaction_actions,
        target: None,
        state: TransportItemState::Travelling,
        interaction_state: None,
        ticks_since_reaching_target: 0,
    }
}

/// `setMiddleYPosition(mob, pos)` — pos + (0, bbox.ysize/2, 0).
fn set_middle_y_position(mob: &Mob, pos: Vec3) -> Vec3 {
    pos.add(
        0.0,
        mob.living.entity.get_bounding_box().get_ysize() / 2.0,
        0.0,
    )
}

/// `getInteractionRange` — path done → 1.0, иначе 0.5.
fn get_interaction_range(mob: &Mob) -> f64 {
    let finished = mob.get_navigation().map(|n| {
        n.borrow()
            .path()
            .map(|p| p.is_done())
            .unwrap_or(false)
    }).unwrap_or(false);
    if finished {
        1.0
    } else {
        0.5
    }
}

/// `isPickingUpItems` — main hand пуст.
fn is_picking_up_items(mob: &Mob) -> bool {
    mob.living.get_main_hand_item().is_empty()
}

/// `matchesGettingItemsRequirement` — container не пуст.
fn matches_getting_items_requirement(
    container: &Rc<RefCell<dyn crate::world::simple_container::Container>>,
) -> bool {
    !container.borrow().is_empty()
}

/// `matchesLeavingItemsRequirement` — container пуст или
/// содержит предмет из руки.
fn matches_leaving_items_requirement(
    mob: &Mob,
    container: &Rc<RefCell<dyn crate::world::simple_container::Container>>,
) -> bool {
    let c = container.borrow();
    if c.is_empty() {
        return true;
    }
    drop(c);
    has_item_matching_hand_item(mob, container)
}

/// `hasItemMatchingHandItem`.
fn has_item_matching_hand_item(
    mob: &Mob,
    container: &Rc<RefCell<dyn crate::world::simple_container::Container>>,
) -> bool {
    let hand = mob.living.get_main_hand_item().clone();
    container
        .borrow()
        .item_snapshot()
        .iter()
        .any(|s| ItemStack::is_same_item(s, &hand))
}

/// `pickupItemFromContainer` — до 16 шт. первого подходящего
/// предмета (shulker-box фильтр Purpur не портирован).
fn pick_up_item_from_container(
    container: &Rc<RefCell<dyn crate::world::simple_container::Container>>,
) -> ItemStack {
    let mut c = container.borrow_mut();
    let size = c.get_container_size();
    for i in 0..size {
        let stack = c.get_item(i);
        if !stack.is_empty() {
            let min = stack.get_count().min(16);
            return c.remove_item(i, min);
        }
    }
    ItemStack::empty()
}

/// `addItemsToContainer` — сложить main-hand в контейнер;
/// возвращает остаток в руке (vanilla-семантика setCount).
fn add_items_to_container(
    mob: &mut Mob,
    container: &Rc<RefCell<dyn crate::world::simple_container::Container>>,
) -> ItemStack {
    let size = container.borrow().get_container_size();
    let mut main_hand = mob.living.get_main_hand_item().clone();
    for i in 0..size {
        if main_hand.is_empty() {
            break;
        }
        let mut slot_stack = container.borrow().get_item(i);
        if slot_stack.is_empty() {
            container.borrow_mut().set_item(i, main_hand.clone());
            return ItemStack::empty();
        }
        if ItemStack::matches(&slot_stack, &main_hand)
            && slot_stack.get_count() < slot_stack.get_max_stack_size()
        {
            let i1 = slot_stack.get_max_stack_size() - slot_stack.get_count();
            let min = i1.min(main_hand.get_count());
            slot_stack.set_count(slot_stack.get_count() + min);
            // Ванильная семантика: `mainHandItem.setCount(count - i1)`.
            main_hand.set_count(main_hand.get_count() - i1);
            container.borrow_mut().set_item(i, slot_stack);
            if main_hand.is_empty() {
                return ItemStack::empty();
            }
        }
    }
    main_hand
}

/// `block_aabb(pos)` — unit-AABB блока (analog `new AABB(pos)`).
fn block_aabb(pos: BlockPos) -> Aabb {
    Aabb::of_size(
        &Vec3::new(
            pos.x as f64 + 0.5,
            pos.y as f64 + 0.5,
            pos.z as f64 + 0.5,
        ),
        1.0,
        1.0,
        1.0,
    )
}

/// `aabb.contains(x,y,z)` для BlockPos.
fn aabb_contains_pos(aabb: &Aabb, pos: BlockPos) -> bool {
    aabb.min_x <= pos.x as f64
        && aabb.max_x >= pos.x as f64 + 1.0
        && aabb.min_y <= pos.y as f64
        && aabb.max_y >= pos.y as f64 + 1.0
        && aabb.min_z <= pos.z as f64
        && aabb.max_z >= pos.z as f64 + 1.0
}

/// `state.block.id` path (`minecraft:chest` → "chest").
fn block_state_id(state: &crate::world::level::block::state::block_state::BlockState) -> String {
    state
        .block
        .id
        .path
        .to_string()
}

/// `state.isSolidRender`-эквивалент для `isChestBlockedAt`.
fn block_state_is_solid(state: &crate::world::level::block::state::block_state::BlockState) -> bool {
    // `BlockStateBase.isSolidRender` — нет в порте; эвристика:
    // не-air и не-fluid. `is_air` через block id.
    let id = block_state_id(state);
    !(id == "air" || id == "cave_air" || id == "void_air")
}

/// `ChestBlock.TYPE != SINGLE` → connected pos (double chest).
fn chest_connected_pos(
    state: &crate::world::level::block::state::block_state::BlockState,
    pos: BlockPos,
) -> Option<BlockPos> {
    // `type` property: "single"|"left"|"right"; `facing` — Direction.
    let ty = state.values.get_by_name("type");
    let is_double = match &ty {
        Some(crate::world::level::block::state::property::StateValue::Str(s)) => {
            s == "left" || s == "right"
        }
        _ => false,
    };
    if !is_double {
        return None;
    }
    // facing → direction step.
    let facing = state.values.get_by_name("facing");
    let dir = match &facing {
        Some(crate::world::level::block::state::property::StateValue::Str(s)) => {
            Direction::by_name(s)
        }
        _ => None,
    }?;
    Some(BlockPos::new(
        pos.x + dir.get_step_x(),
        pos.y + dir.get_step_y(),
        pos.z + dir.get_step_z(),
    ))
}
