//! Перенос `net.minecraft.world.scores.Team` + `PlayerTeam` —
//! команды игроков (видимость, коллизии, префикс/суффикс).

use crate::network::chat::chat_formatting::ChatFormatting;
use crate::network::chat::component::Component;
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `Team.Visibility`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Visibility {
    /// `ALWAYS`.
    Always,
    /// `NEVER`.
    Never,
    /// `HIDE_FOR_OTHER_TEAMS`.
    HideForOtherTeams,
    /// `HIDE_FOR_OWN_TEAM`.
    HideForOwnTeam,
}

/// Все значения (`values()`).
pub const VISIBILITIES: [Visibility; 4] = [
    Visibility::Always,
    Visibility::Never,
    Visibility::HideForOtherTeams,
    Visibility::HideForOwnTeam,
];

impl Visibility {
    /// `values()`.
    pub fn values() -> &'static [Visibility] {
        &VISIBILITIES
    }

    /// `id`.
    pub fn id(&self) -> i32 {
        *self as i32
    }

    /// `getName`.
    pub fn get_name(&self) -> &'static str {
        match self {
            Visibility::Always => "always",
            Visibility::Never => "never",
            Visibility::HideForOtherTeams => "hideForOtherTeams",
            Visibility::HideForOwnTeam => "hideForOwnTeam",
        }
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        self.get_name()
    }

    /// `getDisplayName`.
    pub fn get_display_name(&self) -> Component {
        Component::translatable(&format!("team.visibility.{}", self.get_name()), Vec::new())
    }

    /// `BY_ID` — `ByIdMap.continuous` + `OutOfBoundsStrategy.ZERO`.
    pub fn by_id(id: i32) -> Visibility {
        if (0..VISIBILITIES.len() as i32).contains(&id) {
            VISIBILITIES[id as usize]
        } else {
            VISIBILITIES[0]
        }
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.idMapper(BY_ID, id)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Visibility> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, v: &Visibility| {
                buf.write_var_int(v.id());
            },
            |buf: &mut FriendlyByteBuf| Visibility::by_id(buf.read_var_int()),
        )
    }
}

/// `Team.CollisionRule`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CollisionRule {
    /// `ALWAYS`.
    Always,
    /// `NEVER`.
    Never,
    /// `PUSH_OTHER_TEAMS`.
    PushOtherTeams,
    /// `PUSH_OWN_TEAM`.
    PushOwnTeam,
}

/// Все значения (`values()`).
pub const COLLISION_RULES: [CollisionRule; 4] = [
    CollisionRule::Always,
    CollisionRule::Never,
    CollisionRule::PushOtherTeams,
    CollisionRule::PushOwnTeam,
];

impl CollisionRule {
    /// `values()`.
    pub fn values() -> &'static [CollisionRule] {
        &COLLISION_RULES
    }

    /// `id`.
    pub fn id(&self) -> i32 {
        *self as i32
    }

    /// `getName`.
    pub fn get_name(&self) -> &'static str {
        match self {
            CollisionRule::Always => "always",
            CollisionRule::Never => "never",
            CollisionRule::PushOtherTeams => "pushOtherTeams",
            CollisionRule::PushOwnTeam => "pushOwnTeam",
        }
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        self.get_name()
    }

    /// `getDisplayName`.
    pub fn get_display_name(&self) -> Component {
        Component::translatable(&format!("team.collision.{}", self.get_name()), Vec::new())
    }

    /// `BY_ID`.
    pub fn by_id(id: i32) -> CollisionRule {
        if (0..COLLISION_RULES.len() as i32).contains(&id) {
            COLLISION_RULES[id as usize]
        } else {
            COLLISION_RULES[0]
        }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, CollisionRule> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, v: &CollisionRule| {
                buf.write_var_int(v.id());
            },
            |buf: &mut FriendlyByteBuf| CollisionRule::by_id(buf.read_var_int()),
        )
    }
}

