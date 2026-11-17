//! `net.minecraft.world.level.levelgen.DensityFunctions` —
//! все реализации `DensityFunction` (enum вместо Java-записей).

use super::density_function::{ContextProvider, DensityFunction, FunctionContext, NoiseHolder, SinglePointContext, Visitor};
use super::synth::simplex_noise::SimplexNoise;
use crate::core::holder::Holder;
use crate::random::legacy::LegacyRandomSource;
use crate::random::RandomSource;
use crate::util::bounded_float_function::{BoundedFloatFn, BoundedFloatFunction};
use crate::util::cubic_spline::CubicSpline;
use crate::util::mth;
use crate::util::string_representable::StringRepresentable;
use std::sync::Arc;

/// `SharedFunction` — разделяемая ссылка на функцию плотности.
pub type SharedFunction = Arc<dyn DensityFunction>;

fn sf(f: impl DensityFunction + 'static) -> SharedFunction {
    Arc::new(f)
}

impl DensityFunction for Arc<dyn DensityFunction> {
    fn compute(&self, context: &dyn FunctionContext) -> f64 {
        (**self).compute(context)
    }
    fn fill_array(&self, array: &mut [f64], context_provider: &dyn ContextProvider) {
        (**self).fill_array(array, context_provider)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        (**self).as_any()
    }
    fn map_all(&self, visitor: &mut dyn Visitor) -> Box<dyn DensityFunction> {
        (**self).map_all(visitor)
    }
    fn min_value(&self) -> f64 {
        (**self).min_value()
    }
    fn max_value(&self) -> f64 {
        (**self).max_value()
    }
}

/// `DensityFunction` — конкретная реализация (enum-варианты Java-записей).
#[derive(Clone)]
pub enum DensityFn {
    /// `Constant`.
    Constant(f64),
    /// `Ap2` — двухаргументная функция.
    Ap2 {
        /// Тип операции.
        ty: TwoArgType,
        /// `argument1`.
        argument1: SharedFunction,
        /// `argument2`.
        argument2: SharedFunction,
        /// min/max (предвычисленные).
        min_value: f64,
        max_value: f64,
    },
    /// `MulOrAdd`.
    MulOrAdd {
        /// `specificType`.
        ty: MulOrAddType,
        /// `input`.
        input: SharedFunction,
        /// min/max.
        min_value: f64,
        max_value: f64,
        /// `argument`.
        argument: f64,
    },
    /// `Mapped`.
    Mapped {
        /// `type`.
        ty: MappedType,
        /// `input`.
        input: SharedFunction,
        /// min/max.
        min_value: f64,
        max_value: f64,
    },
    /// `Marker`/`MarkerOrMarked` — маркеры кэширования/интерполяции.
    Marker {
        /// `type`.
        ty: MarkerType,
        /// `wrapped`.
        wrapped: SharedFunction,
    },
    /// `BlendDensity`.
    BlendDensity {
        /// `input`.
        input: SharedFunction,
    },
    /// `Clamp`.
    Clamp {
        /// `input`.
        input: SharedFunction,
        /// min/max.
        min_value: f64,
        max_value: f64,
    },
    /// `Noise`.
    Noise {
        /// `noise` (NoiseHolder).
        noise: NoiseHolder,
        /// `xzScale` (deprecated).
        xz_scale: f64,
        /// `yScale`.
        y_scale: f64,
    },
    /// `RangeChoice`.
    RangeChoice {
        /// `input`.
        input: SharedFunction,
        /// `minInclusive`.
        min_inclusive: f64,
        /// `maxExclusive`.
        max_exclusive: f64,
        /// `whenInRange`.
        when_in_range: SharedFunction,
        /// `whenOutOfRange`.
        when_out_of_range: SharedFunction,
    },
    /// `Shift`/`ShiftA`/`ShiftB` — `ShiftNoise`.
    ShiftNoise {
        /// `offsetNoise`.
        offset_noise: NoiseHolder,
        /// Какой вариант (0=Shift, 1=ShiftA, 2=ShiftB).
        variant: ShiftNoiseVariant,
    },
    /// `ShiftedNoise`.
    ShiftedNoise {
        /// `shiftX`.
        shift_x: SharedFunction,
        /// `shiftY`.
        shift_y: SharedFunction,
        /// `shiftZ`.
        shift_z: SharedFunction,
        /// `xzScale`.
        xz_scale: f64,
        /// `yScale`.
        y_scale: f64,
        /// `noise`.
        noise: NoiseHolder,
    },
    /// `Spline`.
    Spline {
        /// `spline`.
        spline: Arc<dyn CubicSpline<SplinePoint>>,
    },
    /// `WeirdScaledSampler`.
    WeirdScaledSampler {
        /// `input`.
        input: SharedFunction,
        /// `noise`.
        noise: NoiseHolder,
        /// `rarityValueMapper`.
        rarity_value_mapper: RarityValueMapper,
    },
    /// `FindTopSurface`.
    FindTopSurface {
        /// `density`.
        density: SharedFunction,
        /// `upperBound`.
        upper_bound: SharedFunction,
        /// `lowerBound`.
        lower_bound: i32,
        /// `cellHeight`.
        cell_height: i32,
    },
    /// `EndIslandDensityFunction`.
    EndIslands {
        /// `islandNoise`.
        island_noise: SimplexNoise,
        /// Paper noise cache.
        cache: Arc<std::sync::Mutex<NoiseCache>>,
    },
    /// `BlendAlpha` — константа 1.0 (unit codec).
    BlendAlpha,
    /// `BlendOffset` — константа 0.0.
    BlendOffset,
    /// `BeardifierMarker` — маркерbeardifier (0.0).
    BeardifierMarker,
    /// `YClampedGradient`.
    YClampedGradient {
        /// `fromY`.
        from_y: i32,
        /// `toY`.
        to_y: i32,
        /// `fromValue`.
        from_value: f64,
        /// `toValue`.
        to_value: f64,
    },
}

