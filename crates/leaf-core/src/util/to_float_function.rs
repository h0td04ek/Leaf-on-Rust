//! Перенос `net.minecraft.util.ToFloatFunction<T>`.

/// `ToFloatFunction<T>` — функция T → float.
pub trait ToFloatFunction<T> {
    fn apply_as_float(&self, value: &T) -> f32;
}

impl<T, F: Fn(&T) -> f32> ToFloatFunction<T> for F {
    fn apply_as_float(&self, value: &T) -> f32 {
        self(value)
    }
}
