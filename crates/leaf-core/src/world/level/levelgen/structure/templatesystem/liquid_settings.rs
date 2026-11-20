//! Перенос `net.minecraft.world.level.levelgen.structure.templatesystem.LiquidSettings`.

/// `LiquidSettings` — поведение водяного логирования при
/// размещении шаблона.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiquidSettings {
    /// `IGNORE_WATERLOGGING` — не применять водяное логирование.
    IgnoreWaterlogging,
    /// `APPLY_WATERLOGGING` — применять.
    ApplyWaterlogging,
}

impl LiquidSettings {
    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            LiquidSettings::IgnoreWaterlogging => "ignore_waterlogging",
            LiquidSettings::ApplyWaterlogging => "apply_waterlogging",
        }
    }
}