/// `TwoArgumentSimpleFunction.Type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TwoArgType {
    /// `ADD`.
    Add,
    /// `MUL`.
    Mul,
    /// `MIN`.
    Min,
    /// `MAX`.
    Max,
}

/// `MulOrAdd.Type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MulOrAddType {
    /// `MUL`.
    Mul,
    /// `ADD`.
    Add,
}

/// `Mapped.Type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MappedType {
    /// `ABS`.
    Abs,
    /// `SQUARE`.
    Square,
    /// `CUBE`.
    Cube,
    /// `HALF_NEGATIVE`.
    HalfNegative,
    /// `QUARTER_NEGATIVE`.
    QuarterNegative,
    /// `INVERT`.
    Invert,
    /// `SQUEEZE`.
    Squeeze,
}

impl MappedType {
    fn transform(self, value: f64) -> f64 {
        match self {
            MappedType::Abs => value.abs(),
            MappedType::Square => value * value,
            MappedType::Cube => value * value * value,
            MappedType::HalfNegative => if value > 0.0 { value } else { value * 0.5 },
            MappedType::QuarterNegative => if value > 0.0 { value } else { value * 0.25 },
            MappedType::Invert => 1.0 / value,
            MappedType::Squeeze => {
                let d = mth::clamp_f64(value, -1.0, 1.0);
                d / 2.0 - d * d * d / 24.0
            }
        }
    }
}

/// `Marker.Type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkerType {
    /// `Interpolated`.
    Interpolated,
    /// `FlatCache`.
    FlatCache,
    /// `Cache2D`.
    Cache2D,
    /// `CacheOnce`.
    CacheOnce,
    /// `CacheAllInCell`.
    CacheAllInCell,
}

/// `ShiftNoise` вариант.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShiftNoiseVariant {
    /// `Shift`.
    Shift,
    /// `ShiftA`.
    ShiftA,
    /// `ShiftB`.
    ShiftB,
}

/// `WeirdScaledSampler.RarityValueMapper`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RarityValueMapper {
    /// `TYPE1`.
    Type1,
    /// `TYPE2`.
    Type2,
}

impl RarityValueMapper {
    /// `mapper.get(value)` — spaghetti rarity.
    pub fn get(&self, value: f64) -> f64 {
        match self {
            // NoiseRouterData.QuantizedSpaghettiRarity
            RarityValueMapper::Type1 => spaghetti_rarity_3d(value),
            RarityValueMapper::Type2 => spaghetti_rarity_2d(value),
        }
    }

    /// `maxRarity`.
    pub fn max_rarity(&self) -> f64 {
        match self {
            RarityValueMapper::Type1 => 2.0,
            RarityValueMapper::Type2 => 3.0,
        }
    }
}

/// `Spline.Point` — контекст точки для сплайна.
pub struct SplinePoint {
    /// `context`.
    pub context: Box<dyn FunctionContext>,
}

impl Clone for SplinePoint {
    fn clone(&self) -> Self {
        Self { context: self.context.clone_context() }
    }
}

impl SplinePoint {
    /// `new Point(context)`.
    pub fn new(context: &dyn FunctionContext) -> Self {
        Self { context: context.clone_context() }
    }
}

/// `Spline.Coordinate` — Holder<DF>-координата сплайна.
#[derive(Clone)]
pub struct SplineCoordinate {
    /// `function`.
    pub function: Holder<SharedFunction>,
}

impl BoundedFloatFunction<SplinePoint> for SplineCoordinate {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn apply(&self, value: SplinePoint) -> f32 {
        let f = self.function.value().map(|f| f.compute(&*value.context)).unwrap_or(0.0);
        f as f32
    }

    fn min_value(&self) -> f32 {
        self.function
            .value()
            .map(|f| f.min_value() as f32)
            .unwrap_or(f32::NEG_INFINITY)
    }

    fn max_value(&self) -> f32 {
        self.function
            .value()
            .map(|f| f.max_value() as f32)
            .unwrap_or(f32::INFINITY)
    }
}

impl StringRepresentable for SplineCoordinate {
    fn get_serialized_name(&self) -> String {
        format!("Coordinate[{:?}]", self.function)
    }
}

