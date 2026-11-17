//! Перенос `net.minecraft.util.BoundedFloatFunction<C>`.

use std::sync::Arc;

/// `BoundedFloatFunction<C>` — функция C → float с границами min/max.
pub trait BoundedFloatFunction<C> {
    /// `asAny` — для downcast к конкретным реализациям (Spline.Coordinate).
    fn as_any(&self) -> &dyn std::any::Any;
    fn apply(&self, value: C) -> f32;
    fn min_value(&self) -> f32;
    fn max_value(&self) -> f32;
}

/// Стирание типа: `Arc<dyn BoundedFloatFunction<C>>`.
pub type BoundedFloatFn<C> = Arc<dyn BoundedFloatFunction<C> + Send + Sync>;

/// `BoundedFloatFunction.IDENTITY` для Float.
#[derive(Clone, Copy)]
pub struct FloatIdentity;

impl BoundedFloatFunction<f32> for FloatIdentity {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn apply(&self, value: f32) -> f32 {
        value
    }
    fn min_value(&self) -> f32 {
        f32::NEG_INFINITY
    }
    fn max_value(&self) -> f32 {
        f32::INFINITY
    }
}

/// `BoundedFloatFunction.createUnlimited(Float2FloatFunction)`.
pub fn create_unlimited_f32<F>(operation: F) -> impl BoundedFloatFunction<f32> + Clone + Send + Sync + 'static
where
    F: Fn(f32) -> f32 + Clone + Send + Sync + 'static,
{
    UnlimitedFloatFunction { operation }
}

#[derive(Clone)]
struct UnlimitedFloatFunction<F> {
    operation: F,
}

impl<F: Fn(f32) -> f32 + 'static> BoundedFloatFunction<f32> for UnlimitedFloatFunction<F> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn apply(&self, value: f32) -> f32 {
        (self.operation)(value)
    }
    fn min_value(&self) -> f32 {
        f32::NEG_INFINITY
    }
    fn max_value(&self) -> f32 {
        f32::INFINITY
    }
}

/// Ограниченная float-функция из замыкания + границ.
pub fn bounded_closure<F>(apply: F, min: f32, max: f32) -> impl BoundedFloatFunction<f32> + Clone + Send + Sync + 'static
where
    F: Fn(f32) -> f32 + Clone + Send + Sync + 'static,
{
    BoundedClosure { apply, min, max }
}

#[derive(Clone)]
struct BoundedClosure<F> {
    apply: F,
    min: f32,
    max: f32,
}

impl<F: Fn(f32) -> f32 + 'static> BoundedFloatFunction<f32> for BoundedClosure<F> {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn apply(&self, value: f32) -> f32 {
        (self.apply)(value)
    }
    fn min_value(&self) -> f32 {
        self.min
    }
    fn max_value(&self) -> f32 {
        self.max
    }
}

/// `comap` — композиция: `inner.apply(mapper(value))`.
/// Возвращает стёртый BoundedFloatFn<C2>.
pub fn comap<C, C2, F>(
    inner: BoundedFloatFn<C>,
    mapper: F,
) -> BoundedFloatFn<C2>
where
    C: Clone + Send + Sync + 'static,
    C2: Send + Sync + 'static,
    F: Fn(&C2) -> C + Send + Sync + 'static,
{
    Arc::new(Comap { inner, mapper, _phantom: std::marker::PhantomData })
}

struct Comap<C, C2, F> {
    inner: BoundedFloatFn<C>,
    mapper: F,
    _phantom: std::marker::PhantomData<fn() -> C2>,
}

impl<C: Clone + Send + Sync + 'static, C2: Send + Sync + 'static, F: Fn(&C2) -> C + Send + Sync + 'static>
    BoundedFloatFunction<C2> for Comap<C, C2, F>
{
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn apply(&self, value: C2) -> f32 {
        self.inner.apply((self.mapper)(&value))
    }
    fn min_value(&self) -> f32 {
        self.inner.min_value()
    }
    fn max_value(&self) -> f32 {
        self.inner.max_value()
    }
}
