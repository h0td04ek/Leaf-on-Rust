//! Перенос `net.minecraft.world.entity.Pose` — поза сущности.

/// `Pose` — поза сущности.
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
    /// `values()`.
    pub const VALUES: [Pose; 18] = [
        Pose::Standing,
        Pose::FallFlying,
        Pose::Sleeping,
        Pose::Swimming,
        Pose::SpinAttack,
        Pose::Crouching,
        Pose::LongJumping,
        Pose::Dying,
        Pose::Croaking,
        Pose::UsingTongue,
        Pose::Sitting,
        Pose::Roaring,
        Pose::Sniffing,
        Pose::Emerging,
        Pose::Digging,
        Pose::Sliding,
        Pose::Shooting,
        Pose::Inhaling,
    ];

    /// `getName()`.
    pub fn get_name(&self) -> &str {
        match self {
            Pose::Standing => "standing",
            Pose::FallFlying => "fall_flying",
            Pose::Sleeping => "sleeping",
            Pose::Swimming => "swimming",
            Pose::SpinAttack => "spin_attack",
            Pose::Crouching => "crouching",
            Pose::LongJumping => "long_jumping",
            Pose::Dying => "dying",
            Pose::Croaking => "croaking",
            Pose::UsingTongue => "using_tongue",
            Pose::Sitting => "sitting",
            Pose::Roaring => "roaring",
            Pose::Sniffing => "sniffing",
            Pose::Emerging => "emerging",
            Pose::Digging => "digging",
            Pose::Sliding => "sliding",
            Pose::Shooting => "shooting",
            Pose::Inhaling => "inhaling",
        }
    }

    /// `getHeightScale()`.
    pub fn get_height_scale(&self) -> f32 {
        match self {
            Pose::Crouching | Pose::Swimming | Pose::SpinAttack | Pose::UsingTongue | Pose::Sitting | Pose::Roaring | Pose::Sniffing | Pose::Emerging | Pose::Digging | Pose::Sliding | Pose::Shooting | Pose::Inhaling => 0.5,
            _ => 1.0,
        }
    }

    /// `getWidthScale()`.
    pub fn get_width_scale(&self) -> f32 {
        match self {
            Pose::Crouching | Pose::Swimming | Pose::SpinAttack | Pose::UsingTongue | Pose::Sitting | Pose::Roaring | Pose::Sniffing | Pose::Emerging | Pose::Digging | Pose::Sliding | Pose::Shooting | Pose::Inhaling => 0.5,
            _ => 1.0,
        }
    }
}