/// Paper noise cache для EndIslandDensityFunction.
pub struct NoiseCache {
    /// `keys`.
    pub keys: [i64; 8192],
    /// `values`.
    pub values: [f32; 8192],
}

impl NoiseCache {
    fn new() -> Self {
        let mut keys = [0i64; 8192];
        keys.fill(i64::MIN);
        Self { keys, values: [0.0; 8192] }
    }
}

// ---- DensityFunctions.* фабрики ----

/// `interpolated`.
pub fn interpolated(wrapped: SharedFunction) -> SharedFunction {
    sf(DensityFn::Marker { ty: MarkerType::Interpolated, wrapped })
}
/// `flatCache`.
pub fn flat_cache(wrapped: SharedFunction) -> SharedFunction {
    sf(DensityFn::Marker { ty: MarkerType::FlatCache, wrapped })
}
/// `cache2d`.
pub fn cache2d(wrapped: SharedFunction) -> SharedFunction {
    sf(DensityFn::Marker { ty: MarkerType::Cache2D, wrapped })
}
/// `cacheOnce`.
pub fn cache_once(wrapped: SharedFunction) -> SharedFunction {
    sf(DensityFn::Marker { ty: MarkerType::CacheOnce, wrapped })
}
/// `cacheAllInCell`.
pub fn cache_all_in_cell(wrapped: SharedFunction) -> SharedFunction {
    sf(DensityFn::Marker { ty: MarkerType::CacheAllInCell, wrapped })
}
/// `mappedNoise` (полная).
pub fn mapped_noise_full(noise_data: Holder<crate::world::level::levelgen::synth::normal_noise::NoiseParameters>, xz_scale: f64, y_scale: f64, from_y: f64, to_y: f64) -> SharedFunction {
    map_from_unit_to(noise(noise_data, xz_scale, y_scale), from_y, to_y)
}
/// `mappedNoise` (yScale, fromY, toY).
pub fn mapped_noise(noise_data: Holder<crate::world::level::levelgen::synth::normal_noise::NoiseParameters>, y_scale: f64, from_y: f64, to_y: f64) -> SharedFunction {
    mapped_noise_full(noise_data, 1.0, y_scale, from_y, to_y)
}
/// `mappedNoise` (fromY, toY).
pub fn mapped_noise_simple(noise_data: Holder<crate::world::level::levelgen::synth::normal_noise::NoiseParameters>, from_y: f64, to_y: f64) -> SharedFunction {
    mapped_noise_full(noise_data, 1.0, 1.0, from_y, to_y)
}
/// `shiftedNoise2d`.
pub fn shifted_noise2d(shift_x: SharedFunction, shift_z: SharedFunction, xz_scale: f64, noise_data: Holder<crate::world::level::levelgen::synth::normal_noise::NoiseParameters>) -> SharedFunction {
    sf(DensityFn::ShiftedNoise {
        shift_x,
        shift_y: zero(),
        shift_z,
        xz_scale,
        y_scale: 0.0,
        noise: NoiseHolder::new(noise_data),
    })
}
/// `noise(noiseData)`.
pub fn noise(noise_data: Holder<crate::world::level::levelgen::synth::normal_noise::NoiseParameters>, xz_scale: f64, y_scale: f64) -> SharedFunction {
    sf(DensityFn::Noise { noise: NoiseHolder::new(noise_data), xz_scale, y_scale })
}
/// `noise(noiseData, xzScale, yScale)` — то же.
pub fn noise_full(noise_data: Holder<crate::world::level::levelgen::synth::normal_noise::NoiseParameters>, xz_scale: f64, y_scale: f64) -> SharedFunction {
    noise(noise_data, xz_scale, y_scale)
}
/// `noise(noiseData, yScale)`.
pub fn noise_y(noise_data: Holder<crate::world::level::levelgen::synth::normal_noise::NoiseParameters>, y_scale: f64) -> SharedFunction {
    noise(noise_data, 1.0, y_scale)
}
/// `rangeChoice`.
pub fn range_choice(input: SharedFunction, min_inclusive: f64, max_exclusive: f64, when_in_range: SharedFunction, when_out_of_range: SharedFunction) -> SharedFunction {
    sf(DensityFn::RangeChoice { input, min_inclusive, max_exclusive, when_in_range, when_out_of_range })
}
/// `shiftA`.
pub fn shift_a(noise_data: Holder<crate::world::level::levelgen::synth::normal_noise::NoiseParameters>) -> SharedFunction {
    sf(DensityFn::ShiftNoise { offset_noise: NoiseHolder::new(noise_data), variant: ShiftNoiseVariant::ShiftA })
}
/// `shiftB`.
pub fn shift_b(noise_data: Holder<crate::world::level::levelgen::synth::normal_noise::NoiseParameters>) -> SharedFunction {
    sf(DensityFn::ShiftNoise { offset_noise: NoiseHolder::new(noise_data), variant: ShiftNoiseVariant::ShiftB })
}
/// `shift`.
pub fn shift(noise_data: Holder<crate::world::level::levelgen::synth::normal_noise::NoiseParameters>) -> SharedFunction {
    sf(DensityFn::ShiftNoise { offset_noise: NoiseHolder::new(noise_data), variant: ShiftNoiseVariant::Shift })
}
/// `blendDensity`.
pub fn blend_density(input: SharedFunction) -> SharedFunction {
    sf(DensityFn::BlendDensity { input })
}
/// `endIslands`.
pub fn end_islands(seed: i64) -> SharedFunction {
    let mut random: Box<dyn RandomSource> = Box::new(LegacyRandomSource::new(seed));
    random.consume_count(17292);
    sf(DensityFn::EndIslands { island_noise: SimplexNoise::new(&mut *random), cache: Arc::new(std::sync::Mutex::new(NoiseCache::new())) })
}
/// `weirdScaledSampler`.
pub fn weird_scaled_sampler(input: SharedFunction, noise_data: Holder<crate::world::level::levelgen::synth::normal_noise::NoiseParameters>, rarity_value_mapper: RarityValueMapper) -> SharedFunction {
    sf(DensityFn::WeirdScaledSampler { input, noise: NoiseHolder::new(noise_data), rarity_value_mapper })
}
/// `add`.
pub fn add(a1: SharedFunction, a2: SharedFunction) -> SharedFunction {
    two_arg_simple(TwoArgType::Add, a1, a2)
}
/// `mul`.
pub fn mul(a1: SharedFunction, a2: SharedFunction) -> SharedFunction {
    two_arg_simple(TwoArgType::Mul, a1, a2)
}
/// `min`.
pub fn min(a1: SharedFunction, a2: SharedFunction) -> SharedFunction {
    two_arg_simple(TwoArgType::Min, a1, a2)
}
/// `max`.
pub fn max(a1: SharedFunction, a2: SharedFunction) -> SharedFunction {
    two_arg_simple(TwoArgType::Max, a1, a2)
}
/// `spline`.
pub fn spline(spline_fn: Arc<dyn CubicSpline<SplinePoint>>) -> SharedFunction {
    sf(DensityFn::Spline { spline: spline_fn })
}
/// `zero`.
pub fn zero() -> SharedFunction {
    sf(DensityFn::Constant(0.0))
}
/// `constant`.
pub fn constant(value: f64) -> SharedFunction {
    sf(DensityFn::Constant(value))
}
/// `yClampedGradient`.
pub fn y_clamped_gradient(from_y: i32, to_y: i32, from_value: f64, to_value: f64) -> SharedFunction {
    sf(DensityFn::YClampedGradient { from_y, to_y, from_value, to_value })
}
/// `map`.
pub fn map(input: SharedFunction, ty: MappedType) -> SharedFunction {
    mapped_create(ty, input)
}
/// `mapFromUnitTo`.
fn map_from_unit_to(density_function: SharedFunction, from_y: f64, to_y: f64) -> SharedFunction {
    let d = (from_y + to_y) * 0.5;
    let d1 = (to_y - from_y) * 0.5;
    add(constant(d), mul(constant(d1), density_function))
}
/// `blendAlpha`.
pub fn blend_alpha() -> SharedFunction {
    sf(DensityFn::BlendAlpha)
}
/// `blendOffset`.
pub fn blend_offset() -> SharedFunction {
    sf(DensityFn::BlendOffset)
}
/// `lerp(delta, minFn, maxFn)`.
pub fn lerp(delta_function: SharedFunction, min_function: SharedFunction, max_function: SharedFunction) -> SharedFunction {
    if let Some(DensityFn::Constant(constant_value)) = min_function.as_any().downcast_ref::<DensityFn>() {
        return lerp_constant(delta_function, *constant_value, max_function);
    }
    let density_function = cache_once(delta_function);
    let density_function1 = add(mul(density_function.clone(), constant(-1.0)), constant(1.0));
    add(mul(min_function, density_function1), mul(max_function, density_function))
}
/// `lerp(delta, min, maxFn)`.
pub fn lerp_constant(delta_function: SharedFunction, min: f64, max_function: SharedFunction) -> SharedFunction {
    add(mul(delta_function, add(max_function, constant(-min))), constant(min))
}
/// `findTopSurface`.
pub fn find_top_surface(density: SharedFunction, upper_bound: SharedFunction, lower_bound: i32, cell_height: i32) -> SharedFunction {
    sf(DensityFn::FindTopSurface { density, upper_bound, lower_bound, cell_height })
}

