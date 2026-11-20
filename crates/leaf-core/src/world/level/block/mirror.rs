//! Перенос `net.minecraft.world.level.block.Mirror`.
//! (`OctahedralGroup`-трансформации — с портом blockstate.)

/// `Mirror` — 3 значения в порядке объявления.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mirror {
    /// `NONE`.
    NONE,
    /// `LEFT_RIGHT`.
    LEFT_RIGHT,
    /// `FRONT_BACK`.
    FRONT_BACK,
}

impl Mirror {
    /// `values()`.
    pub const VALUES: [Mirror; 3] = [Mirror::NONE, Mirror::LEFT_RIGHT, Mirror::FRONT_BACK];

    /// `mirror(float angle)` — отражение угла (для entity
    /// yRot при mirror-повороте шаблона).
    pub fn mirror_f32(&self, angle: f32) -> f32 {
        match self {
            Mirror::NONE => angle,
            Mirror::LEFT_RIGHT => 180.0 - angle,
            Mirror::FRONT_BACK => -angle,
        }
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            Mirror::NONE => "none",
            Mirror::LEFT_RIGHT => "left_right",
            Mirror::FRONT_BACK => "front_back",
        }
    }

    /// `CODEC.byName`.
    pub fn by_name(name: &str) -> Option<Mirror> {
        Self::VALUES.iter().copied().find(|m| m.get_serialized_name() == name)
    }
}
