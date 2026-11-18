//! Перенос `net.minecraft.world.level.block.state.StateHolder`-ветки
//! для жидкостей — `FluidState` (`StateHolder<Fluid, FluidState>`).

use super::state_holder::StateHolder;
use crate::resources::identifier::Identifier;

/// `Fluid` — реестровая запись жидкости.
#[derive(Clone)]
pub struct Fluid {
    /// Registry-ключ (`minecraft:water`).
    pub id: Identifier,
    /// `FluidState`-определения (уровень и т.п.).
    pub state_definition: Option<std::sync::Arc<super::state_definition::StateDefinition>>,
    /// Источник/текучая жидкость.
    pub is_source: bool,
}

/// Глобальный реестр жидкостей.
pub fn fluid_registry() -> &'static std::sync::RwLock<Vec<std::sync::Arc<Fluid>>> {
    static REG: std::sync::OnceLock<std::sync::RwLock<Vec<std::sync::Arc<Fluid>>>> = std::sync::OnceLock::new();
    REG.get_or_init(|| std::sync::RwLock::new(Vec::new()))
}

impl Fluid {
    /// `new`.
    pub fn new(id: Identifier, is_source: bool) -> Self {
        Self { id, state_definition: None, is_source }
    }

    /// `getBucket()` — ведро жидкости (`None` для empty).
    pub fn get_bucket(&self) -> Option<Identifier> {
        if self.id == Identifier::minecraft("empty") {
            None
        } else {
            Identifier::with_default_namespace(&format!("{}_bucket", self.id.get_path())).ok()
        }
    }

    /// `isEmpty()`.
    pub fn is_empty(&self) -> bool {
        self.id == Identifier::minecraft("empty")
    }

    /// `isRandomlyTicking()`.
    pub fn is_randomly_ticking(&self) -> bool {
        false
    }

    /// `getTickDelay(level)` — задержка тика жидкости.
    pub fn get_tick_delay(&self, _level: &dyn crate::world::level::level_reader::LevelReader) -> i32 {
        5
    }

    /// `getFlow(level, pos, state)` — вектор течения.
    pub fn get_flow(&self, _level: &dyn crate::world::level::level_height_accessor::LevelHeightAccessor, _pos: crate::core::block_pos::BlockPos, _state: &FluidState) -> crate::world::phys::vec3::Vec3 {
        crate::world::phys::vec3::Vec3::ZERO
    }

    /// `getHeight(state, level, pos)` — высота поверхности.
    pub fn get_height(&self, _state: &FluidState, _level: &dyn crate::world::level::level_height_accessor::LevelHeightAccessor, _pos: crate::core::block_pos::BlockPos) -> f32 {
        if self.is_source { 1.0 } else { 0.875 }
    }

    /// `getOwnHeight(state)` — собственная высота (без level).
    pub fn get_own_height(&self, _state: &FluidState) -> f32 {
        if self.is_source { 1.0 } else { 0.875 }
    }

    /// `isSource(state)`.
    pub fn is_source_state(&self, state: &FluidState) -> bool {
        let _ = state;
        self.is_source
    }

    /// `getAmount(state)`.
    pub fn get_amount(&self, state: &FluidState) -> i32 {
        let _ = state;
        if self.is_source { 8 } else { 0 }
    }

    /// `canBeReplacedWith(state, level, pos, fluid, direction)`.
    pub fn can_be_replaced_with(&self, _state: &FluidState, _level: &dyn crate::world::level::level_height_accessor::LevelHeightAccessor, _pos: crate::core::block_pos::BlockPos, _fluid: &Fluid, _direction: crate::core::direction::Direction) -> bool {
        false
    }

    /// `getExplosionResistance()`.
    pub fn get_explosion_resistance(&self) -> f32 {
        if self.id == Identifier::minecraft("lava") { 100.0 } else { 100.0 }
    }

