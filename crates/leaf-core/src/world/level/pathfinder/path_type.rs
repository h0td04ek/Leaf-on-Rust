//! Перенос `net.minecraft.world.level.pathfinder.PathType`.

/// `PathType` — тип пути с ценой (malus).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum PathType {
    /// `BLOCKED`.
    Blocked,
    /// `OPEN`.
    Open,
    /// `WALKABLE`.
    Walkable,
    /// `WALKABLE_DOOR`.
    WalkableDoor,
    /// `TRAPDOOR`.
    Trapdoor,
    /// `POWDER_SNOW`.
    PowderSnow,
    /// `DANGER_POWDER_SNOW`.
    DangerPowderSnow,
    /// `FENCE`.
    Fence,
    /// `LAVA`.
    Lava,
    /// `WATER`.
    Water,
    /// `WATER_BORDER`.
    WaterBorder,
    /// `RAIL`.
    Rail,
    /// `UNPASSABLE_RAIL`.
    UnpassableRail,
    /// `DANGER_FIRE`.
    DangerFire,
    /// `DAMAGE_FIRE`.
    DamageFire,
    /// `DANGER_OTHER`.
    DangerOther,
    /// `DAMAGE_OTHER`.
    DamageOther,
    /// `DOOR_OPEN`.
    DoorOpen,
    /// `DOOR_WOOD_CLOSED`.
    DoorWoodClosed,
    /// `DOOR_IRON_CLOSED`.
    DoorIronClosed,
    /// `BREACH`.
    Breach,
    /// `LEAVES`.
    Leaves,
    /// `STICKY_HONEY`.
    StickyHoney,
    /// `COCOA`.
    Cocoa,
    /// `DAMAGE_CAUTIOUS`.
    DamageCautious,
    /// `DANGER_TRAPDOOR`.
    DangerTrapdoor,
}

impl PathType {
    /// `getMalus` — цена прохода.
    pub fn get_malus(self) -> f32 {
        match self {
            PathType::Blocked
            | PathType::PowderSnow
            | PathType::Fence
            | PathType::Lava
            | PathType::UnpassableRail
            | PathType::DoorWoodClosed
            | PathType::DoorIronClosed
            | PathType::Leaves
            | PathType::DamageOther => -1.0,
            PathType::Water | PathType::WaterBorder | PathType::DangerFire | PathType::DangerOther
            | PathType::StickyHoney => 8.0,
            PathType::DamageFire => 16.0,
            PathType::Breach => 4.0,
            _ => 0.0,
        }
    }
}
