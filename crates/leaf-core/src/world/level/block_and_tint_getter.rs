//! `net.minecraft.world.level.BlockAndTintGetter` —
//! `BlockGetter` + свет/тинт (для генерации и рендера).
//!
//! Отличие от Java: `getLightEngine()` возвращает
//! `Option<&LevelLightEngine>` (`None` — если движок недоступен;
//! `EmptyBlockAndTintGetter` переопределяет brightness-методы).

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::world::level::block_getter::BlockGetter;
use crate::world::level::color_resolver::ColorResolver;
use crate::world::level::light_layer::LightLayer;
use crate::world::level::lighting::level_light_engine::LevelLightEngine;

/// `BlockAndTintGetter extends BlockGetter`.
pub trait BlockAndTintGetter: BlockGetter {
    /// `getShade(direction, shade)`.
    fn get_shade(&self, direction: Direction, shade: bool) -> f32;

    /// `getLightEngine()` — `Option` (см. шапку).
    fn get_light_engine(&self) -> Option<&LevelLightEngine>;

    /// `getBlockTint(pos, colorResolver)`.
    fn get_block_tint(&self, pos: BlockPos, color_resolver: &dyn ColorResolver) -> i32;

    /// `getBrightness(lightLayer, pos)`.
    fn get_brightness(&self, light_layer: LightLayer, pos: BlockPos) -> i32 {
        if let Some(engine) = self.get_light_engine() {
            engine.get_light_value(&pos, light_layer) as i32
        } else if let Some(level) = self.as_level_reader() {
            level.get_brightness(light_layer, pos)
        } else {
            0
        }
    }

    /// `getRawBrightness(pos, amount)`.
    fn get_raw_brightness(&self, pos: BlockPos, amount_to_subtract: i32) -> i32 {
        if let Some(level) = self.as_level_reader() {
            return level.get_raw_brightness(pos, amount_to_subtract);
        }
        let block = self.get_brightness(LightLayer::Block, pos);
        let sky = (self.get_brightness(LightLayer::Sky, pos) - amount_to_subtract).max(0);
        block.max(sky)
    }

    /// `canSeeSky(pos)`.
    fn can_see_sky(&self, pos: BlockPos) -> bool {
        self.get_brightness(LightLayer::Sky, pos) >= 15
    }
}
