//! Перенос `net.minecraft.world.entity.Relative` — битовые флаги
//! относительных координат телепорта.

use std::collections::BTreeSet;

/// `Relative` — флаг относительности поля.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Relative {
    /// `X` (bit 0).
    X = 0,
    /// `Y` (bit 1).
    Y = 1,
    /// `Z` (bit 2).
    Z = 2,
    /// `Y_ROT` (bit 3).
    YRot = 3,
    /// `X_ROT` (bit 4).
    XRot = 4,
    /// `DELTA_X` (bit 5).
    DeltaX = 5,
    /// `DELTA_Y` (bit 6).
    DeltaY = 6,
    /// `DELTA_Z` (bit 7).
    DeltaZ = 7,
    /// `ROTATE_DELTA` (bit 8).
    RotateDelta = 8,
}

/// `Set<Relative>` — в Rust `BTreeSet` (детерминированный порядок).
pub type RelativeSet = BTreeSet<Relative>;

impl Relative {
    /// Все значения (порядок `values()`).
    pub const VALUES: [Relative; 9] = [
        Relative::X,
        Relative::Y,
        Relative::Z,
        Relative::YRot,
        Relative::XRot,
        Relative::DeltaX,
        Relative::DeltaY,
        Relative::DeltaZ,
        Relative::RotateDelta,
    ];

    /// `ALL`.
    pub fn all() -> RelativeSet {
        Relative::VALUES.iter().copied().collect()
    }

    /// `ROTATION` — {X_ROT, Y_ROT}.
    pub fn rotation_set() -> RelativeSet {
        [Relative::XRot, Relative::YRot].into_iter().collect()
    }

    /// `DELTA` — {DELTA_X, DELTA_Y, DELTA_Z, ROTATE_DELTA}.
    pub fn delta_set() -> RelativeSet {
        [
            Relative::DeltaX,
            Relative::DeltaY,
            Relative::DeltaZ,
            Relative::RotateDelta,
        ]
        .into_iter()
        .collect()
    }

    /// `union(sets...)`.
    pub fn union(sets: &[&RelativeSet]) -> RelativeSet {
        let mut out = RelativeSet::new();
        for s in sets {
            out.extend(s.iter().copied());
        }
        out
    }

    /// `rotation(yRelative, xRelative)`.
    pub fn rotation(y_relative: bool, x_relative: bool) -> RelativeSet {
        let mut set = RelativeSet::new();
        if y_relative {
            set.insert(Relative::YRot);
        }
        if x_relative {
            set.insert(Relative::XRot);
        }
        set
    }

    /// `position(xRelative, yRelative, zRelative)`.
    pub fn position(x_relative: bool, y_relative: bool, z_relative: bool) -> RelativeSet {
        let mut set = RelativeSet::new();
        if x_relative {
            set.insert(Relative::X);
        }
        if y_relative {
            set.insert(Relative::Y);
        }
        if z_relative {
            set.insert(Relative::Z);
        }
        set
    }

    /// `direction(xRelative, yRelative, zRelative)` — delta-направление.
    pub fn direction(x_relative: bool, y_relative: bool, z_relative: bool) -> RelativeSet {
        let mut set = RelativeSet::new();
        if x_relative {
            set.insert(Relative::DeltaX);
        }
        if y_relative {
            set.insert(Relative::DeltaY);
        }
        if z_relative {
            set.insert(Relative::DeltaZ);
        }
        set
    }

    fn mask(&self) -> i32 {
        1 << (*self as i32)
    }

    fn is_set(&self, data: i32) -> bool {
        data & self.mask() == self.mask()
    }

    /// `unpack(data)` — int bitmask → set.
    pub fn unpack(data: i32) -> RelativeSet {
        Relative::VALUES
            .iter()
            .filter(|r| r.is_set(data))
            .copied()
            .collect()
    }

    /// `pack(set)` — set → int bitmask.
    pub fn pack(relatives: &RelativeSet) -> i32 {
        relatives.iter().fold(0, |acc, r| acc | r.mask())
    }

    /// `SET_STREAM_CODEC` — чтение/запись набора как INT bitmask.
    pub fn read_set(buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf) -> RelativeSet {
        Relative::unpack(buf.read_int())
    }

    pub fn write_set(buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf, set: &RelativeSet) {
        buf.write_int(Relative::pack(set));
    }
}
