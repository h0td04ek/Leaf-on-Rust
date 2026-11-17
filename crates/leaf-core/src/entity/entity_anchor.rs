//! Перенос `net.minecraft.commands.arguments.EntityAnchorArgument.Anchor`.

/// `Anchor` — точка привязки сущности.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    /// `feet`.
    Feet = 0,
    /// `eyes`.
    Eyes = 1,
}

impl Anchor {
    /// `values()`-порядок.
    pub const VALUES: [Anchor; 2] = [Anchor::Feet, Anchor::Eyes];

    /// `byId` — ordinal (`readEnum`).
    pub fn by_id(id: i32) -> Self {
        Self::VALUES[id.rem_euclid(2) as usize]
    }

    /// `ordinal` (`writeEnum`).
    pub fn ordinal(&self) -> i32 {
        *self as i32
    }

    /// `getByName`.
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "feet" => Some(Anchor::Feet),
            "eyes" => Some(Anchor::Eyes),
            _ => None,
        }
    }

    /// `name`.
    pub fn name(&self) -> &'static str {
        match self {
            Anchor::Feet => "feet",
            Anchor::Eyes => "eyes",
        }
    }
}