/// `TwoArgumentSimpleFunction.create`.
pub fn two_arg_simple(ty: TwoArgType, argument1: SharedFunction, argument2: SharedFunction) -> SharedFunction {
    let d = argument1.min_value();
    let d1 = argument2.min_value();
    let d2 = argument1.max_value();
    let d3 = argument2.max_value();
    let (d4, d5) = match ty {
        TwoArgType::Add => (d + d1, d2 + d3),
        TwoArgType::Mul => (
            if d > 0.0 && d1 > 0.0 { d * d1 } else if d2 < 0.0 && d3 < 0.0 { d2 * d3 } else { (d * d3).min(d2 * d1) },
            if d > 0.0 && d1 > 0.0 { d2 * d3 } else if d2 < 0.0 && d3 < 0.0 { d * d1 } else { (d * d1).max(d2 * d3) },
        ),
        TwoArgType::Min => (d.min(d1), d2.min(d3)),
        TwoArgType::Max => (d.max(d1), d2.max(d3)),
    };
    // MulOrAdd-оптимизация для констант.
    if matches!(ty, TwoArgType::Mul | TwoArgType::Add) {
        let mul_or_add_ty = if ty == TwoArgType::Add { MulOrAddType::Add } else { MulOrAddType::Mul };
        if let Some(DensityFn::Constant(constant_value)) = argument1.as_any().downcast_ref::<DensityFn>() {
            let cv = *constant_value;
            return sf(DensityFn::MulOrAdd { ty: mul_or_add_ty, input: argument2, min_value: d4, max_value: d5, argument: cv });
        }
        if let Some(DensityFn::Constant(constant_value)) = argument2.as_any().downcast_ref::<DensityFn>() {
            let cv = *constant_value;
            return sf(DensityFn::MulOrAdd { ty: mul_or_add_ty, input: argument1, min_value: d4, max_value: d5, argument: cv });
        }
    }
    sf(DensityFn::Ap2 { ty, argument1, argument2, min_value: d4, max_value: d5 })
}

