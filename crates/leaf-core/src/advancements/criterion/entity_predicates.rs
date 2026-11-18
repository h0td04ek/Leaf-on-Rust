//! Перенос `net.minecraft.advancements.criterion` entity-предикатов:
//! EntityTypePredicate, DistancePredicate, MovementPredicate,
//! LocationPredicate, MobEffectsPredicate, EntityFlagsPredicate,
//! EntityEquipmentPredicate, SlotsPredicate, DamageSourcePredicate,
//! EntitySubPredicate (+варианты), EntityPredicate.

use crate::advancements::criterion::data_component_matchers::DataComponentMatchers;
use crate::advancements::criterion::item_predicate::ItemPredicate;
use crate::advancements::criterion::min_max_bounds::{MinMaxBoundsDoubles, MinMaxBoundsInts};
use crate::advancements::criterion::nbt_predicate::NbtPredicate;
use crate::advancements::criterion::state_properties_predicate::StatePropertiesPredicate;
use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::core::holder_set::HolderSet;
use crate::nbt::tag::Tag;
use crate::resources::identifier::Identifier;
use crate::world::damagesource::damage_source::DamageSource;
use crate::world::entity::entity::Entity;
use crate::world::entity::entity_type::EntityType;
use crate::world::level::level_reader::LevelReader;
use crate::world::phys::vec3::Vec3;
use std::collections::HashMap;

/// Общий доступ к entity для предикатов (как `Entity` в Java +
/// living-части через `LivingEntityAsTarget`).
pub trait EntityPredicateTarget {
    /// `getType()`.
    fn target_type(&self) -> &Holder<EntityType>;
    /// позиция.
    fn target_pos(&self) -> Vec3;
    /// `getKnownMovement()`.
    fn target_known_movement(&self) -> Vec3;
    /// `fallDistance`.
    fn target_fall_distance(&self) -> f32;
    /// `onGround()`.
    fn target_on_ground(&self) -> bool;
    /// `getOnPos()` — блок под ногами.
    fn target_on_pos(&self) -> BlockPos;
    /// `getBlockPosBelowThatAffectsMyMovement()`.
    fn target_pos_below_affecting_movement(&self) -> BlockPos;
    /// `isBaby()`.
    fn target_is_baby(&self) -> bool;
    /// `isOnFire()`.
    fn target_is_on_fire(&self) -> bool;
    /// `isInWater()`.
    fn target_is_in_water(&self) -> bool;
    /// `isCrouching()` / `isSprinting()` / `isSwimming()` /
    /// `isFallFlying()` / `isFlying()`.
    fn target_is_crouching(&self) -> bool {
        false
    }
    fn target_is_sprinting(&self) -> bool {
        false
    }
    fn target_is_swimming(&self) -> bool {
        false
    }
    fn target_is_fall_flying(&self) -> bool {
        false
    }
    fn target_is_flying(&self) -> bool {
        false
    }
    /// `getVehicle()`.
    fn target_vehicle(&self) -> Option<&Entity> {
        None
    }
    /// `getPassengers()`.
    fn target_passengers(&self) -> Vec<&Entity> {
        Vec::new()
    }
    /// `tickCount`.
    fn target_tick_count(&self) -> i32;
    /// `getTeam()` — имя команды.
    fn target_team_name(&self) -> Option<String> {
        None
    }
    /// Эффект моба по identifier — `hasEffect`/`getEffect`
    /// (`None` для не-living).
    fn target_effect(
        &self,
        _id: &Identifier,
    ) -> Option<crate::world::effect::mob_effect_instance::MobEffectInstance> {
        None
    }
    /// Предмет в слоте экипировки (`None` — нет доступа).
    fn target_equipment(
        &self,
        _slot: crate::entity::equipment_slot::EquipmentSlot,
    ) -> Option<&crate::world::item::item_stack::ItemStack> {
        None
    }
    /// `DataComponentGetter` — components entity.
    fn target_components(&self) -> Option<&dyn crate::core::component::data_component_getter::DataComponentGetter> {
        None
    }
    /// `saveWithoutId` для `NbtPredicate`.
    fn target_save_nbt(&self) -> crate::nbt::compound::CompoundTag;
    /// `getSlot(slot)` для `SlotsPredicate`.
    fn target_slot_item(&self, _slot: i64) -> Option<&crate::world::item::item_stack::ItemStack> {
        None
    }
    /// `getTarget()` для `Mob` (для targetedEntity).
    fn target_target_entity(&self) -> Option<&dyn EntityPredicateTarget> {
        None
    }
    /// Размер слизи для `SlimePredicate`.
    fn target_slime_size(&self) -> Option<i32> {
        None
    }
    /// Стойкость/флаги для `RaiderPredicate`/`SheepPredicate` —
    /// через entity-specific реализации позже.
    fn target_raider_flags(&self) -> Option<(bool, bool, bool, i32)> {
        None
    }
    fn target_sheep_flags(&self) -> Option<(Option<crate::world::item::dye_color::DyeColor>, Option<bool>)> {
        None
    }
}

// ---------- EntityTypePredicate ----------

/// `EntityTypePredicate` — `HolderSet<EntityType<?>>`.
#[derive(Clone, Debug)]
pub struct EntityTypePredicate {
    /// `types`.
    pub types: HolderSet<EntityType>,
}

