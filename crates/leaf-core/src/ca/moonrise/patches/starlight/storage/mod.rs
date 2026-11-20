//! `ca.spottedleaf.moonrise.patches.starlight.storage`.

/// `StarlightSectionData` — состояние освещения секции (block/sky light).
pub trait StarlightSectionData {
    /// `starlight$getBlockLightState`.
    fn starlight_get_block_light_state(&self) -> i32;
    /// `starlight$setBlockLightState`.
    fn starlight_set_block_light_state(&mut self, state: i32);
    /// `starlight$getSkyLightState`.
    fn starlight_get_sky_light_state(&self) -> i32;
    /// `starlight$setSkyLightState`.
    fn starlight_set_sky_light_state(&mut self, state: i32);
}
