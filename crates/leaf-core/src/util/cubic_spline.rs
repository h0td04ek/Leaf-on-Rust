//! Перенос `net.minecraft.util.CubicSpline<C, I>`.
//!
//! DFU-кодек (`codec(valueCodec)`) отложен; логика сплайнов портирована.
//! Тип `I extends BoundedFloatFunction<C>` стёрт до `BoundedFloatFn<C>`
//! (Arc<dyn BoundedFloatFunction<C>>) — все координаты в Java всё равно
//! одного типа.

use crate::util::bounded_float_function::{BoundedFloatFn, BoundedFloatFunction};
use crate::util::mth;

/// `CubicSpline<C, I>` — кубический сплайн: apply(C)->float + границы.
pub trait CubicSpline<C>: Send + Sync {
    fn apply(&self, value: C) -> f32;
    fn min_value(&self) -> f32;
    fn max_value(&self) -> f32;
    /// `parityString` — отладочное представление.
    fn parity_string(&self) -> String;
    /// `mapAll(visitor)` — применить visitor к координате и рекурсивно к значениям.
    fn map_all(&self, visitor: &mut dyn FnMut(&BoundedFloatFn<C>) -> BoundedFloatFn<C>) -> Box<dyn CubicSpline<C>>;
}

/// `CubicSpline.constant(value)` — константный сплайн.
pub fn constant<C>(value: f32) -> Box<dyn CubicSpline<C>> {
    Box::new(ConstantSpline { value })
}

/// `CubicSpline.builder(coordinate)` — Builder с IDENTITY valueTransformer.
pub fn builder<C: Clone + 'static>(coordinate: BoundedFloatFn<C>) -> CubicSplineBuilder<C> {
    CubicSplineBuilder::new(coordinate)
}

/// `CubicSpline.Constant<C, I>(value)`.
#[derive(Clone)]
pub struct ConstantSpline {
    value: f32,
}

impl<C> CubicSpline<C> for ConstantSpline {
    fn apply(&self, _value: C) -> f32 {
        self.value
    }
    fn min_value(&self) -> f32 {
        self.value
    }
    fn max_value(&self) -> f32 {
        self.value
    }
    fn parity_string(&self) -> String {
        format!("k={:.3}", self.value)
    }
    fn map_all(&self, _visitor: &mut dyn FnMut(&BoundedFloatFn<C>) -> BoundedFloatFn<C>) -> Box<dyn CubicSpline<C>> {
        Box::new(self.clone())
    }
}

/// `CubicSpline.Builder<C, I>`.
pub struct CubicSplineBuilder<C> {
    coordinate: BoundedFloatFn<C>,
    locations: Vec<f32>,
    values: Vec<Box<dyn CubicSpline<C>>>,
    derivatives: Vec<f32>,
}

impl<C: Clone + 'static> CubicSplineBuilder<C> {
    pub fn new(coordinate: BoundedFloatFn<C>) -> Self {
        Self { coordinate, locations: Vec::new(), values: Vec::new(), derivatives: Vec::new() }
    }

    /// `addPoint(location, float value)` — Constant-сплайн.
    pub fn add_point_float(&mut self, location: f32, value: f32) -> &mut Self {
        self.add_point_internal(location, constant(value), 0.0)
    }

    /// `addPoint(location, float value, float derivative)`.
    pub fn add_point_float_derivative(&mut self, location: f32, value: f32, derivative: f32) -> &mut Self {
        self.add_point_internal(location, constant(value), derivative)
    }

    /// `addPoint(location, CubicSpline value)`.
    pub fn add_point(&mut self, location: f32, value: Box<dyn CubicSpline<C>>) -> &mut Self {
        self.add_point_internal(location, value, 0.0)
    }

    /// `addPoint(location, CubicSpline value, float derivative)`.
    pub fn add_point_with_derivative(
        &mut self,
        location: f32,
        value: Box<dyn CubicSpline<C>>,
        derivative: f32,
    ) -> &mut Self {
        self.add_point_internal(location, value, derivative)
    }

    fn add_point_internal(&mut self, location: f32, value: Box<dyn CubicSpline<C>>, derivative: f32) -> &mut Self {
        if let Some(&last) = self.locations.last() {
            assert!(location > last, "Please register points in ascending order");
        }
        self.locations.push(location);
        self.values.push(value);
        self.derivatives.push(derivative);
        self
    }

    /// `build()` — пустой → panic.
    pub fn build(self) -> Box<dyn CubicSpline<C>> {
        assert!(!self.locations.is_empty(), "No elements added");
        Multipoint::create(self.coordinate, &self.locations, self.values, &self.derivatives)
    }
}

/// `CubicSpline.Multipoint<C, I>`.
pub struct Multipoint<C> {
    coordinate: BoundedFloatFn<C>,
    locations: Vec<f32>,
    values: Vec<Box<dyn CubicSpline<C>>>,
    derivatives: Vec<f32>,
    min_value: f32,
    max_value: f32,
}

fn validate_sizes(locations_len: usize, values_len: usize, derivatives_len: usize) {
    if locations_len != values_len || locations_len != derivatives_len {
        panic!(
            "All lengths must be equal, got: {} {} {}",
            locations_len, values_len, derivatives_len
        );
    }
    if locations_len == 0 {
        panic!("Cannot create a multipoint spline with no points");
    }
}