/// `Mapped.create`.
pub fn mapped_create(ty: MappedType, input: SharedFunction) -> SharedFunction {
    let d = input.min_value();
    let d1 = input.max_value();
    let d2 = ty.transform(d);
    let d3 = ty.transform(d1);
    if ty == MappedType::Invert {
        if d < 0.0 && d1 > 0.0 {
            return sf(DensityFn::Mapped { ty, input, min_value: f64::NEG_INFINITY, max_value: f64::INFINITY });
        }
        return sf(DensityFn::Mapped { ty, input, min_value: d3, max_value: d2 });
    }
    if ty != MappedType::Abs && ty != MappedType::Square {
        return sf(DensityFn::Mapped { ty, input, min_value: d2, max_value: d3 });
    }
    sf(DensityFn::Mapped { ty, input, min_value: 0.0f64.max(d), max_value: d2.max(d3) })
}

/// `QuantizedSpaghettiRarity.getSpaghettiRarity3D`.
fn spaghetti_rarity_3d(value: f64) -> f64 {
    let d = mth::square_f64(value);
    d * d * 0.6 + d * 0.5 + 1.0
}

/// `QuantizedSpaghettiRarity.getSphaghettiRarity2D`.
fn spaghetti_rarity_2d(value: f64) -> f64 {
    let d = mth::square_f64(value);
    d * d * 0.6 + d * 0.5 + 1.0
}

impl DensityFunction for DensityFn {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn compute(&self, context: &dyn FunctionContext) -> f64 {
        match self {
            DensityFn::Constant(value) => *value,
            DensityFn::Ap2 { ty, argument1, argument2, .. } => {
                let d = argument1.compute(context);
                match ty {
                    TwoArgType::Add => d + argument2.compute(context),
                    TwoArgType::Mul => if d == 0.0 { 0.0 } else { d * argument2.compute(context) },
                    TwoArgType::Min => if d < argument2.min_value() { d } else { d.min(argument2.compute(context)) },
                    TwoArgType::Max => if d > argument2.max_value() { d } else { d.max(argument2.compute(context)) },
                }
            }
            DensityFn::MulOrAdd { ty, input, argument, .. } => {
                let value = input.compute(context);
                match ty {
                    MulOrAddType::Mul => value * argument,
                    MulOrAddType::Add => value + argument,
                }
            }
            DensityFn::Mapped { ty, input, .. } => ty.transform(input.compute(context)),
            DensityFn::Marker { wrapped, .. } | DensityFn::BlendDensity { input: wrapped } => wrapped.compute(context),
            DensityFn::Clamp { input, min_value, max_value } => mth::clamp_f64(input.compute(context), *min_value, *max_value),
            DensityFn::Noise { noise, xz_scale, y_scale } => {
                noise.get_value(context.block_x() as f64 * xz_scale, context.block_y() as f64 * y_scale, context.block_z() as f64 * xz_scale)
            }
            DensityFn::RangeChoice { input, min_inclusive, max_exclusive, when_in_range, when_out_of_range } => {
                let d = input.compute(context);
                if d >= *min_inclusive && d < *max_exclusive {
                    when_in_range.compute(context)
                } else {
                    when_out_of_range.compute(context)
                }
            }
            DensityFn::ShiftNoise { offset_noise, variant } => {
                let (x, y, z) = match variant {
                    ShiftNoiseVariant::Shift => (context.block_x() as f64, context.block_y() as f64, context.block_z() as f64),
                    ShiftNoiseVariant::ShiftA => (context.block_x() as f64, 0.0, context.block_z() as f64),
                    ShiftNoiseVariant::ShiftB => (context.block_z() as f64, context.block_x() as f64, 0.0),
                };
                offset_noise.get_value(x * 0.25, y * 0.25, z * 0.25) * 4.0
            }
            DensityFn::ShiftedNoise { shift_x, shift_y, shift_z, xz_scale, y_scale, noise } => {
                let d = context.block_x() as f64 * xz_scale + shift_x.compute(context);
                let d1 = context.block_y() as f64 * y_scale + shift_y.compute(context);
                let d2 = context.block_z() as f64 * xz_scale + shift_z.compute(context);
                noise.get_value(d, d1, d2)
            }
            DensityFn::Spline { spline } => {
                spline.apply(SplinePoint::new(context)) as f64
            }
            DensityFn::WeirdScaledSampler { input, noise, rarity_value_mapper } => {
                let d = rarity_value_mapper.get(input.compute(context));
                d * noise.get_value(context.block_x() as f64 / d, context.block_y() as f64 / d, context.block_z() as f64 / d).abs()
            }
            DensityFn::FindTopSurface { density, upper_bound, lower_bound, cell_height } => {
                let i = mth::floor(upper_bound.compute(context) / *cell_height as f64) * *cell_height;
                if i <= *lower_bound {
                    return *lower_bound as f64;
                }
                let mut i1 = i;
                while i1 >= *lower_bound {
                    let point = SinglePointContext::new(context.block_x(), i1, context.block_z());
                    if density.compute(&point) > 0.0 {
                        return i1 as f64;
                    }
                    i1 -= cell_height;
                }
                *lower_bound as f64
            }
            DensityFn::EndIslands { island_noise, cache } => {
                let h = end_island_height_value(island_noise, cache, context.block_x() / 8, context.block_z() / 8);
                (h as f64 - 8.0) / 128.0
            }
            DensityFn::BlendAlpha => 1.0,
            DensityFn::BlendOffset | DensityFn::BeardifierMarker => 0.0,
            DensityFn::YClampedGradient { from_y, to_y, from_value, to_value } => {
                mth::clamped_map_f64(context.block_y() as f64, *from_y as f64, *to_y as f64, *from_value, *to_value)
            }
        }
    }

