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
