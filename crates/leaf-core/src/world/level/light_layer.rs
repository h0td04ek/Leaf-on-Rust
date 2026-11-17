//! `net.minecraft.world.level.LightLayer`.

/// `LightLayer` — слой освещения.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LightLayer {
    /// `SKY`.
    Sky,
    /// `BLOCK`.
    Block,
}
