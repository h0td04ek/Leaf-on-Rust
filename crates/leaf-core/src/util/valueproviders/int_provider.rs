//! Перенос `net.minecraft.util.valueproviders.IntProvider`.
//! DFU-кодеки (`CODEC`, `NON_NEGATIVE_CODEC`, `IntProviderType`) —
//! появятся вместе с портом DataFixerUpper; здесь чистая логика.

use crate::random::RandomSource;

/// `IntProvider` — провайдер случайных int (трейтовый объект).
pub trait IntProvider {
    fn sample(&self, random: &mut dyn RandomSource) -> i32;
    fn min_value(&self) -> i32;
    fn max_value(&self) -> i32;

    /// `toString`-представление для отладки.
    fn describe(&self) -> String;
}
