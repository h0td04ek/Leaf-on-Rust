//! Перенос `ClientboundPlayerInfoUpdatePacket`.

use crate::authlib::game_profile::GameProfile;
use crate::authlib::property_map::PropertyMap;
use crate::network::chat::component::Component;
use crate::network::chat::component_json::{component_stream_codec, optional_component_stream_codec};
use crate::network::chat::remote_chat_session::RemoteChatSessionData;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::util::uuid::Uuid;
use crate::world::level::game_type::GameType;
use std::collections::HashSet;

/// `ClientboundPlayerInfoUpdatePacket.Action` — порядок = ordinal
/// (важен для EnumSet bitmask).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayerInfoUpdateAction {
    /// `ADD_PLAYER`.
    AddPlayer,
    /// `INITIALIZE_CHAT`.
    InitializeChat,
    /// `UPDATE_GAME_MODE`.
    UpdateGameMode,
    /// `UPDATE_LISTED`.
    UpdateListed,
    /// `UPDATE_LATENCY`.
    UpdateLatency,
    /// `UPDATE_DISPLAY_NAME`.
    UpdateDisplayName,
    /// `UPDATE_LIST_ORDER`.
    UpdateListOrder,
    /// `UPDATE_HAT`.
    UpdateHat,
}

/// Все значения в порядке объявления.
pub const PLAYER_INFO_UPDATE_ACTIONS: [PlayerInfoUpdateAction; 8] = [
    PlayerInfoUpdateAction::AddPlayer,
    PlayerInfoUpdateAction::InitializeChat,
    PlayerInfoUpdateAction::UpdateGameMode,
    PlayerInfoUpdateAction::UpdateListed,
    PlayerInfoUpdateAction::UpdateLatency,
    PlayerInfoUpdateAction::UpdateDisplayName,
    PlayerInfoUpdateAction::UpdateListOrder,
    PlayerInfoUpdateAction::UpdateHat,
];

impl PlayerInfoUpdateAction {
    /// ordinal.
    pub fn ordinal(&self) -> usize {
        PLAYER_INFO_UPDATE_ACTIONS.iter().position(|v| v == self).unwrap()
    }
}

/// `ClientboundPlayerInfoUpdatePacket.Entry`.
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerInfoEntry {
    /// `profileId`.
    pub profile_id: Uuid,
    /// `profile` — только при `ADD_PLAYER`.
    pub profile: Option<GameProfile>,
    /// `listed`.
    pub listed: bool,
    /// `latency`.
    pub latency: i32,
    /// `gameMode`.
    pub game_mode: GameType,
    /// `displayName`.
    pub display_name: Option<Component>,
    /// `showHat`.
    pub show_hat: bool,
    /// `listOrder`.
    pub list_order: i32,
    /// `chatSession`.
    pub chat_session: Option<RemoteChatSessionData>,
}

impl PlayerInfoEntry {
    /// `Entry(UUID, listed)` — Paper-конструктор для UPDATE_LISTED.
    pub fn new_minimal(profile_id: Uuid, listed: bool) -> Self {
        Self {
            profile_id,
            profile: None,
            listed,
            latency: 0,
            game_mode: GameType::by_id(0),
            display_name: None,
            show_hat: true,
            list_order: 0,
            chat_session: None,
        }
    }
}

/// `EntryBuilder` — накопление полей по actions.
#[derive(Clone, Debug)]
pub struct PlayerInfoEntryBuilder {
    /// `profileId`.
    pub profile_id: Uuid,
    /// `profile`.
    pub profile: Option<GameProfile>,
    /// `listed`.
    pub listed: bool,
    /// `latency`.
    pub latency: i32,
    /// `gameMode`.
    pub game_mode: GameType,
    /// `displayName`.
    pub display_name: Option<Component>,
    /// `showHat`.
    pub show_hat: bool,
    /// `listOrder`.
    pub list_order: i32,
    /// `chatSession`.
    pub chat_session: Option<RemoteChatSessionData>,
}

impl PlayerInfoEntryBuilder {
    /// `EntryBuilder::new`.
    pub fn new(profile_id: Uuid) -> Self {
        Self {
            profile_id,
            profile: None,
            listed: false,
            latency: 0,
            game_mode: GameType::by_id(0),
            display_name: None,
            show_hat: false,
            list_order: 0,
            chat_session: None,
        }
    }

    /// `build`.
    pub fn build(self) -> PlayerInfoEntry {
        PlayerInfoEntry {
            profile_id: self.profile_id,
            profile: self.profile,
            listed: self.listed,
            latency: self.latency,
            game_mode: self.game_mode,
            display_name: self.display_name,
            show_hat: self.show_hat,
            list_order: self.list_order,
            chat_session: self.chat_session,
        }
    }
}

/// `ClientboundPlayerInfoUpdatePacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundPlayerInfoUpdatePacket {
    /// `actions` — множество действий (EnumSet).
    pub actions: HashSet<PlayerInfoUpdateAction>,
    /// `entries`.
    pub entries: Vec<PlayerInfoEntry>,
}

impl ClientboundPlayerInfoUpdatePacket {
    /// Основной конструктор.
    pub fn new(actions: HashSet<PlayerInfoUpdateAction>, entries: Vec<PlayerInfoEntry>) -> Self {
        Self { actions, entries }
    }