    /// `getShape(state, level, pos)` — форма (AABB).
    pub fn get_shape(&self, _state: &FluidState, _level: &dyn crate::world::level::level_height_accessor::LevelHeightAccessor, _pos: crate::core::block_pos::BlockPos) -> Option<crate::world::phys::aabb::Aabb> {
        None
    }

    /// `tick(level, pos, blockState, fluidState)`.
    pub fn tick(&self, _level: &mut dyn crate::world::level::level_reader::LevelReader, _pos: crate::core::block_pos::BlockPos, _block_state: &crate::world::level::block::state::block_state::BlockState, _fluid_state: &FluidState) {}

    /// `randomTick(level, pos, fluidState, random)`.
    pub fn random_tick(&self, _level: &mut dyn crate::world::level::level_reader::LevelReader, _pos: crate::core::block_pos::BlockPos, _state: &FluidState, _random: &mut dyn crate::random::RandomSource) {}

    /// `animateTick(level, pos, state, random)`.
    pub fn animate_tick(&self, _level: &mut dyn crate::world::level::level_reader::LevelReader, _pos: crate::core::block_pos::BlockPos, _state: &FluidState, _random: &mut dyn crate::random::RandomSource) {}

    /// `entityInside(level, pos, entity, effectApplier)` — без эффекта по умолчанию.
    pub fn entity_inside(&self, _level: &mut dyn crate::world::level::level_reader::LevelReader, _pos: crate::core::block_pos::BlockPos, _entity: &mut crate::world::entity::entity::Entity) {}

    /// `getDripParticle()`.
    pub fn get_drip_particle(&self) -> Option<crate::core::particles::particle_options::ParticleOptions> {
        None
    }

    /// `createLegacyBlock(state)` — блок-наследие (`None` для empty/базовых жидкостей).
    pub fn create_legacy_block(&self, _state: &FluidState) -> Option<crate::world::level::block::state::block_state::BlockState> {
        None
    }
}

/// `FluidState` — состояние жидкости (уровень, falling и т.п.).
#[derive(Clone)]
pub struct FluidState {
    /// `Fluid`-тип.
    pub fluid: Fluid,
    /// Значения свойств.
    pub values: StateHolder,
}

impl FluidState {
    /// Пустое состояние (нет жидкости).
    pub fn empty() -> Self {
        Self { fluid: Fluid::new(Identifier::minecraft("empty"), false), values: StateHolder::new() }
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.fluid.id == Identifier::minecraft("empty")
    }

    /// `is(FluidTags.WATER)` — жидкость является водой
    /// (включая текущую воду; `bubble_column` — нет).
    pub fn is_water(&self) -> bool {
        self.fluid.id == Identifier::minecraft("water")
            || self.is_in_named_tag(&crate::tags::fluid_tags::FluidTags::water())
    }

    /// `is(FluidTags.LAVA)`.
    pub fn is_lava(&self) -> bool {
        self.fluid.id == Identifier::minecraft("lava")
            || self.is_in_named_tag(&crate::tags::fluid_tags::FluidTags::lava())
    }

    /// `is(TagKey<Fluid>)` по именованному тегу.
    pub fn is_in_named_tag(&self, tag_location: &Identifier) -> bool {
        !self.is_empty() && crate::tags::is_fluid_in_named_tag(&self.fluid.id, tag_location)
    }
}

impl PartialEq for Fluid {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.is_source == other.is_source
    }
}
impl Eq for Fluid {}
impl std::fmt::Debug for Fluid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Fluid").field(&self.id).finish()
    }
}

impl PartialEq for FluidState {
    fn eq(&self, other: &Self) -> bool {
        self.fluid == other.fluid && self.values == other.values
    }
}
impl Eq for FluidState {}
impl std::fmt::Debug for FluidState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FluidState").field("fluid", &self.fluid.id).field("values", &self.values).finish()
    }
}

impl std::hash::Hash for Fluid {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.is_source.hash(state);
    }
}

impl std::hash::Hash for FluidState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fluid.hash(state);
        self.values.hash(state);
    }
}
