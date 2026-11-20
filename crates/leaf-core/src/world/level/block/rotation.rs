//! Перенос `net.minecraft.world.level.block.Rotation`.
//! `OctahedralGroup` (полный набор 24 поворотов октакуба) — TODO:
//! нужен для blockstate-трансформаций, здесь не используется.

use crate::core::direction::{Axis, Direction};
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::random::RandomSource;
use crate::util;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rotation {
    NONE,
    CLOCKWISE_90,
    CLOCKWISE_180,
    COUNTERCLOCKWISE_90,
}

impl Rotation {
    pub const VALUES: [Rotation; 4] = [
        Rotation::NONE,
        Rotation::CLOCKWISE_90,
        Rotation::CLOCKWISE_180,
        Rotation::COUNTERCLOCKWISE_90,
    ];

    /// Порядок объявления в Java: NONE(0), CW90(1), 180(2), CCW90(3).
    /// Индекс для `getIndex` — позиция в VALUES, совпадает.
    pub fn get_index(&self) -> usize {
        *self as usize
    }

    pub fn get_id(&self) -> &'static str {
        match self {
            Rotation::NONE => "none",
            Rotation::CLOCKWISE_90 => "clockwise_90",
            Rotation::CLOCKWISE_180 => "180",
            Rotation::COUNTERCLOCKWISE_90 => "counterclockwise_90",
        }
    }

    /// `StringRepresentable.getSerializedName` / `CODEC.byName`.
    pub fn by_name(name: &str) -> Option<Rotation> {
        Self::VALUES.iter().copied().find(|r| r.get_id() == name)
    }

    pub fn get_serialized_name(&self) -> &'static str {
        self.get_id()
    }

    /// `getRotated(Rotation)` — композиция поворотов.
    pub fn get_rotated(&self, rotation: Rotation) -> Rotation {
        match rotation {
            Rotation::CLOCKWISE_90 => match self {
                Rotation::NONE => Rotation::CLOCKWISE_90,
                Rotation::CLOCKWISE_90 => Rotation::CLOCKWISE_180,
                Rotation::CLOCKWISE_180 => Rotation::COUNTERCLOCKWISE_90,
                Rotation::COUNTERCLOCKWISE_90 => Rotation::NONE,
            },
            Rotation::CLOCKWISE_180 => match self {
                Rotation::NONE => Rotation::CLOCKWISE_180,
                Rotation::CLOCKWISE_90 => Rotation::COUNTERCLOCKWISE_90,
                Rotation::CLOCKWISE_180 => Rotation::NONE,
                Rotation::COUNTERCLOCKWISE_90 => Rotation::CLOCKWISE_90,
            },
            Rotation::COUNTERCLOCKWISE_90 => match self {
                Rotation::NONE => Rotation::COUNTERCLOCKWISE_90,
                Rotation::CLOCKWISE_90 => Rotation::NONE,
                Rotation::CLOCKWISE_180 => Rotation::CLOCKWISE_90,
                Rotation::COUNTERCLOCKWISE_90 => Rotation::CLOCKWISE_180,
            },
            _ => *self,
        }
    }

    /// `rotate(Direction)`
    pub fn rotate(&self, facing: Direction) -> Direction {
        if facing.get_axis() == Axis::Y {
            facing
        } else {
            match self {
                Rotation::CLOCKWISE_90 => facing.get_clock_wise(),
                Rotation::CLOCKWISE_180 => facing.get_opposite(),
                Rotation::COUNTERCLOCKWISE_90 => facing.get_counter_clock_wise(),
                _ => facing,
            }
        }
    }

    /// `rotate(float angle)` — поворот угла на 90°*index.
    pub fn rotate_f32(&self, angle: f32) -> f32 {
        let shift = match self {
            Rotation::CLOCKWISE_90 => 90.0,
            Rotation::CLOCKWISE_180 => 180.0,
            Rotation::COUNTERCLOCKWISE_90 => -90.0,
            Rotation::NONE => 0.0,
        };
        (angle + shift).rem_euclid(360.0)
    }

    /// `rotate(int rotation, int positionCount)`
    pub fn rotate_index(&self, rotation: i32, position_count: i32) -> i32 {
        match self {
            Rotation::CLOCKWISE_90 => (rotation + position_count / 4).rem_euclid(position_count),
            Rotation::CLOCKWISE_180 => (rotation + position_count / 2).rem_euclid(position_count),
            Rotation::COUNTERCLOCKWISE_90 => {
                (rotation + position_count * 3 / 4).rem_euclid(position_count)
            }
            _ => rotation,
        }
    }

    /// `getShuffled`
    pub fn get_shuffled(random: &mut dyn RandomSource) -> Vec<Rotation> {
        util::shuffled_copy(&Self::VALUES, random)
    }

    /// `byIndex` — `ByIdMap.continuous(..., WRAP)`.
    pub fn by_index(index: i32) -> Rotation {
        Self::VALUES[index.rem_euclid(4) as usize]
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.idMapper(BY_ID, getIndex)` (VarInt index).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Rotation> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, v: &Rotation| {
                buf.write_var_int(v.get_index() as i32);
            },
            |buf: &mut FriendlyByteBuf| Rotation::by_index(buf.read_var_int()),
        )
    }
}