/// Бит-флаги `options` (`PlayerTeam.packOptions`).
pub const TEAM_OPTION_ALLOW_FRIENDLY_FIRE: i32 = 1;
/// `canSeeFriendlyInvisibles`.
pub const TEAM_OPTION_SEE_FRIENDLY_INVISIBLES: i32 = 2;

/// `PlayerTeam` — данные команды (без `Scoreboard`-колбэков,
/// которые относятся к серверной логике).
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerTeam {
    /// `name`.
    pub name: String,
    /// `displayName`.
    pub display_name: Component,
    /// `options` — битовое поле (`packOptions`).
    pub options: i32,
    /// `nametagVisibility`.
    pub nametag_visibility: Visibility,
    /// `deathMessageVisibility`.
    pub death_message_visibility: Visibility,
    /// `collisionRule`.
    pub collision_rule: CollisionRule,
    /// `color`.
    pub color: ChatFormatting,
    /// `playerPrefix`.
    pub player_prefix: Component,
    /// `playerSuffix`.
    pub player_suffix: Component,
    /// `players` — имена участников.
    pub players: Vec<String>,
}

impl PlayerTeam {
    /// Конструктор с дефолтами как у `PlayerTeam` в Java.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: Component::empty(),
            options: 0,
            nametag_visibility: Visibility::Always,
            death_message_visibility: Visibility::Always,
            collision_rule: CollisionRule::Always,
            color: ChatFormatting::Reset,
            player_prefix: Component::empty(),
            player_suffix: Component::empty(),
            players: Vec::new(),
        }
    }

    /// `getName`.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// `getDisplayName`.
    pub fn get_display_name(&self) -> &Component {
        &self.display_name
    }

    /// `setDisplayName`.
    pub fn set_display_name(&mut self, display_name: Component) {
        self.display_name = display_name;
    }

    /// `isAllowFriendlyFire`.
    pub fn is_allow_friendly_fire(&self) -> bool {
        self.options & TEAM_OPTION_ALLOW_FRIENDLY_FIRE != 0
    }

    /// `setAllowFriendlyFire`.
    pub fn set_allow_friendly_fire(&mut self, allow: bool) {
        self.options = if allow {
            self.options | TEAM_OPTION_ALLOW_FRIENDLY_FIRE
        } else {
            self.options & !TEAM_OPTION_ALLOW_FRIENDLY_FIRE
        };
    }

    /// `canSeeFriendlyInvisibles`.
    pub fn can_see_friendly_invisibles(&self) -> bool {
        self.options & TEAM_OPTION_SEE_FRIENDLY_INVISIBLES != 0
    }

    /// `setSeeFriendlyInvisibles`.
    pub fn set_see_friendly_invisibles(&mut self, see: bool) {
        self.options = if see {
            self.options | TEAM_OPTION_SEE_FRIENDLY_INVISIBLES
        } else {
            self.options & !TEAM_OPTION_SEE_FRIENDLY_INVISIBLES
        };
    }

    /// `packOptions`.
    pub fn pack_options(&self) -> i32 {
        let mut i = 0;
        if self.is_allow_friendly_fire() {
            i |= TEAM_OPTION_ALLOW_FRIENDLY_FIRE;
        }
        if self.can_see_friendly_invisibles() {
            i |= TEAM_OPTION_SEE_FRIENDLY_INVISIBLES;
        }
        i
    }

    /// `unpackOptions`.
    pub fn unpack_options(&mut self, flags: i32) {
        self.set_allow_friendly_fire(flags & TEAM_OPTION_ALLOW_FRIENDLY_FIRE != 0);
        self.set_see_friendly_invisibles(flags & TEAM_OPTION_SEE_FRIENDLY_INVISIBLES != 0);
    }

    /// `getPlayers`.
    pub fn get_players(&self) -> &[String] {
        &self.players
    }

    /// `getPlayerPrefix`/`getPlayerSuffix` (Paper API).
    pub fn get_player_prefix(&self) -> &Component {
        &self.player_prefix
    }
    /// `getPlayerSuffix`.
    pub fn get_player_suffix(&self) -> &Component {
        &self.player_suffix
    }
}
