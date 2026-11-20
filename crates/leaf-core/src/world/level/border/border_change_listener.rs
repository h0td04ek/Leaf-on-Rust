//! `net.minecraft.world.level.border.BorderChangeListener` —
//! колбэки изменений `WorldBorder`.

use super::world_border::WorldBorder;

/// Слушатель событий границы мира.
pub trait BorderChangeListener: Send + Sync {
    /// `onSetCenter`.
    fn on_set_center(&self, border: &WorldBorder, x: f64, z: f64);
    /// `onSetSize`.
    fn on_set_size(&self, border: &WorldBorder, size: f64);
    /// `onLerpSize`.
    fn on_lerp_size(&self, border: &WorldBorder, old_size: f64, new_size: f64, time: i64, start_time: i64);
    /// `onSetAbsoluteMaxSize`.
    fn on_set_absolute_max_size(&self, border: &WorldBorder, size: i32);
    /// `onSetWarningBlocks`.
    fn on_set_warning_blocks(&self, border: &WorldBorder, warning_blocks: i32);
    /// `onSetWarningTime`.
    fn on_set_warning_time(&self, border: &WorldBorder, warning_time: i32);
    /// `onSetDamagePerBlock`.
    fn on_set_damage_per_block(&self, border: &WorldBorder, damage_per_block: f64);
    /// `onSetSafeZone`.
    fn on_set_safe_zone(&self, border: &WorldBorder, safe_zone: f64);
}
