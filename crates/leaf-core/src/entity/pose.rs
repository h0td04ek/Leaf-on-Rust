//! `Pose` — поза entity.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `Pose`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pose {
    /// `STANDING`.
    Standing,
    /// `FALL_FLYING`.
    FallFlying,
    /// `SLEEPING`.
    Sleeping,
    /// `SWIMMING`.
    Swimming,
    /// `SPIN_ATTACK`.
    SpinAttack,
    /// `CROUCHING`.
    Crouching,
    /// `LONG_JUMPING`.
    LongJumping,
    /// `DYING`.
    Dying,
    /// `CROAKING`.
    Croaking,
    /// `USING_TONGUE`.
    UsingTongue,
    /// `SITTING`.
    Sitting,
    /// `ROARING`.
    Roaring,
    /// `SNIFFING`.
    Sniffing,
    /// `EMERGING`.
    Emerging,
    /// `DIGGING`.
    Digging,
    /// `SLIDING`.
    Sliding,
    /// `SHOOTING`.
    Shooting,
    /// `INHALING`.
    Inhaling,
}

impl Pose {
    /// `id()`.
    pub fn id(&self) -> i32 {
        match self {
            Self::Standing => 0,
            Self::FallFlying => 1,
            Self::Sleeping => 2,
            Self::Swimming => 3,
            Self::SpinAttack => 4,
            Self::Crouching => 5,
            Self::LongJumping => 6,
            Self::Dying => 7,
            Self::Croaking => 8,
            Self::UsingTongue => 9,
            Self::Sitting => 10,
            Self::Roaring => 11,
            Self::Sniffing => 12,
            Self::Emerging => 13,
            Self::Digging => 14,
            Self::Sliding => 15,
            Self::Shooting => 16,
            Self::Inhaling => 17,
        }
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            Self::Standing => "standing",
            Self::FallFlying => "fall_flying",
            Self::Sleeping => "sleeping",
            Self::Swimming => "swimming",
            Self::SpinAttack => "spin_attack",
            Self::Crouching => "crouching",
            Self::LongJumping => "long_jumping",
            Self::Dying => "dying",
            Self::Croaking => "croaking",
            Self::UsingTongue => "using_tongue",
            Self::Sitting => "sitting",
            Self::Roaring => "roaring",
            Self::Sniffing => "sniffing",
            Self::Emerging => "emerging",
            Self::Digging => "digging",
            Self::Sliding => "sliding",
            Self::Shooting => "shooting",
            Self::Inhaling => "inhaling",
        }
    }

    /// `byId`.
    pub fn by_id(id: i32) -> Self {
        match id {
            0 => Self::Standing,
            1 => Self::FallFlying,
            2 => Self::Sleeping,
            3 => Self::Swimming,
            4 => Self::SpinAttack,
            5 => Self::Crouching,
            6 => Self::LongJumping,
            7 => Self::Dying,
            8 => Self::Croaking,
            9 => Self::UsingTongue,
            10 => Self::Sitting,
            11 => Self::Roaring,
            12 => Self::Sniffing,
            13 => Self::Emerging,
            14 => Self::Digging,
            15 => Self::Sliding,
            16 => Self::Shooting,
            17 => Self::Inhaling,
            _ => Self::Standing, // OutOfBoundsStrategy.ZERO
        }
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.idMapper(BY_ID, Pose::id)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Pose> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, p: &Pose| { buf.write_var_int(p.id()); },
            |buf: &mut FriendlyByteBuf| Pose::by_id(buf.read_var_int()),
        )
    }
}