    fn fill_array(&self, array: &mut [f64], context_provider: &dyn ContextProvider) {
        match self {
            DensityFn::Constant(value) => array.fill(*value),
            DensityFn::Ap2 { ty, argument1, argument2, .. } => {
                argument1.fill_array(array, context_provider);
                match ty {
                    TwoArgType::Add => {
                        let mut doubles = vec![0.0; array.len()];
                        argument2.fill_array(&mut doubles, context_provider);
                        for (a, d) in array.iter_mut().zip(doubles.iter()) {
                            *a += d;
                        }
                    }
                    TwoArgType::Mul => {
                        for (i, a) in array.iter_mut().enumerate() {
                            let d = *a;
                            *a = if d == 0.0 { 0.0 } else { d * argument2.compute(&*context_provider.for_index(i)) };
                        }
                    }
                    TwoArgType::Min => {
                        let d1 = argument2.min_value();
                        for (i, a) in array.iter_mut().enumerate() {
                            let d2 = *a;
                            *a = if d2 < d1 { d2 } else { d2.min(argument2.compute(&*context_provider.for_index(i))) };
                        }
                    }
                    TwoArgType::Max => {
                        let d1 = argument2.max_value();
                        for (i, a) in array.iter_mut().enumerate() {
                            let d2 = *a;
                            *a = if d2 > d1 { d2 } else { d2.max(argument2.compute(&*context_provider.for_index(i))) };
                        }
                    }
                }
            }
            DensityFn::MulOrAdd { ty, input, argument, .. } => {
                input.fill_array(array, context_provider);
                for a in array.iter_mut() {
                    *a = match ty {
                        MulOrAddType::Mul => *a * argument,
                        MulOrAddType::Add => *a + argument,
                    };
                }
            }
            DensityFn::Mapped { ty, input, .. } => {
                input.fill_array(array, context_provider);
                for a in array.iter_mut() {
                    *a = ty.transform(*a);
                }
            }
            DensityFn::Marker { wrapped, .. } | DensityFn::BlendDensity { input: wrapped } => wrapped.fill_array(array, context_provider),
            DensityFn::Clamp { input, min_value, max_value } => {
                input.fill_array(array, context_provider);
                for a in array.iter_mut() {
                    *a = mth::clamp_f64(*a, *min_value, *max_value);
                }
            }
            DensityFn::Noise { .. }
            | DensityFn::RangeChoice { .. }
            | DensityFn::ShiftNoise { .. }
            | DensityFn::ShiftedNoise { .. }
            | DensityFn::Spline { .. }
            | DensityFn::WeirdScaledSampler { .. }
            | DensityFn::FindTopSurface { .. }
            | DensityFn::EndIslands { .. } => context_provider.fill_all_directly(array, self),
            DensityFn::BlendAlpha => array.fill(1.0),
            DensityFn::BlendOffset | DensityFn::BeardifierMarker => array.fill(0.0),
            DensityFn::YClampedGradient { .. } => context_provider.fill_all_directly(array, self),
        }
    }