impl EntityTypePredicate {
    /// `matches(type)`.
    pub fn matches(&self, entity_type: &Holder<EntityType>) -> bool {
        holder_set_contains(&self.types, entity_type)
    }
}

/// Проверка `HolderSet.contains` — для `Named` резолвится через
/// реестр тегов когда он будет подключён; пока `false`.
pub(crate) fn holder_set_contains<T>(set: &HolderSet<T>, holder: &Holder<T>) -> bool
where
    T: PartialEq,
{
    match set {
        HolderSet::Direct(v) => v.iter().any(|h| h == holder),
        HolderSet::Named(_) => false,
    }
}

// ---------- DistancePredicate ----------

/// `DistancePredicate` — x/y/z/horizontal/absolute.
#[derive(Clone, Debug, Default)]
pub struct DistancePredicate {
    /// `x`.
    pub x: MinMaxBoundsDoubles,
    /// `y`.
    pub y: MinMaxBoundsDoubles,
    /// `z`.
    pub z: MinMaxBoundsDoubles,
    /// `horizontal`.
    pub horizontal: MinMaxBoundsDoubles,
    /// `absolute`.
    pub absolute: MinMaxBoundsDoubles,
}

impl DistancePredicate {
    /// `horizontal(bounds)`.
    pub fn horizontal(bounds: MinMaxBoundsDoubles) -> Self {
        Self {
            horizontal: bounds,
            ..Default::default()
        }
    }
    /// `vertical(bounds)`.
    pub fn vertical(bounds: MinMaxBoundsDoubles) -> Self {
        Self {
            y: bounds,
            ..Default::default()
        }
    }
    /// `absolute(bounds)`.
    pub fn absolute(bounds: MinMaxBoundsDoubles) -> Self {
        Self {
            absolute: bounds,
            ..Default::default()
        }
    }
    /// `matches(x1,y1,z1,x2,y2,z2)` — как в Java.
    pub fn matches(&self, x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> bool {
        let dx = x1 - x2;
        let dy = y1 - y2;
        let dz = z1 - z2;
        if !self.x.matches(dx) || !self.y.matches(dy) || !self.z.matches(dz) {
            return false;
        }
        let horizontal = (dx * dx + dz * dz).sqrt();
        let absolute = (dx * dx + dy * dy + dz * dz).sqrt();
        self.horizontal.matches(horizontal) && self.absolute.matches(absolute)
    }
}

// ---------- MovementPredicate ----------

/// `MovementPredicate` — x/y/z/speed/horizontalSpeed/verticalSpeed/
/// fallDistance.
#[derive(Clone, Debug, Default)]
pub struct MovementPredicate {
    /// `x`.
    pub x: MinMaxBoundsDoubles,
    /// `y`.
    pub y: MinMaxBoundsDoubles,
    /// `z`.
    pub z: MinMaxBoundsDoubles,
    /// `speed`.
    pub speed: MinMaxBoundsDoubles,
    /// `horizontalSpeed`.
    pub horizontal_speed: MinMaxBoundsDoubles,
    /// `verticalSpeed`.
    pub vertical_speed: MinMaxBoundsDoubles,
    /// `fallDistance`.
    pub fall_distance: MinMaxBoundsDoubles,
}

impl MovementPredicate {
    /// `matches(x,y,z,speed,horizontalSpeed,verticalSpeed,fallDistance)`.
    pub fn matches(
        &self,
        x: f64,
        y: f64,
        z: f64,
        speed: f64,
        horizontal_speed: f64,
        vertical_speed: f64,
        fall_distance: f64,
    ) -> bool {
        self.x.matches(x)
            && self.y.matches(y)
            && self.z.matches(z)
            && self.speed.matches(speed)
            && self.horizontal_speed.matches(horizontal_speed)
            && self.vertical_speed.matches(vertical_speed)
            && self.fall_distance.matches(fall_distance)
    }
}

// ---------- LocationPredicate ----------

/// `LocationPredicate.PositionPredicate`.
#[derive(Clone, Debug, Default)]
pub struct PositionPredicate {
    /// `x`.
    pub x: MinMaxBoundsDoubles,
    /// `y`.
    pub y: MinMaxBoundsDoubles,
    /// `z`.
    pub z: MinMaxBoundsDoubles,
}

impl PositionPredicate {
    /// `matches(x,y,z)`.
    pub fn matches(&self, x: f64, y: f64, z: f64) -> bool {
        self.x.matches(x) && self.y.matches(y) && self.z.matches(z)
    }
}

/// `LightPredicate` — block/sky light bounds.
#[derive(Clone, Debug, Default)]
pub struct LightPredicate {
    /// `block`.
    pub block: Option<MinMaxBoundsInts>,
    /// `sky`.
    pub sky: Option<MinMaxBoundsInts>,
}

impl LightPredicate {
    /// `matches(level, pos)`.
    pub fn matches(&self, level: &dyn LevelReader, pos: BlockPos) -> bool {
        if let Some(b) = &self.block {
            let light = level.get_brightness(crate::world::level::light_layer::LightLayer::Block, pos);
            if !b.matches(light) {
                return false;
            }
        }
        if let Some(s) = &self.sky {
            let light = level.get_brightness(crate::world::level::light_layer::LightLayer::Sky, pos);
            if !s.matches(light) {
                return false;
            }
        }
        true
    }
}

/// `FluidPredicate` — fluid holder-set + properties + tag.
#[derive(Clone, Debug, Default)]
pub struct FluidPredicate {
    /// `fluids`.
    pub fluids: Option<HolderSet<crate::world::level::block::state::fluid_state::Fluid>>,
    /// `properties`.
    pub properties: Option<StatePropertiesPredicate>,
    /// `tag` — `TagKey<Fluid>`.
    pub tag: Option<crate::tags::tag_key::TagKey<crate::world::level::block::state::fluid_state::Fluid>>,
}

impl FluidPredicate {
    /// `matches(level, pos)`.
    pub fn matches(&self, level: &dyn LevelReader, pos: BlockPos) -> bool {
        let state = match level.get_fluid_state(pos) {
            Some(s) => s,
            None => return false,
        };
        if let Some(fluids) = &self.fluids {
            let mut found = false;
            if let HolderSet::Direct(v) = fluids {
                for h in v {
                    if let Some(f) = h.value() {
                        if f.id == state.fluid.id {
                            found = true;
                            break;
                        }
                    }
                }
            }
            if !found {
                return false;
            }
        }
        if let Some(props) = &self.properties {
            if !props.matches_fluid_state(&state) {
                return false;
            }
        }
        true
    }
}

/// `LocationPredicate` — position/biomes/structures/dimension/
/// smokey/light/block/fluid/canSeeSky.
#[derive(Clone, Default)]
pub struct LocationPredicate {
    /// `position`.
    pub position: Option<PositionPredicate>,
    /// `biomes`.
    pub biomes: Option<HolderSet<crate::world::level::biome::biome::Biome>>,
    /// `structures`.
    pub structures: Option<HolderSet<crate::world::level::levelgen::structure::Structure>>,
    /// `dimension`.
    pub dimension: Option<Identifier>,
    /// `smokey`.
    pub smokey: Option<bool>,
    /// `light`.
    pub light: Option<LightPredicate>,
    /// `block`.
    pub block: Option<crate::advancements::criterion::block_predicate::BlockPredicate>,
    /// `fluid`.
    pub fluid: Option<FluidPredicate>,
    /// `canSeeSky`.
    pub can_see_sky: Option<bool>,
}

impl LocationPredicate {
    /// `matches(level, x, y, z)`.
    pub fn matches(&self, level: &dyn LevelReader, x: f64, y: f64, z: f64) -> bool {
        let pos = BlockPos::new(x.floor() as i32, y.floor() as i32, z.floor() as i32);
        if let Some(p) = &self.position {
            if !p.matches(x, y, z) {
                return false;
            }
        }
        if let Some(biomes) = &self.biomes {
            if !level
                .biome_at(pos)
                .map(|b| holder_set_contains(biomes, b))
                .unwrap_or(false)
            {
                return false;
            }
        }
        if let Some(structures) = &self.structures {
            if !level.structure_at(pos, structures).unwrap_or(false) {
                return false;
            }
        }
        if let Some(dim) = &self.dimension {
            if level.dimension_id() != Some(dim) {
                return false;
            }
        }
        if let Some(smokey) = self.smokey {
            if level.is_smokey_at(pos) != smokey {
                return false;
            }
        }
        if let Some(light) = &self.light {
            if !light.matches(level, pos) {
                return false;
            }
        }
        if let Some(block) = &self.block {
            if !block.matches(level, pos) {
                return false;
            }
        }
        if let Some(fluid) = &self.fluid {
            if !fluid.matches(level, pos) {
                return false;
            }
        }
        if let Some(can_see_sky) = self.can_see_sky {
            if level.can_see_sky(pos) != can_see_sky {
                return false;
            }
        }
        true
    }
}

// ---------- MobEffectsPredicate ----------

/// `MobEffectsPredicate` — map эффект → instance-predicate.
#[derive(Clone, Debug, Default)]
pub struct MobEffectsPredicate {
    /// `effectMap` — `Identifier` эффекта → предикат.
    pub effect_map: HashMap<Identifier, MobEffectInstancePredicate>,
}

/// `MobEffectsPredicate.MobEffectInstancePredicate`.
#[derive(Clone, Debug, Default)]
pub struct MobEffectInstancePredicate {
    /// `amplifier`.
    pub amplifier: Option<MinMaxBoundsInts>,
    /// `duration`.
    pub duration: Option<MinMaxBoundsInts>,
    /// `ambient`.
    pub ambient: Option<bool>,
    /// `visible`.
    pub visible: Option<bool>,
}

impl MobEffectsPredicate {
    /// `matches(entity)`.
    pub fn matches(&self, entity: &dyn EntityPredicateTarget) -> bool {
        for (id, pred) in &self.effect_map {
            let inst = match entity.target_effect(id) {
                Some(i) => i,
                None => return false,
            };
            if let Some(a) = &pred.amplifier {
                if !a.matches(inst.amplifier()) {
                    return false;
                }
            }
            if let Some(d) = &pred.duration {
                if !d.matches(inst.duration()) {
                    return false;
                }
            }
            if let Some(amb) = pred.ambient {
                if inst.details.ambient != amb {
                    return false;
                }
            }
            if let Some(vis) = pred.visible {
                if inst.details.show_particles != vis {
                    return false;
                }
            }
        }
        true
    }
}

// ---------- EntityFlagsPredicate ----------

/// `EntityFlagsPredicate`.
#[derive(Clone, Debug, Default)]
pub struct EntityFlagsPredicate {
    /// `isOnGround`.
    pub is_on_ground: Option<bool>,
    /// `isOnFire`.
    pub is_on_fire: Option<bool>,
    /// `isCrouching`.
    pub is_crouching: Option<bool>,
    /// `isSprinting`.
    pub is_sprinting: Option<bool>,
    /// `isSwimming`.
    pub is_swimming: Option<bool>,
    /// `isFlying`.
    pub is_flying: Option<bool>,
    /// `isBaby`.
    pub is_baby: Option<bool>,
    /// `isInWater`.
    pub is_in_water: Option<bool>,
    /// `isFallFlying`.
    pub is_fall_flying: Option<bool>,
}

impl EntityFlagsPredicate {
    /// `matches(entity)`.
    pub fn matches(&self, entity: &dyn EntityPredicateTarget) -> bool {
        self.check(self.is_on_ground, entity.target_on_ground())
            && self.check(self.is_on_fire, entity.target_is_on_fire())
            && self.check(self.is_crouching, entity.target_is_crouching())
            && self.check(self.is_sprinting, entity.target_is_sprinting())
            && self.check(self.is_swimming, entity.target_is_swimming())
            && self.check(self.is_flying, entity.target_is_flying())
            && self.check(self.is_baby, entity.target_is_baby())
            && self.check(self.is_in_water, entity.target_is_in_water())
            && self.check(self.is_fall_flying, entity.target_is_fall_flying())
    }
    fn check(&self, expected: Option<bool>, actual: bool) -> bool {
        expected.map_or(true, |e| e == actual)
    }
}

// ---------- EntityEquipmentPredicate ----------

/// `EntityEquipmentPredicate`.
#[derive(Clone, Debug, Default)]
pub struct EntityEquipmentPredicate {
    /// `head`.
    pub head: Option<ItemPredicate>,
    /// `chest`.
    pub chest: Option<ItemPredicate>,
    /// `legs`.
    pub legs: Option<ItemPredicate>,
    /// `feet`.
    pub feet: Option<ItemPredicate>,
    /// `body`.
    pub body: Option<ItemPredicate>,
    /// `mainhand`.
    pub mainhand: Option<ItemPredicate>,
    /// `offhand`.
    pub offhand: Option<ItemPredicate>,
}

impl EntityEquipmentPredicate {
    /// `matches(entity)`.
    pub fn matches(&self, entity: &dyn EntityPredicateTarget) -> bool {
        use crate::entity::equipment_slot::EquipmentSlot as S;
        self.check(self.head.as_ref(), S::Head, entity)
            && self.check(self.chest.as_ref(), S::Chest, entity)
            && self.check(self.legs.as_ref(), S::Legs, entity)
            && self.check(self.feet.as_ref(), S::Feet, entity)
            && self.check(self.body.as_ref(), S::Body, entity)
            && self.check(self.mainhand.as_ref(), S::Mainhand, entity)
            && self.check(self.offhand.as_ref(), S::Offhand, entity)
    }
    fn check(
        &self,
        pred: Option<&ItemPredicate>,
        slot: crate::entity::equipment_slot::EquipmentSlot,
        entity: &dyn EntityPredicateTarget,
    ) -> bool {
        match pred {
            None => true,
            Some(p) => match entity.target_equipment(slot) {
                Some(stack) => p.test(stack),
                None => false,
            },
        }
    }
}

// ---------- SlotsPredicate ----------

/// `SlotRange`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SlotRange {
    /// `indexMin`.
    pub index_min: i32,
    /// `slotMin`.
    pub slot_min: i32,
    /// `count`.
    pub count: i32,
}

