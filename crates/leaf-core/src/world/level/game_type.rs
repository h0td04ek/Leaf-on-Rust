//! Перенос `net.minecraft.world.level.GameType`.

/// `GameType` — enum режимов игры.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameType {
    /// `SURVIVAL`.
    Survival = 0,
    /// `CREATIVE`.
    Creative = 1,
    /// `ADVENTURE`.
    Adventure = 2,
    /// `SPECTATOR`.
    Spectator = 3,
}

/// Все значения (`values()`).
pub const GAME_TYPES: [GameType; 4] = [
    GameType::Survival,
    GameType::Creative,
    GameType::Adventure,
    GameType::Spectator,
];

/// `GameType.DEFAULT_MODE`.
pub const DEFAULT_MODE: GameType = GameType::Survival;

impl GameType {
    /// `values()`.
    pub fn values() -> &'static [GameType] {
        &GAME_TYPES
    }

    /// `getId`.
    pub fn get_id(&self) -> i32 {
        *self as i32
    }

    /// `byId` — `ByIdMap.continuous(..., ZERO)` — вне диапазона → SURVIVAL(0).
    pub fn by_id(id: i32) -> GameType {
        if (0..=3).contains(&id) {
            GAME_TYPES[id as usize]
        } else {
            DEFAULT_MODE
        }
    }

    /// `byNullableId` — `-1` → `None`, иначе `byId`.
    pub fn by_nullable_id(id: i8) -> Option<GameType> {
        if id == -1 {
            None
        } else {
            Some(Self::by_id(id as i32))
        }
    }

    /// `getNullableId` — `None` → `-1`.
    pub fn get_nullable_id(game_type: Option<GameType>) -> i32 {
        game_type.map_or(-1, |g| g.get_id())
    }

    /// `byName` — по сериализованному имени.
    pub fn by_name(name: &str) -> GameType {
        Self::by_name_or(name, DEFAULT_MODE)
    }

    /// `byName(name, fallback)`.
    pub fn by_name_or(name: &str, fallback: GameType) -> GameType {
        GAME_TYPES.iter().copied().find(|g| g.get_name() == name).unwrap_or(fallback)
    }

    /// `getName`/`getSerializedName`.
    pub fn get_name(&self) -> &'static str {
        match self {
            GameType::Survival => "survival",
            GameType::Creative => "creative",
            GameType::Adventure => "adventure",
            GameType::Spectator => "spectator",
        }
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        self.get_name()
    }

    /// `isCreative`.
    pub fn is_creative(&self) -> bool {
        *self == GameType::Creative
    }

    /// `isSurvival`.
    pub fn is_survival(&self) -> bool {
        matches!(self, GameType::Survival | GameType::Adventure)
    }

    /// `isBlockPlacingRestricted`.
    pub fn is_block_placing_restricted(&self) -> bool {
        matches!(self, GameType::Adventure | GameType::Spectator)
    }

    /// `getShortDisplayName`/`getLongDisplayName` — translatable components.
    pub fn get_short_display_name(&self) -> crate::network::chat::component::Component {
        crate::network::chat::component::Component::translatable(format!(
            "selectWorld.gameMode.{}",
            self.get_name()
        ), Vec::new())
    }

    /// `getLongDisplayName`.
    pub fn get_long_display_name(&self) -> crate::network::chat::component::Component {
        crate::network::chat::component::Component::translatable(format!(
            "gameMode.{}",
            self.get_name()
        ), Vec::new())
    }

    /// `updatePlayerAbilities`.
    pub fn update_player_abilities(self, abilities: &mut crate::entity::player::abilities::Abilities) {
        let spectator = self == Self::Spectator;
        abilities.instabuild = self.is_creative();
        abilities.mayfly = abilities.instabuild || spectator;
        abilities.invulnerable = abilities.mayfly;
        if !abilities.instabuild {
            abilities.flying = spectator;
        }
        abilities.may_build = !self.is_block_placing_restricted();
    }

    /// `STREAM_CODEC` — idMapper (VarInt id).
    pub fn stream_codec() -> impl crate::network::codec::stream_codec::StreamCodec<
        crate::network::friendly_byte_buf::FriendlyByteBuf,
        GameType,
    > {
        crate::network::codec::stream_codec::FnCodec::new(
            |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf, g: &GameType| {
                buf.write_var_int(g.get_id());
            },
            |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf| {
                GameType::by_id(buf.read_var_int())
            },
        )
    }
}