    /// `createPlayerInitializing` (без Paper-варианта с listed).
    pub fn create_player_initializing(entries: Vec<PlayerInfoEntry>) -> Self {
        let actions: HashSet<PlayerInfoUpdateAction> = [
            PlayerInfoUpdateAction::AddPlayer,
            PlayerInfoUpdateAction::InitializeChat,
            PlayerInfoUpdateAction::UpdateGameMode,
            PlayerInfoUpdateAction::UpdateListed,
            PlayerInfoUpdateAction::UpdateLatency,
            PlayerInfoUpdateAction::UpdateDisplayName,
            PlayerInfoUpdateAction::UpdateHat,
            PlayerInfoUpdateAction::UpdateListOrder,
        ]
        .into_iter()
        .collect();
        Self::new(actions, entries)
    }

    /// `newEntries` — записи только если есть ADD_PLAYER.
    pub fn new_entries(&self) -> Vec<PlayerInfoEntry> {
        if self.actions.contains(&PlayerInfoUpdateAction::AddPlayer) {
            self.entries.clone()
        } else {
            Vec::new()
        }
    }

    /// Чтение одного entry по actions (порядок — ordinal).
    fn read_entry(buf: &mut FriendlyByteBuf, actions: &HashSet<PlayerInfoUpdateAction>) -> PlayerInfoEntry {
        let profile_id = buf.read_uuid();
        let mut builder = PlayerInfoEntryBuilder::new(profile_id);
        for action in PLAYER_INFO_UPDATE_ACTIONS {
            if !actions.contains(&action) {
                continue;
            }
            match action {
                PlayerInfoUpdateAction::AddPlayer => {
                    let name = byte_buf_codecs::PLAYER_NAME.decode(buf);
                    let properties = byte_buf_codecs::game_profile_properties().decode(buf);
                    builder.profile = Some(GameProfile::with_properties(profile_id, name, properties));
                }
                PlayerInfoUpdateAction::InitializeChat => {
                    builder.chat_session =
                        buf.read_optional(|b| RemoteChatSessionData::read(b).expect("invalid chat session"));
                }
                PlayerInfoUpdateAction::UpdateGameMode => {
                    builder.game_mode = GameType::by_id(buf.read_var_int());
                }
                PlayerInfoUpdateAction::UpdateListed => {
                    builder.listed = buf.read_bool();
                }
                PlayerInfoUpdateAction::UpdateLatency => {
                    builder.latency = buf.read_var_int();
                }
                PlayerInfoUpdateAction::UpdateDisplayName => {
                    builder.display_name = optional_component_stream_codec().decode(buf);
                }
                PlayerInfoUpdateAction::UpdateListOrder => {
                    builder.list_order = buf.read_var_int();
                }
                PlayerInfoUpdateAction::UpdateHat => {
                    builder.show_hat = buf.read_bool();
                }
            }
        }
        builder.build()
    }

    /// Запись одного entry по actions.
    fn write_entry(
        buf: &mut FriendlyByteBuf,
        entry: &PlayerInfoEntry,
        actions: &HashSet<PlayerInfoUpdateAction>,
    ) {
        buf.write_uuid(&entry.profile_id);
        for action in PLAYER_INFO_UPDATE_ACTIONS {
            if !actions.contains(&action) {
                continue;
            }
            match action {
                PlayerInfoUpdateAction::AddPlayer => {
                    let profile = entry.profile.as_ref().expect("profile for ADD_PLAYER");
                    byte_buf_codecs::PLAYER_NAME.encode(buf, &profile.name);
                    byte_buf_codecs::game_profile_properties().encode(buf, &profile.properties);
                }
                PlayerInfoUpdateAction::InitializeChat => {
                    // Paper: не отправлять истёкший ключ
                    let chat_session = entry
                        .chat_session
                        .as_ref()
                        .filter(|s| !s.profile_public_key.has_expired());
                    if let Some(s) = chat_session {
                        buf.write_bool(true);
                        s.write(buf).expect("chat session write");
                    } else {
                        buf.write_bool(false);
                    }
                }
                PlayerInfoUpdateAction::UpdateGameMode => {
                    buf.write_var_int(entry.game_mode.get_id());
                }
                PlayerInfoUpdateAction::UpdateListed => {
                    buf.write_bool(entry.listed);
                }
                PlayerInfoUpdateAction::UpdateLatency => {
                    buf.write_var_int(entry.latency);
                }
                PlayerInfoUpdateAction::UpdateDisplayName => {
                    optional_component_stream_codec().encode(buf, &entry.display_name);
                }
                PlayerInfoUpdateAction::UpdateListOrder => {
                    buf.write_var_int(entry.list_order);
                }
                PlayerInfoUpdateAction::UpdateHat => {
                    buf.write_bool(entry.show_hat);
                }
            }
        }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundPlayerInfoUpdatePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundPlayerInfoUpdatePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let action_set: HashSet<usize> = buf.read_enum_set(&PLAYER_INFO_UPDATE_ACTIONS);
                let actions: HashSet<PlayerInfoUpdateAction> = action_set
                    .into_iter()
                    .filter_map(|i| PLAYER_INFO_UPDATE_ACTIONS.get(i).copied())
                    .collect();
                let entries = buf.read_list(|b| Self::read_entry(b, &actions));
                ClientboundPlayerInfoUpdatePacket::new(actions, entries)
            },
        )
    }
}

impl Packet for ClientboundPlayerInfoUpdatePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_player_info_update::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        let action_ordinals: HashSet<usize> = self.actions.iter().map(|a| a.ordinal()).collect();
        buf.write_enum_set(&PLAYER_INFO_UPDATE_ACTIONS, &action_ordinals)
            .expect("enum set");
        buf.write_collection(&self.entries, |b, e| {
            Self::write_entry(b, e, &self.actions);
        });
    }
}