    fn map_all(&self, visitor: &mut dyn Visitor) -> Box<dyn DensityFunction> {
        match self {
            DensityFn::Constant(value) => Box::new(DensityFn::Constant(*value)),
            DensityFn::Ap2 { ty, argument1, argument2, min_value, max_value } => {
                let a1 = argument1.map_all(visitor);
                let a2 = argument2.map_all(visitor);
                Box::new(DensityFn::Ap2 {
                    ty: *ty,
                    argument1: Arc::from(a1),
                    argument2: Arc::from(a2),
                    min_value: *min_value,
                    max_value: *max_value,
                })
            }
            DensityFn::MulOrAdd { ty, input, min_value, max_value, argument } => {
                let i = input.map_all(visitor);
                Box::new(DensityFn::MulOrAdd { ty: *ty, input: Arc::from(i), min_value: *min_value, max_value: *max_value, argument: *argument })
            }
            DensityFn::Mapped { ty, input, .. } => {
                let i = input.map_all(visitor);
                Box::new(mapped_create(*ty, Arc::from(i)))
            }
            DensityFn::Marker { ty, wrapped } => {
                let w = wrapped.map_all(visitor);
                Box::new(DensityFn::Marker { ty: *ty, wrapped: Arc::from(w) })
            }
            DensityFn::BlendDensity { input } => {
                let i = input.map_all(visitor);
                Box::new(DensityFn::BlendDensity { input: Arc::from(i) })
            }
            DensityFn::Clamp { input, min_value, max_value } => {
                let i = input.map_all(visitor);
                Box::new(DensityFn::Clamp { input: Arc::from(i), min_value: *min_value, max_value: *max_value })
            }
            DensityFn::Noise { noise, xz_scale, y_scale } => {
                Box::new(DensityFn::Noise { noise: visitor.visit_noise(noise), xz_scale: *xz_scale, y_scale: *y_scale })
            }
            DensityFn::RangeChoice { input, min_inclusive, max_exclusive, when_in_range, when_out_of_range } => {
                let i = input.map_all(visitor);
                let ir = when_in_range.map_all(visitor);
                let or = when_out_of_range.map_all(visitor);
                Box::new(DensityFn::RangeChoice {
                    input: Arc::from(i),
                    min_inclusive: *min_inclusive,
                    max_exclusive: *max_exclusive,
                    when_in_range: Arc::from(ir),
                    when_out_of_range: Arc::from(or),
                })
            }
            DensityFn::ShiftNoise { offset_noise, variant } => {
                Box::new(DensityFn::ShiftNoise { offset_noise: visitor.visit_noise(offset_noise), variant: *variant })
            }
            DensityFn::ShiftedNoise { shift_x, shift_y, shift_z, xz_scale, y_scale, noise } => {
                let sx = shift_x.map_all(visitor);
                let sy = shift_y.map_all(visitor);
                let sz = shift_z.map_all(visitor);
                Box::new(DensityFn::ShiftedNoise {
                    shift_x: Arc::from(sx),
                    shift_y: Arc::from(sy),
                    shift_z: Arc::from(sz),
                    xz_scale: *xz_scale,
                    y_scale: *y_scale,
                    noise: visitor.visit_noise(noise),
                })
            }
            DensityFn::Spline { spline } => {
                // Сплайн координат маппится через visitor на каждую координату.
                let mut coord_mapper = |coord: &BoundedFloatFn<SplinePoint>| -> BoundedFloatFn<SplinePoint> {
                    if let Some(sc) = coord.as_any().downcast_ref::<SplineCoordinate>() {
                        let f = sc
                            .function
                            .value()
                            .map(|f| Arc::from(f.map_all(visitor)))
                            .unwrap_or_else(|| constant(0.0));
                        Arc::new(SplineCoordinate { function: Holder::Direct(f) })
                    } else {
                        coord.clone()
                    }
                };
                let mapped = spline.map_all(&mut coord_mapper);
                Box::new(DensityFn::Spline { spline: Arc::from(mapped) })
            }
            DensityFn::WeirdScaledSampler { input, noise, rarity_value_mapper } => {
                let i = input.map_all(visitor);
                Box::new(DensityFn::WeirdScaledSampler {
                    input: Arc::from(i),
                    noise: visitor.visit_noise(noise),
                    rarity_value_mapper: *rarity_value_mapper,
                })
            }
            DensityFn::FindTopSurface { density, upper_bound, lower_bound, cell_height } => {
                let d = density.map_all(visitor);
                let u = upper_bound.map_all(visitor);
                Box::new(DensityFn::FindTopSurface {
                    density: Arc::from(d),
                    upper_bound: Arc::from(u),
                    lower_bound: *lower_bound,
                    cell_height: *cell_height,
                })
            }
            DensityFn::EndIslands { .. } => Box::new(self.clone()),
            DensityFn::BlendAlpha | DensityFn::BlendOffset | DensityFn::BeardifierMarker | DensityFn::YClampedGradient { .. } => {
                Box::new(self.clone())
            }
        }
    }

