//! `net.minecraft.world.level.biome.BiomeSpecialEffects`.

use crate::util::string_representable::StringRepresentable;

/// `BiomeSpecialEffects` — визуальные эффекты биома.
#[derive(Clone, Debug, PartialEq)]
pub struct BiomeSpecialEffects {
    /// `waterColor`.
    pub water_color: i32,
    /// `foliageColorOverride`.
    pub foliage_color_override: Option<i32>,
    /// `dryFoliageColorOverride`.
    pub dry_foliage_color_override: Option<i32>,
    /// `grassColorOverride`.
    pub grass_color_override: Option<i32>,
    /// `grassColorModifier`.
    pub grass_color_modifier: GrassColorModifier,
}

/// `BiomeSpecialEffects.Builder`.
#[derive(Default)]
pub struct BiomeSpecialEffectsBuilder {
    water_color: Option<i32>,
    foliage_color_override: Option<i32>,
    dry_foliage_color_override: Option<i32>,
    grass_color_override: Option<i32>,
    grass_color_modifier: GrassColorModifier,
}

impl BiomeSpecialEffectsBuilder {
    /// `waterColor`.
    pub fn water_color(mut self, water_color: i32) -> Self {
        self.water_color = Some(water_color);
        self
    }
    /// `foliageColorOverride`.
    pub fn foliage_color_override(mut self, c: i32) -> Self {
        self.foliage_color_override = Some(c);
        self
    }
    /// `dryFoliageColorOverride`.
    pub fn dry_foliage_color_override(mut self, c: i32) -> Self {
        self.dry_foliage_color_override = Some(c);
        self
    }
    /// `grassColorOverride`.
    pub fn grass_color_override(mut self, c: i32) -> Self {
        self.grass_color_override = Some(c);
        self
    }
    /// `grassColorModifier`.
    pub fn grass_color_modifier(mut self, m: GrassColorModifier) -> Self {
        self.grass_color_modifier = m;
        self
    }
    /// `build`.
    pub fn build(self) -> BiomeSpecialEffects {
        BiomeSpecialEffects {
            water_color: self.water_color.expect("Missing 'water' color."),
            foliage_color_override: self.foliage_color_override,
            dry_foliage_color_override: self.dry_foliage_color_override,
            grass_color_override: self.grass_color_override,
            grass_color_modifier: self.grass_color_modifier,
        }
    }
}

impl Default for GrassColorModifier {
    fn default() -> Self {
        GrassColorModifier::None
    }
}

/// `BiomeSpecialEffects.GrassColorModifier`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GrassColorModifier {
    /// `NONE`.
    None,
    /// `DARK_FOREST`.
    DarkForest,
    /// `SWAMP`.
    Swamp,
}

impl GrassColorModifier {
    /// `modifyColor(x, z, grassColor)`.
    pub fn modify_color(&self, x: f64, z: f64, grass_color: i32) -> i32 {
        match self {
            GrassColorModifier::None => grass_color,
            GrassColorModifier::DarkForest => ((grass_color & 0xFEFEFE) + 0x28_28_28) >> 1,
            GrassColorModifier::Swamp => {
                let value = crate::world::level::biome::biome::biome_info_noise().get_value(x * 0.0225, z * 0.0225, false);
                if value < -0.1 { 0x4C_65_34 } else { 0x6A_7F_5D }
            }
        }
    }
}

impl StringRepresentable for GrassColorModifier {
    fn get_serialized_name(&self) -> String {
        match self {
            GrassColorModifier::None => "none".into(),
            GrassColorModifier::DarkForest => "dark_forest".into(),
            GrassColorModifier::Swamp => "swamp".into(),
        }
    }
}
