//! `ca.spottedleaf.moonrise.patches.starlight.chunk`.

use crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::{
    EmptinessMapRef, NibbleArrayRef,
};

/// `StarlightChunk` — доступ к nibble-массивам освещения и картам пустоты
/// чанка для StarLight.
pub trait StarlightChunk {
    /// `starlight$getBlockNibbles`.
    fn starlight_get_block_nibbles(&self) -> Vec<Option<NibbleArrayRef>>;
    /// `starlight$setBlockNibbles`.
    fn starlight_set_block_nibbles(&self, nibbles: Vec<Option<NibbleArrayRef>>);
    /// `starlight$getSkyNibbles`.
    fn starlight_get_sky_nibbles(&self) -> Vec<Option<NibbleArrayRef>>;
    /// `starlight$setSkyNibbles`.
    fn starlight_set_sky_nibbles(&self, nibbles: Vec<Option<NibbleArrayRef>>);
    /// `starlight$getSkyEmptinessMap`.
    fn starlight_get_sky_emptiness_map(&self) -> Option<EmptinessMapRef>;
    /// `starlight$setSkyEmptinessMap`.
    fn starlight_set_sky_emptiness_map(&self, map: Option<EmptinessMapRef>);
    /// `starlight$getBlockEmptinessMap`.
    fn starlight_get_block_emptiness_map(&self) -> Option<EmptinessMapRef>;
    /// `starlight$setBlockEmptinessMap`.
    fn starlight_set_block_emptiness_map(&self, map: Option<EmptinessMapRef>);
}