impl SlotRange {
    /// `new(indexMin, slotMin, count)`.
    pub fn new(index_min: i32, slot_min: i32, count: i32) -> Self {
        Self {
            index_min,
            slot_min,
            count,
        }
    }
    /// `indexOf(slot)`.
    pub fn index_of(&self, slot: i32) -> Option<i32> {
        if slot < self.slot_min || slot >= self.slot_min + self.count {
            None
        } else {
            Some(self.index_min + (slot - self.slot_min))
        }
    }
}

/// `SlotsPredicate`.
#[derive(Clone, Debug, Default)]
pub struct SlotsPredicate {
    /// `slots`.
    pub slots: HashMap<SlotRange, ItemPredicate>,
}

impl SlotsPredicate {
    /// `matches(entity)` — для каждой пары проверить предметы
    /// в диапазоне слотов.
    pub fn matches(&self, entity: &dyn EntityPredicateTarget) -> bool {
        for (range, pred) in &self.slots {
            for i in 0..range.count {
                let slot = range.slot_min + i;
                let item = entity.target_slot_item(slot as i64);
                match item {
                    Some(stack) => {
                        if !pred.test(stack) {
                            return false;
                        }
                    }
                    None => return false,
                }
            }
        }
        true
    }
}

// ---------- DamageSourcePredicate ----------

