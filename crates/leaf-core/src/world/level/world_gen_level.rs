//! `net.minecraft.world.level.WorldGenLevel` —
//! `ServerLevelAccessor` + seed для генерации.

use super::server_level_accessor::ServerLevelAccessor;

/// `WorldGenLevel extends ServerLevelAccessor`.
pub trait WorldGenLevel: ServerLevelAccessor {
    /// `getSeed()`.
    fn get_seed(&self) -> i64;

    /// `ensureCanWrite(pos)` — default `true`
    /// (переопределяется в `WorldGenRegion`).
    fn ensure_can_write(&self, _pos: crate::core::block_pos::BlockPos) -> bool {
        true
    }

    /// `setCurrentlyGenerating(supplier)` — debug-метка
    /// генерации (default no-op).
    fn set_currently_generating(&mut self, _currently_generating: Option<String>) {}
}
