//! Перенос `net.minecraft.world.entity.player.Abilities`.

use crate::nbt::CompoundTag;

/// `Abilities` — флаги способностей игрока.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Abilities {
    /// `invulnerable`.
    pub invulnerable: bool,
    /// `flying`.
    pub flying: bool,
    /// `mayfly`.
    pub mayfly: bool,
    /// `instabuild`.
    pub instabuild: bool,
    /// `mayBuild`.
    pub may_build: bool,
    /// `flyingSpeed`.
    pub flying_speed: f32,
    /// `walkingSpeed`.
    pub walking_speed: f32,
}

/// Дефолты Java.
pub const DEFAULT_INVULNERABLE: bool = false;
pub const DEFAULT_FLYING: bool = false;
pub const DEFAULT_MAY_FLY: bool = false;
pub const DEFAULT_INSTABUILD: bool = false;
pub const DEFAULT_MAY_BUILD: bool = true;
pub const DEFAULT_FLYING_SPEED: f32 = 0.05;
pub const DEFAULT_WALKING_SPEED: f32 = 0.1;

impl Default for Abilities {
    fn default() -> Self {
        Self {
            invulnerable: DEFAULT_INVULNERABLE,
            flying: DEFAULT_FLYING,
            mayfly: DEFAULT_MAY_FLY,
            instabuild: DEFAULT_INSTABUILD,
            may_build: DEFAULT_MAY_BUILD,
            flying_speed: DEFAULT_FLYING_SPEED,
            walking_speed: DEFAULT_WALKING_SPEED,
        }
    }
}

impl Abilities {
    /// `getFlyingSpeed`.
    pub fn get_flying_speed(&self) -> f32 {
        self.flying_speed
    }

    /// `setFlyingSpeed`.
    pub fn set_flying_speed(&mut self, flying_speed: f32) {
        self.flying_speed = flying_speed;
    }

    /// `getWalkingSpeed`.
    pub fn get_walking_speed(&self) -> f32 {
        self.walking_speed
    }

    /// `setWalkingSpeed`.
    pub fn set_walking_speed(&mut self, walking_speed: f32) {
        self.walking_speed = walking_speed;
    }

    /// `pack()` → `Abilities.Packed`.
    pub fn pack(&self) -> Packed {
        Packed {
            invulnerable: self.invulnerable,
            flying: self.flying,
            may_fly: self.mayfly,
            instabuild: self.instabuild,
            may_build: self.may_build,
            flying_speed: self.flying_speed,
            walking_speed: self.walking_speed,
        }
    }

    /// `apply(packed)`.
    pub fn apply(&mut self, packed: &Packed) {
        self.invulnerable = packed.invulnerable;
        self.flying = packed.flying;
        self.mayfly = packed.may_fly;
        self.instabuild = packed.instabuild;
        self.may_build = packed.may_build;
        self.flying_speed = packed.flying_speed;
        self.walking_speed = packed.walking_speed;
    }
}

/// `Abilities.Packed` — сериализуемая форма способностей.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Packed {
    /// `invulnerable`.
    pub invulnerable: bool,
    /// `flying`.
    pub flying: bool,
    /// `mayFly`.
    pub may_fly: bool,
    /// `instabuild`.
    pub instabuild: bool,
    /// `mayBuild`.
    pub may_build: bool,
    /// `flyingSpeed`.
    pub flying_speed: f32,
    /// `walkingSpeed`.
    pub walking_speed: f32,
}

impl Packed {
    pub fn from_nbt(tag: &CompoundTag) -> Self {
        Self {
            invulnerable: tag.get_bool_or("invulnerable", DEFAULT_INVULNERABLE),
            flying: tag.get_bool_or("flying", DEFAULT_FLYING),
            may_fly: tag.get_bool_or("mayfly", DEFAULT_MAY_FLY),
            instabuild: tag.get_bool_or("instabuild", DEFAULT_INSTABUILD),
            may_build: tag.get_bool_or("mayBuild", DEFAULT_MAY_BUILD),
            flying_speed: tag.get_float("flySpeed").unwrap_or(DEFAULT_FLYING_SPEED),
            walking_speed: tag.get_float("walkSpeed").unwrap_or(DEFAULT_WALKING_SPEED),
        }
    }

    pub fn to_nbt(self) -> CompoundTag {
        let mut tag = CompoundTag::new();
        tag.put_bool("invulnerable", self.invulnerable);
        tag.put_bool("flying", self.flying);
        tag.put_bool("mayfly", self.may_fly);
        tag.put_bool("instabuild", self.instabuild);
        tag.put_bool("mayBuild", self.may_build);
        tag.put_float("flySpeed", self.flying_speed);
        tag.put_float("walkSpeed", self.walking_speed);
        tag
    }
}