/// `DamageSourcePredicate`.
#[derive(Clone, Default)]
pub struct DamageSourcePredicate {
    /// `tags` — `TagPredicate<DamageType>` как список
    /// `(TagKey, expected)`.
    pub tags: Vec<(crate::tags::tag_key::TagKey<crate::world::damagesource::damage_type::DamageType>, bool)>,
    /// `directEntity`.
    pub direct_entity: Option<Box<EntityPredicate>>,
    /// `sourceEntity`.
    pub source_entity: Option<Box<EntityPredicate>>,
    /// `isDirect`.
    pub is_direct: Option<bool>,
}

impl DamageSourcePredicate {
    /// `matches(level, position, damageSource)`.
    pub fn matches(
        &self,
        level: &dyn LevelReader,
        position: Option<Vec3>,
        source: &DamageSource,
    ) -> bool {
        for (tag, expected) in &self.tags {
            if source.is_tag(tag) != *expected {
                return false;
            }
        }
        if let Some(direct) = &self.direct_entity {
            let ok = source
                .get_direct_entity()
                .map(|e| direct.matches_entity(level, position, e))
                .unwrap_or(false);
            if !ok {
                return false;
            }
        }
        if let Some(src) = &self.source_entity {
            let ok = source
                .get_causing_entity()
                .map(|e| src.matches_entity(level, position, e))
                .unwrap_or(false);
            if !ok {
                return false;
            }
        }
        if let Some(is_direct) = self.is_direct {
            let actual = match (source.get_direct_entity(), source.get_causing_entity()) {
                (Some(d), Some(e)) => std::ptr::eq(d, e),
                _ => false,
            };
            if is_direct != actual {
                return false;
            }
        }
        true
    }
}

