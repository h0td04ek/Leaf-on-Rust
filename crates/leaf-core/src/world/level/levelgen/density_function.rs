//! `net.minecraft.world.level.levelgen.DensityFunction` —
//! интерфейс плотностной функции + контексты.

use crate::core::holder::Holder;
use crate::world::level::levelgen::blending::blender::Blender;
use crate::world::level::levelgen::synth::normal_noise::{NormalNoise, NoiseParameters};

/// `DensityFunction` — функция плотности в точке генерации.
///
/// Реализации — варианты `DensityFunctions` enum (см. `density_functions.rs`).
/// `map_all`/`fill_array` требуют `&mut` доступа к подфункциям —
/// в Rust они реализованы через `Visitor`, который возвращает
/// замену (как `mapAll` в Java).
pub trait DensityFunction: Send + Sync {
    /// `compute(context)`.
    fn compute(&self, context: &dyn FunctionContext) -> f64;

    /// `fillArray(array, contextProvider)` — заполнить массив значений.
    fn fill_array(&self, array: &mut [f64], context_provider: &dyn ContextProvider);

    /// `mapAll(visitor)` — применить visitor ко всем подфункциям.
    /// Возвращает новую функцию (владение через Box).
    fn map_all(&self, visitor: &mut dyn Visitor) -> Box<dyn DensityFunction>;

    /// `asAny` — downcast к конкретной реализации (`DensityFn`).
    fn as_any(&self) -> &dyn std::any::Any;

    /// `minValue`.
    fn min_value(&self) -> f64;
    /// `maxValue`.
    fn max_value(&self) -> f64;
}

/// `DensityFunction.ContextProvider`.
pub trait ContextProvider: Send + Sync {
    /// `forIndex(arrayIndex)`.
    fn for_index(&self, array_index: usize) -> Box<dyn FunctionContext>;
    /// `fillAllDirectly(values, function)`.
    fn fill_all_directly(&self, values: &mut [f64], function: &dyn DensityFunction);
}

/// `DensityFunction.FunctionContext`.
pub trait FunctionContext: Send + Sync {
    /// `blockX`.
    fn block_x(&self) -> i32;
    /// `blockY`.
    fn block_y(&self) -> i32;
    /// `blockZ`.
    fn block_z(&self) -> i32;

    /// Клонирование контекста (для `Spline.Point`, который хранит контекст).
    fn clone_context(&self) -> Box<dyn FunctionContext>;

    /// `getBlender` — пустой blender по умолчанию.
    fn get_blender(&self) -> Blender {
        Blender::empty()
    }
}

/// `DensityFunction.SinglePointContext`.
#[derive(Clone, Copy, Debug)]
pub struct SinglePointContext {
    block_x: i32,
    block_y: i32,
    block_z: i32,
}

impl SinglePointContext {
    /// `new SinglePointContext(x, y, z)`.
    pub fn new(block_x: i32, block_y: i32, block_z: i32) -> Self {
        Self { block_x, block_y, block_z }
    }
}

impl FunctionContext for SinglePointContext {
    fn block_x(&self) -> i32 {
        self.block_x
    }
    fn block_y(&self) -> i32 {
        self.block_y
    }
    fn block_z(&self) -> i32 {
        self.block_z
    }
    fn clone_context(&self) -> Box<dyn FunctionContext> {
        Box::new(*self)
    }
}

impl std::fmt::Debug for dyn DensityFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DensityFunction")
    }
}

/// `DensityFunction.Visitor` — трансформер функций.
pub trait Visitor {
    /// `apply(densityFunction)`.
    fn apply(&mut self, function: &dyn DensityFunction) -> Box<dyn DensityFunction>;

    /// `visitNoise(noiseHolder)`.
    fn visit_noise(&mut self, noise_holder: &NoiseHolder) -> NoiseHolder {
        noise_holder.clone()
    }
}

/// `DensityFunction.NoiseHolder` — Holder<NoiseParameters> + разрешённый шум.
#[derive(Clone)]
pub struct NoiseHolder {
    /// `noiseData`.
    pub noise_data: Holder<NoiseParameters>,
    /// `noise` (None до bootstrap'а реестра шумов).
    pub noise: Option<NormalNoise>,
}

impl NoiseHolder {
    /// `new NoiseHolder(noiseData)`.
    pub fn new(noise_data: Holder<NoiseParameters>) -> Self {
        Self { noise_data, noise: None }
    }

    /// `getValue(x, y, z)` — 0.0 без разрешённого шума.
    pub fn get_value(&self, x: f64, y: f64, z: f64) -> f64 {
        match &self.noise {
            None => 0.0,
            Some(n) => n.get_value(x, y, z),
        }
    }

    /// `maxValue`.
    pub fn max_value(&self) -> f64 {
        match &self.noise {
            None => 2.0,
            Some(n) => n.max_value(),
        }
    }
}