    fn min_value(&self) -> f64 {
        match self {
            DensityFn::Constant(value) => *value,
            DensityFn::Ap2 { min_value, .. }
            | DensityFn::MulOrAdd { min_value, .. }
            | DensityFn::Mapped { min_value, .. }
            | DensityFn::Clamp { min_value, .. } => *min_value,
            DensityFn::Marker { wrapped, .. } | DensityFn::BlendDensity { input: wrapped } => wrapped.min_value(),
            DensityFn::Noise { noise, .. } => -noise.max_value(),
            DensityFn::RangeChoice { when_in_range, when_out_of_range, .. } => when_in_range.min_value().min(when_out_of_range.min_value()),
            DensityFn::ShiftNoise { offset_noise, .. } => -offset_noise.max_value() * 4.0,
            DensityFn::ShiftedNoise { noise, .. } => -noise.max_value(),
            DensityFn::Spline { spline } => spline.min_value() as f64,
            DensityFn::WeirdScaledSampler { rarity_value_mapper, noise, .. } => 0.0,
            DensityFn::FindTopSurface { lower_bound, .. } => *lower_bound as f64,
            DensityFn::EndIslands { .. } => -0.84375,
            DensityFn::BlendAlpha => 1.0,
            DensityFn::BlendOffset | DensityFn::BeardifierMarker => 0.0,
            DensityFn::YClampedGradient { from_value, to_value, .. } => from_value.min(*to_value),
        }
    }

    fn max_value(&self) -> f64 {
        match self {
            DensityFn::Constant(value) => *value,
            DensityFn::Ap2 { max_value, .. }
            | DensityFn::MulOrAdd { max_value, .. }
            | DensityFn::Mapped { max_value, .. }
            | DensityFn::Clamp { max_value, .. } => *max_value,
            DensityFn::Marker { wrapped, .. } | DensityFn::BlendDensity { input: wrapped } => wrapped.max_value(),
            DensityFn::Noise { noise, .. } => noise.max_value(),
            DensityFn::RangeChoice { when_in_range, when_out_of_range, .. } => when_in_range.max_value().max(when_out_of_range.max_value()),
            DensityFn::ShiftNoise { offset_noise, .. } => offset_noise.max_value() * 4.0,
            DensityFn::ShiftedNoise { noise, .. } => noise.max_value(),
            DensityFn::Spline { spline } => spline.max_value() as f64,
            DensityFn::WeirdScaledSampler { rarity_value_mapper, noise, .. } => rarity_value_mapper.max_rarity() * noise.max_value(),
            DensityFn::FindTopSurface { lower_bound, upper_bound, .. } => (*lower_bound as f64).max(upper_bound.max_value()),
            DensityFn::EndIslands { .. } => 0.5625,
            DensityFn::BlendAlpha => 1.0,
            DensityFn::BlendOffset | DensityFn::BeardifierMarker => 0.0,
            DensityFn::YClampedGradient { from_value, to_value, .. } => from_value.max(*to_value),
        }
    }
}

/// `EndIslandDensityFunction.getHeightValue`.
fn end_island_height_value(noise: &SimplexNoise, cache: &std::sync::Mutex<NoiseCache>, x: i32, z: i32) -> f32 {
    let i = x / 2;
    let i1 = z / 2;
    let i2 = x % 2;
    let i3 = z % 2;
    let mut f = 100.0f32 - mth::sqrt_f32((x * x + z * z) as f32) * 8.0;
    f = mth::clamp_f32(f, -100.0, 80.0);

    let mut cache = cache.lock().unwrap();
    for i4 in -12..=12 {
        for i5 in -12..=12 {
            let l = i + i4;
            let l1 = i1 + i5;
            let chunk_key = crate::world::level::chunk_pos::ChunkPos::as_long_static(l, l1);
            // it.unimi.dsi.fastutil.HashCommon.mix
            let mixed = (chunk_key as u64).wrapping_mul(0x9E3779B97F4A7C15);
            let cache_index = ((mixed ^ (mixed >> 32)) as u32 & 8191) as usize;
            let f1;
            if cache.keys[cache_index] == chunk_key {
                f1 = cache.values[cache_index];
            } else {
                f1 = if (l as i64) * (l as i64) + (l1 as i64) * (l1 as i64) > 4096 && noise.get_value_2d(l as f64, l1 as f64) < -0.9 {
                    (l.abs() as f32 * 3439.0 + l1.abs() as f32 * 147.0) % 13.0 + 9.0
                } else {
                    f32::MIN
                };
                cache.keys[cache_index] = chunk_key;
                cache.values[cache_index] = f1;
            }
            if f1 != f32::MIN {
                let f2 = i2 - i4 * 2;
                let f3 = i3 - i5 * 2;
                let mut f4 = 100.0f32 - mth::sqrt_f32((f2 * f2 + f3 * f3) as f32) * f1;
                f4 = mth::clamp_f32(f4, -100.0, 80.0);
                f = f.max(f4);
            }
        }
    }
    f
}