// ---------- EntitySubPredicate ----------

/// `EntitySubPredicate` — саб-предикаты entity
/// (lightning/fishing_hook/slime/raider/sheep; `player` —
/// `PlayerPredicate` портируется отдельно).
#[derive(Clone)]
pub enum EntitySubPredicate {
    /// `lightning`.
    LightningBolt(LightningBoltPredicate),
    /// `fishing_hook`.
    FishingHook(FishingHookPredicate),
    /// `slime`.
    Slime(SlimePredicate),
    /// `raider`.
    Raider(RaiderPredicate),
    /// `sheep`.
    Sheep(SheepPredicate),
}

impl EntitySubPredicate {
    /// `matches(entity, level, position)`.
    pub fn matches(
        &self,
        entity: &dyn EntityPredicateTarget,
        level: &dyn LevelReader,
        position: Option<Vec3>,
    ) -> bool {
        match self {
            EntitySubPredicate::LightningBolt(p) => p.matches(entity, level, position),
            EntitySubPredicate::FishingHook(p) => p.matches(entity, level, position),
            EntitySubPredicate::Slime(p) => p.matches(entity, level, position),
            EntitySubPredicate::Raider(p) => p.matches(entity, level, position),
            EntitySubPredicate::Sheep(p) => p.matches(entity, level, position),
        }
    }
}

/// `LightningBoltPredicate`.
#[derive(Clone, Default)]
pub struct LightningBoltPredicate {
    /// `entityPredicate` — на саму молнию.
    pub entity_predicate: Option<Box<EntityPredicate>>,
    /// `targetPredicate` — на struck entity (проверяется через
    /// `target_target_entity` если есть).
    pub target_predicate: Option<Box<EntityPredicate>>,
}

impl LightningBoltPredicate {
    pub fn matches(
        &self,
        entity: &dyn EntityPredicateTarget,
        level: &dyn LevelReader,
        position: Option<Vec3>,
    ) -> bool {
        if let Some(p) = &self.entity_predicate {
            if !p.matches_target(level, position, entity) {
                return false;
            }
        }
        if let Some(t) = &self.target_predicate {
            let ok = entity
                .target_target_entity()
                .map(|te| t.matches_target(level, position, te))
                .unwrap_or(false);
            if !ok {
                return false;
            }
        }
        true
    }
}

/// `FishingHookPredicate`.
#[derive(Clone, Default)]
pub struct FishingHookPredicate {
    /// `bobber` — `EntityPredicate` на крючок.
    pub bobber: Option<Box<EntityPredicate>>,
    /// `hooked_in_water`.
    pub hooked_in_water: Option<bool>,
    /// `open_water`.
    pub open_water: Option<bool>,
}

impl FishingHookPredicate {
    pub fn matches(
        &self,
        entity: &dyn EntityPredicateTarget,
        level: &dyn LevelReader,
        position: Option<Vec3>,
    ) -> bool {
        if let Some(b) = &self.bobber {
            if !b.matches_target(level, position, entity) {
                return false;
            }
        }
        entity
            .target_type()
            .value()
            .map(|t| t.to_string() == "minecraft:fishing_hook")
            .unwrap_or(false)
    }
}

/// `SlimePredicate`.
#[derive(Clone, Debug, Default)]
pub struct SlimePredicate {
    /// `size`.
    pub size: Option<MinMaxBoundsInts>,
}

