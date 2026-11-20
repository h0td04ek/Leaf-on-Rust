//! `ca.spottedleaf.moonrise.patches.starlight.chunk`.

use crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::SWMRNibbleArray;

/// `StarlightChunk` — доступ к nibble-массивам освещения и картам пустоты
/// чанка для StarLight.
pub trait StarlightChunk {
    /// `starlight$getBlockNibbles`.
    fn starlight_get_block_nibbles(&self) -> &[Option<SWMRNibbleArray>];
    /// `starlight$setBlockNibbles`.
    fn starlight_set_block_nibbles(&mut self, nibbles: Vec<Option<SWMRNibbleArray>>);
    /// `starlight$getSkyNibbles`.
    fn starlight_get_sky_nibbles(&self) -> &[Option<SWMRNibbleArray>];
    /// `starlight$setSkyNibbles`.
    fn starlight_set_sky_nibbles(&mut self, nibbles: Vec<Option<SWMRNibbleArray>>);
    /// `starlight$getSkyEmptinessMap`.
    fn starlight_get_sky_emptiness_map(&self) -> Option<&[bool]>;
    /// `starlight$setSkyEmptinessMap`.
    fn starlight_set_sky_emptiness_map(&mut self, map: Option<Vec<bool>>);
    /// `starlight$getBlockEmptinessMap`.
    fn starlight_get_block_emptiness_map(&self) -> Option<&[bool]>;
    /// `starlight$setBlockEmptinessMap`.
    fn starlight_set_block_emptiness_map(&mut self, map: Option<Vec<bool>>);
}
