//! Перенос `net.minecraft.world.level` — блоки, уровни, чанки.

pub mod block;
pub mod chunk_pos;

/// `net.minecraft.world.level.Level` — маркер типа для `ResourceKey<Level>`
/// и GlobalPos. Полная реализация Level — отдельный этап порта.
#[derive(Debug)]
pub struct Level;