impl SlimePredicate {
    pub fn matches(
        &self,
        entity: &dyn EntityPredicateTarget,
        _level: &dyn LevelReader,
        _position: Option<Vec3>,
    ) -> bool {
        match &self.size {
            Some(s) => match entity.target_slime_size() {
                Some(size) => s.matches(size),
                None => false,
            },
            None => entity
                .target_type()
                .value()
                .map(|t| {
                    let id = t.to_string();
                    id == "minecraft:slime" || id == "minecraft:magma_cube"
                })
                .unwrap_or(false),
        }
    }
}

/// `RaiderPredicate` — hasRaid/isCaptain/isPatrolLeader/stun_ticks/
/// health/max_health (флаги через `target_raider_flags`).
#[derive(Clone, Debug, Default)]
pub struct RaiderPredicate {
    /// `hasRaid`.
    pub has_raid: Option<bool>,
    /// `isCaptain`.
    pub is_captain: Option<bool>,
    /// `isPatrolLeader`.
    pub is_patrol_leader: Option<bool>,
    /// `stun_ticks`.
    pub stun_ticks: Option<MinMaxBoundsInts>,
    /// `health`.
    pub health: Option<MinMaxBoundsDoubles>,
    /// `max_health`.
    pub max_health: Option<MinMaxBoundsDoubles>,
}

impl RaiderPredicate {
    pub fn matches(
        &self,
        entity: &dyn EntityPredicateTarget,
        _level: &dyn LevelReader,
        _position: Option<Vec3>,
    ) -> bool {
        let is_raider = entity
            .target_type()
            .value()
            .map(|t| {
                matches!(
                    t.to_string().as_str(),
                    "minecraft:pillager"
                        | "minecraft:vindicator"
                        | "minecraft:evoker"
                        | "minecraft:raveager"
                        | "minecraft:ravager"
                        | "minecraft:witch"
                )
            })
            .unwrap_or(false);
        if !is_raider {
            return false;
        }
        if self.has_raid.is_some()
            || self.is_captain.is_some()
            || self.is_patrol_leader.is_some()
            || self.stun_ticks.is_some()
            || self.health.is_some()
            || self.max_health.is_some()
        {
            // Детальные поля — при порте Raider entity;
            // `target_raider_flags` даёт (has_raid, is_captain,
            // is_patrol_leader, stun_ticks).
            if let Some((raid, captain, leader, stun)) = entity.target_raider_flags() {
                if let Some(h) = self.has_raid {
                    if h != raid {
                        return false;
                    }
                }
                if let Some(c) = self.is_captain {
                    if c != captain {
                        return false;
                    }
                }
                if let Some(l) = self.is_patrol_leader {
                    if l != leader {
                        return false;
                    }
                }
                if let Some(s) = &self.stun_ticks {
                    if !s.matches(stun) {
                        return false;
                    }
                }
            } else {
                return false;
            }
        }
        true
    }
}

/// `SheepPredicate` — color/sheared.
#[derive(Clone, Debug, Default)]
pub struct SheepPredicate {
    /// `color`.
    pub color: Option<crate::world::item::dye_color::DyeColor>,
    /// `sheared`.
    pub sheared: Option<bool>,
}

impl SheepPredicate {
    pub fn matches(
        &self,
        entity: &dyn EntityPredicateTarget,
        _level: &dyn LevelReader,
        _position: Option<Vec3>,
    ) -> bool {
        let is_sheep = entity
            .target_type()
            .value()
            .map(|t| t.to_string() == "minecraft:sheep")
            .unwrap_or(false);
        if !is_sheep {
            return false;
        }
        if self.color.is_some() || self.sheared.is_some() {
            return match entity.target_sheep_flags() {
                Some((color, sheared)) => {
                    if let Some(c) = self.color {
                        if color != Some(c) {
                            return false;
                        }
                    }
                    if let Some(s) = self.sheared {
                        if sheared != Some(s) {
                            return false;
                        }
                    }
                    true
                }
                None => false,
            };
        }
        true
    }
}

// ---------- EntityPredicate ----------

/// `EntityPredicate.LocationWrapper`.
#[derive(Clone, Default)]
pub struct LocationWrapper {
    /// `located`.
    pub located: Option<LocationPredicate>,
    /// `steppingOn`.
    pub stepping_on: Option<LocationPredicate>,
    /// `affectsMovement`.
    pub affects_movement: Option<LocationPredicate>,
}

/// `EntityPredicate`.
#[derive(Clone, Default)]
pub struct EntityPredicate {
    /// `entityType`.
    pub entity_type: Option<EntityTypePredicate>,
    /// `distanceToPlayer`.
    pub distance_to_player: Option<DistancePredicate>,
    /// `movement`.
    pub movement: Option<MovementPredicate>,
    /// `location`.
    pub location: LocationWrapper,
    /// `effects`.
    pub effects: Option<MobEffectsPredicate>,
    /// `nbt`.
    pub nbt: Option<NbtPredicate>,
    /// `flags`.
    pub flags: Option<EntityFlagsPredicate>,
    /// `equipment`.
    pub equipment: Option<EntityEquipmentPredicate>,
    /// `subPredicate`.
    pub sub_predicate: Option<EntitySubPredicate>,
    /// `periodicTick`.
    pub periodic_tick: Option<i32>,
    /// `vehicle`.
    pub vehicle: Option<Box<EntityPredicate>>,
    /// `passenger`.
    pub passenger: Option<Box<EntityPredicate>>,
    /// `targetedEntity`.
    pub targeted_entity: Option<Box<EntityPredicate>>,
    /// `team`.
    pub team: Option<String>,
    /// `slots`.
    pub slots: Option<SlotsPredicate>,
    /// `components`.
    pub components: DataComponentMatchers,
}

