//! Перенос `net.minecraft.world.level.levelgen.VerticalAnchor`
//! — вертикальный якорь высоты (absolute/above_bottom/below_top).

use super::heightproviders::WorldGenerationContext;

/// `VerticalAnchor` — точка отсчёта по Y.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalAnchor {
    /// `Absolute(y)`.
    Absolute(i32),
    /// `AboveBottom(offset)` — `minY + offset`.
    AboveBottom(i32),
    /// `BelowTop(offset)` — `minY + height - 1 - offset`.
    BelowTop(i32),
}

impl VerticalAnchor {
    /// `absolute(value)`.
    pub fn absolute(value: i32) -> Self {
        Self::Absolute(value)
    }

    /// `aboveBottom(value)`.
    pub fn above_bottom(value: i32) -> Self {
        Self::AboveBottom(value)
    }

    /// `belowTop(value)`.
    pub fn below_top(value: i32) -> Self {
        Self::BelowTop(value)
    }

    /// `BOTTOM` = above_bottom(0).
    pub const BOTTOM: Self = Self::AboveBottom(0);
    /// `TOP` = below_top(0).
    pub const TOP: Self = Self::BelowTop(0);

    /// `bottom()`.
    pub fn bottom() -> Self {
        Self::BOTTOM
    }

    /// `top()`.
    pub fn top() -> Self {
        Self::TOP
    }

    /// `resolveY(context)`.
    pub fn resolve_y(&self, context: &WorldGenerationContext) -> i32 {
        match *self {
            Self::Absolute(y) => y,
            Self::AboveBottom(offset) => context.get_min_gen_y() + offset,
            Self::BelowTop(offset) => {
                context.get_gen_depth() - 1 + context.get_min_gen_y() - offset
            }
        }
    }
}
