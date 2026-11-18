//! Перенос `net.minecraft.world.level.pathfinder.PathComputationType`.

/// `PathComputationType` — режим вычисления проходимости.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PathComputationType {
    /// `LAND`.
    Land,
    /// `WATER`.
    Water,
    /// `AIR`.
    Air,
}
