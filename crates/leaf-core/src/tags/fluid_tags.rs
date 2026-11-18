//! Перенос `net.minecraft.tags.FluidTags` — константы тегов жидкостей.

use crate::resources::identifier::Identifier;

/// `FluidTags`.
pub struct FluidTags;

impl FluidTags {
    /// `FluidTags.WATER` — `minecraft:water`.
    pub fn water() -> Identifier {
        Identifier::minecraft("water")
    }
    /// `FluidTags.LAVA` — `minecraft:lava`.
    pub fn lava() -> Identifier {
        Identifier::minecraft("lava")
    }
}