impl EntityPredicate {
    /// `matches(level, position, entity)` — entity как target.
    pub fn matches_target(
        &self,
        level: &dyn LevelReader,
        position: Option<Vec3>,
        entity: &dyn EntityPredicateTarget,
    ) -> bool {
        if let Some(t) = &self.entity_type {
            if !t.matches(entity.target_type()) {
                return false;
            }
        }
        if let Some(pos) = position {
            if let Some(d) = &self.distance_to_player {
                let p = entity.target_pos();
                if !d.matches(pos.x, pos.y, pos.z, p.x, p.y, p.z) {
                    return false;
                }
            }
        } else if self.distance_to_player.is_some() {
            return false;
        }
        if let Some(m) = &self.movement {
            let known = entity.target_known_movement();
            let vec = known.scale(20.0);
            let speed = (vec.x * vec.x + vec.y * vec.y + vec.z * vec.z).sqrt();
            let horizontal = (vec.x * vec.x + vec.z * vec.z).sqrt();
            let vertical = vec.y.abs();
            if !m.matches(vec.x, vec.y, vec.z, speed, horizontal, vertical, entity.target_fall_distance() as f64) {
                return false;
            }
        }
        if let Some(located) = &self.location.located {
            let p = entity.target_pos();
            if !located.matches(level, p.x, p.y, p.z) {
                return false;
            }
        }
        if let Some(stepping) = &self.location.stepping_on {
            let on_pos = entity.target_on_pos();
            let c = Vec3::new(
                on_pos.x as f64 + 0.5,
                on_pos.y as f64,
                on_pos.z as f64 + 0.5,
            );
            if !entity.target_on_ground() || !stepping.matches(level, c.x, c.y, c.z) {
                return false;
            }
        }
        if let Some(affects) = &self.location.affects_movement {
            let bp = entity.target_pos_below_affecting_movement();
            let c = Vec3::new(bp.x as f64 + 0.5, bp.y as f64, bp.z as f64 + 0.5);
            if !affects.matches(level, c.x, c.y, c.z) {
                return false;
            }
        }
        if let Some(effects) = &self.effects {
            if !effects.matches(entity) {
                return false;
            }
        }
        if let Some(flags) = &self.flags {
            if !flags.matches(entity) {
                return false;
            }
        }
        if let Some(equipment) = &self.equipment {
            if !equipment.matches(entity) {
                return false;
            }
        }
        if let Some(sub) = &self.sub_predicate {
            if !sub.matches(entity, level, position) {
                return false;
            }
        }
        if let Some(vehicle) = &self.vehicle {
            let ok = entity
                .target_vehicle()
                .map(|v| vehicle.matches_entity(level, position, v))
                .unwrap_or(false);
            if !ok {
                return false;
            }
        }
        if let Some(passenger) = &self.passenger {
            if !entity
                .target_passengers()
                .iter()
                .any(|p| passenger.matches_entity(level, position, p))
            {
                return false;
            }
        }
        if let Some(targeted) = &self.targeted_entity {
            let ok = entity
                .target_target_entity()
                .map(|t| targeted.matches_target(level, position, t))
                .unwrap_or(false);
            if !ok {
                return false;
            }
        }
        if let Some(tick) = self.periodic_tick {
            if tick > 0 && entity.target_tick_count() % tick != 0 {
                return false;
            }
        }
        if let Some(team) = &self.team {
            match entity.target_team_name() {
                Some(name) if name == *team => {}
                _ => return false,
            }
        }
        if let Some(slots) = &self.slots {
            if !slots.matches(entity) {
                return false;
            }
        }
        if let Some(components) = entity.target_components() {
            if !self.components.test(components) {
                return false;
            }
        }
        if let Some(nbt) = &self.nbt {
            let tag = Tag::Compound(entity.target_save_nbt());
            if !nbt.matches(Some(&tag)) {
                return false;
            }
        }
        true
    }

    /// `matches(level, position, entity: &Entity)` — базовая entity.
    pub fn matches_entity(&self, level: &dyn LevelReader, position: Option<Vec3>, entity: &Entity) -> bool {
        self.matches_target(level, position, &EntityAsTarget(entity))
    }
}

/// Адаптер `&Entity` → `EntityPredicateTarget` (базовые поля;
/// living-части недоступны — как в Java для не-LivingEntity).
pub struct EntityAsTarget<'a>(pub &'a Entity);