fn linear_extend(
    coordinate: f32,
    locations: &[f32],
    value: f32,
    derivatives: &[f32],
    index: usize,
) -> f32 {
    let f = derivatives[index];
    if f == 0.0 { value } else { value + f * (coordinate - locations[index]) }
}

impl<C: Clone + 'static> Multipoint<C> {
    /// `Multipoint.create(coordinate, locations, values, derivatives)`.
    pub fn create(
        coordinate: BoundedFloatFn<C>,
        locations: &[f32],
        values: Vec<Box<dyn CubicSpline<C>>>,
        derivatives: &[f32],
    ) -> Box<dyn CubicSpline<C>> {
        validate_sizes(locations.len(), values.len(), derivatives.len());
        let i = locations.len() - 1;
        let mut f = f32::INFINITY;
        let mut f1 = f32::NEG_INFINITY;
        let f2 = coordinate.min_value();
        let f3 = coordinate.max_value();
        if f2 < locations[0] {
            let f4 = linear_extend(f2, locations, values[0].min_value(), derivatives, 0);
            let f5 = linear_extend(f2, locations, values[0].max_value(), derivatives, 0);
            f = f.min(f4.min(f5));
            f1 = f1.max(f4.max(f5));
        }
        if f3 > locations[i] {
            let f4 = linear_extend(f3, locations, values[i].min_value(), derivatives, i);
            let f5 = linear_extend(f3, locations, values[i].max_value(), derivatives, i);
            f = f.min(f4.min(f5));
            f1 = f1.max(f4.max(f5));
        }
        for cubic_spline in &values {
            f = f.min(cubic_spline.min_value());
            f1 = f1.max(cubic_spline.max_value());
        }
        for i1 in 0..i {
            let f5 = locations[i1];
            let f6 = locations[i1 + 1];
            let f7 = f6 - f5;
            let cubic_spline1 = &values[i1];
            let cubic_spline2 = &values[i1 + 1];
            let f8 = cubic_spline1.min_value();
            let f9 = cubic_spline1.max_value();
            let f10 = cubic_spline2.min_value();
            let f11 = cubic_spline2.max_value();
            let f12 = derivatives[i1];
            let f13 = derivatives[i1 + 1];
            if f12 != 0.0 || f13 != 0.0 {
                let f14 = f12 * f7;
                let f15 = f13 * f7;
                let min = f8.min(f10);
                let max = f9.max(f11);
                let f16 = f14 - f11 + f8;
                let f17 = f14 - f10 + f9;
                let f18 = -f15 + f10 - f9;
                let f19 = -f15 + f11 - f8;
                let min1 = f16.min(f18);
                let max1 = f17.max(f19);
                f = f.min(min + 0.25 * min1);
                f1 = f1.max(max + 0.25 * max1);
            }
        }
        Box::new(Self {
            coordinate,
            locations: locations.to_vec(),
            values,
            derivatives: derivatives.to_vec(),
            min_value: f,
            max_value: f1,
        })
    }
}

impl<C: Clone + 'static> CubicSpline<C> for Multipoint<C> {
    fn apply(&self, value: C) -> f32 {
        let f = self.coordinate.apply(value.clone());
        let i = find_interval_start(&self.locations, f);
        let i1 = self.locations.len() - 1;
        if i < 0 {
            return linear_extend(f, &self.locations, self.values[0].apply(value), &self.derivatives, 0);
        } else if i as usize == i1 {
            return linear_extend(f, &self.locations, self.values[i1].apply(value), &self.derivatives, i1);
        } else {
            let i = i as usize;
            let f1 = self.locations[i];
            let f2 = self.locations[i + 1];
            let f3 = (f - f1) / (f2 - f1);
            let f4 = self.derivatives[i];
            let f5 = self.derivatives[i + 1];
            let f6 = self.values[i].apply(value.clone());
            let f7 = self.values[i + 1].apply(value);
            let f8 = f4 * (f2 - f1) - (f7 - f6);
            let f9 = -f5 * (f2 - f1) + (f7 - f6);
            mth::lerp_f32(f3, f6, f7) + f3 * (1.0 - f3) * mth::lerp_f32(f3, f8, f9)
        }
    }

    fn min_value(&self) -> f32 {
        self.min_value
    }
    fn max_value(&self) -> f32 {
        self.max_value
    }

    fn parity_string(&self) -> String {
        format!(
            "Spline{{coordinate={}, locations={}, derivatives={}, values=[{}]}}",
            "<coord>",
            fmt_f32_slice(&self.locations),
            fmt_f32_slice(&self.derivatives),
            self.values.iter().map(|v| v.parity_string()).collect::<Vec<_>>().join(", ")
        )
    }

    fn map_all(&self, visitor: &mut dyn FnMut(&BoundedFloatFn<C>) -> BoundedFloatFn<C>) -> Box<dyn CubicSpline<C>> {
        Multipoint::create(
            visitor(&self.coordinate),
            &self.locations,
            self.values.iter().map(|s| s.map_all(visitor)).collect(),
            &self.derivatives,
        )
    }
}

fn find_interval_start(locations: &[f32], start: f32) -> i32 {
    mth::binary_search(0, locations.len() as i32, &mut |i| start < locations[i as usize]) - 1
}

fn fmt_f32_slice(v: &[f32]) -> String {
    let inner: Vec<String> = v.iter().map(|x| format!("{:.3}", x)).collect();
    format!("[{}]", inner.join(", "))
}