impl<'a> EntityPredicateTarget for EntityAsTarget<'a> {
    fn target_type(&self) -> &Holder<EntityType> {
        self.0.get_type()
    }
    fn target_pos(&self) -> Vec3 {
        self.0.get_pos()
    }
    fn target_known_movement(&self) -> Vec3 {
        Vec3::ZERO
    }
    fn target_fall_distance(&self) -> f32 {
        self.0.fall_distance
    }
    fn target_on_ground(&self) -> bool {
        self.0.is_on_ground()
    }
    fn target_on_pos(&self) -> BlockPos {
        BlockPos::new(
            self.0.pos.x.floor() as i32,
            (self.0.pos.y - 0.2).floor() as i32,
            self.0.pos.z.floor() as i32,
        )
    }
    fn target_pos_below_affecting_movement(&self) -> BlockPos {
        self.target_on_pos()
    }
    fn target_is_baby(&self) -> bool {
        false
    }
    fn target_is_on_fire(&self) -> bool {
        self.0.is_on_fire()
    }
    fn target_is_in_water(&self) -> bool {
        self.0.is_in_water()
    }
    fn target_is_crouching(&self) -> bool {
        false
    }
    fn target_is_sprinting(&self) -> bool {
        false
    }
    fn target_is_swimming(&self) -> bool {
        false
    }
    fn target_is_fall_flying(&self) -> bool {
        false
    }
    fn target_is_flying(&self) -> bool {
        false
    }
    fn target_vehicle(&self) -> Option<&Entity> {
        self.0.get_vehicle()
    }
    fn target_passengers(&self) -> Vec<&Entity> {
        self.0.get_passengers()
    }
    fn target_tick_count(&self) -> i32 {
        self.0.tick_count
    }
    fn target_team_name(&self) -> Option<String> {
        self.0.get_team().map(|t| t.get_name().to_string())
    }
    fn target_components(&self) -> Option<&dyn crate::core::component::data_component_getter::DataComponentGetter> {
        None
    }
    fn target_save_nbt(&self) -> crate::nbt::compound::CompoundTag {
        let mut tag = crate::nbt::compound::CompoundTag::new();
        self.0.save(&mut tag);
        tag
    }
}

/// `EntityPredicateTarget` для `LivingEntity` — делегирует базовой
/// entity + живым частям (эффекты, экипировка, флаги).
pub struct LivingEntityAsTarget<'a>(pub &'a crate::world::entity::living_entity::LivingEntity);

impl<'a> EntityPredicateTarget for LivingEntityAsTarget<'a> {
    fn target_type(&self) -> &Holder<EntityType> {
        self.0.entity.get_type()
    }
    fn target_pos(&self) -> Vec3 {
        self.0.entity.get_pos()
    }
    fn target_known_movement(&self) -> Vec3 {
        self.0.entity.velocity
    }
    fn target_fall_distance(&self) -> f32 {
        self.0.entity.fall_distance
    }
    fn target_on_ground(&self) -> bool {
        self.0.entity.is_on_ground()
    }
    fn target_on_pos(&self) -> BlockPos {
        BlockPos::new(
            self.0.entity.pos.x.floor() as i32,
            (self.0.entity.pos.y - 0.2).floor() as i32,
            self.0.entity.pos.z.floor() as i32,
        )
    }
    fn target_pos_below_affecting_movement(&self) -> BlockPos {
        self.target_on_pos()
    }
    fn target_is_baby(&self) -> bool {
        self.0.entity.is_baby()
    }
    fn target_is_on_fire(&self) -> bool {
        self.0.entity.is_on_fire()
    }
    fn target_is_in_water(&self) -> bool {
        self.0.is_in_water()
    }
    fn target_is_crouching(&self) -> bool {
        self.0.entity.is_crouching()
    }
    fn target_is_sprinting(&self) -> bool {
        self.0.entity.is_sprinting()
    }
    fn target_is_swimming(&self) -> bool {
        self.0.entity.is_swimming()
    }
    fn target_is_fall_flying(&self) -> bool {
        self.0.entity.is_fall_flying()
    }
    fn target_is_flying(&self) -> bool {
        self.0.entity.is_flying()
    }
    fn target_vehicle(&self) -> Option<&Entity> {
        self.0.entity.get_vehicle()
    }
    fn target_passengers(&self) -> Vec<&Entity> {
        self.0.entity.get_passengers()
    }
    fn target_tick_count(&self) -> i32 {
        self.0.entity.tick_count
    }
    fn target_team_name(&self) -> Option<String> {
        self.0.entity.get_team().map(|t| t.get_name().to_string())
    }
    fn target_effect(
        &self,
        id: &Identifier,
    ) -> Option<crate::world::effect::mob_effect_instance::MobEffectInstance> {
        // Holder<MobEffect> по identifier — через реестр ключей
        let key = crate::resources::resource_key::ResourceKey::create_with_registry_name(
            crate::core::registry::registries::mob_effect(),
            id.clone(),
        );
        let holder = crate::world::effect::mob_effects::holder_of(key);
        self.0.get_effect_instance(&holder).cloned()
    }
    fn target_equipment(
        &self,
        slot: crate::entity::equipment_slot::EquipmentSlot,
    ) -> Option<&crate::world::item::item_stack::ItemStack> {
        Some(self.0.get_item_slot(slot))
    }
    fn target_save_nbt(&self) -> crate::nbt::compound::CompoundTag {
        let mut tag = crate::nbt::compound::CompoundTag::new();
        self.0.entity.save(&mut tag);
        tag
    }
}
