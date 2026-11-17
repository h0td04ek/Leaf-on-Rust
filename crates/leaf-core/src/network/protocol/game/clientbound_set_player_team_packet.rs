//! Перенос `ClientboundSetPlayerTeamPacket`.

use crate::network::chat::chat_formatting::ChatFormatting;
use crate::network::chat::component::Component;
use crate::network::chat::component_json::component_stream_codec;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::scores::team::{CollisionRule, PlayerTeam, Visibility};
use std::collections::HashSet;

const METHOD_ADD: i32 = 0;
const METHOD_REMOVE: i32 = 1;
const METHOD_CHANGE: i32 = 2;
const METHOD_JOIN: i32 = 3;
const METHOD_LEAVE: i32 = 4;

fn should_have_parameters(method: i32) -> bool {
    method == METHOD_ADD || method == METHOD_CHANGE
}

fn should_have_player_list(method: i32) -> bool {
    method == METHOD_ADD || method == METHOD_JOIN || method == METHOD_LEAVE
}

fn chat_formatting_ordinal(f: &ChatFormatting) -> i32 {
    ChatFormatting::VALUES.iter().position(|v| v == f).unwrap() as i32
}

fn chat_formatting_by_ordinal(id: i32) -> ChatFormatting {
    ChatFormatting::VALUES[id.rem_euclid(ChatFormatting::VALUES.len() as i32) as usize]
}

/// `ClientboundSetPlayerTeamPacket.Action`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeamPacketAction {
    /// `ADD`.
    Add,
    /// `REMOVE`.
    Remove,
}

/// `ClientboundSetPlayerTeamPacket.Parameters`.
#[derive(Clone, Debug, PartialEq)]
pub struct TeamParameters {
    /// `displayName`.
    pub display_name: Component,
    /// `options`.
    pub options: i32,
    /// `nametagVisibility`.
    pub nametag_visibility: Visibility,
    /// `collisionRule`.
    pub collision_rule: CollisionRule,
    /// `color`.
    pub color: ChatFormatting,
    /// `playerPrefix`.
    pub player_prefix: Component,
    /// `playerSuffix`.
    pub player_suffix: Component,
}

impl TeamParameters {
    /// `Parameters(PlayerTeam)`.
    pub fn from_team(team: &PlayerTeam) -> Self {
        Self {
            display_name: team.get_display_name().copy(),
            options: team.pack_options(),
            nametag_visibility: team.nametag_visibility,
            collision_rule: team.collision_rule,
            color: team.color,
            player_prefix: team.get_player_prefix().copy(),
            player_suffix: team.get_player_suffix().copy(),
        }
    }

    /// `write`.
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        component_stream_codec().encode(buf, &self.display_name);
        buf.write_byte(self.options);
        Visibility::stream_codec().encode(buf, &self.nametag_visibility);
        CollisionRule::stream_codec().encode(buf, &self.collision_rule);
        buf.write_var_int(chat_formatting_ordinal(&self.color));
        component_stream_codec().encode(buf, &self.player_prefix);
        component_stream_codec().encode(buf, &self.player_suffix);
    }

    /// read из буфера (private в Java — здесь pub(crate)-эквивалент).
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let display_name = component_stream_codec().decode(buf);
        let options = buf.read_byte() as i32;
        let nametag_visibility = Visibility::stream_codec().decode(buf);
        let collision_rule = CollisionRule::stream_codec().decode(buf);
        let color = chat_formatting_by_ordinal(buf.read_var_int());
        let player_prefix = component_stream_codec().decode(buf);
        let player_suffix = component_stream_codec().decode(buf);
        Self {
            display_name,
            options,
            nametag_visibility,
            collision_rule,
            color,
            player_prefix,
            player_suffix,
        }
    }
}

/// `ClientboundSetPlayerTeamPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSetPlayerTeamPacket {
    /// `name`.
    pub name: String,
    /// `method`.
    pub method: i32,
    /// `parameters`.
    pub parameters: Option<TeamParameters>,
    /// `players`.
    pub players: Vec<String>,
}

impl ClientboundSetPlayerTeamPacket {
    /// Основной конструктор (private в Java — args как у него).
    pub fn new(
        name: String,
        method: i32,
        parameters: Option<TeamParameters>,
        players: Vec<String>,
    ) -> Self {
        Self { name, method, parameters, players }
    }

    /// `createAddOrModifyPacket`.
    pub fn create_add_or_modify_packet(team: &PlayerTeam, use_add: bool) -> Self {
        Self::new(
            team.get_name().to_string(),
            if use_add { METHOD_ADD } else { METHOD_CHANGE },
            Some(TeamParameters::from_team(team)),
            if use_add {
                team.get_players().to_vec()
            } else {
                Vec::new()
            },
        )
    }

    /// `createRemovePacket`.
    pub fn create_remove_packet(team: &PlayerTeam) -> Self {
        Self::new(team.get_name().to_string(), METHOD_REMOVE, None, Vec::new())
    }

    /// `createPlayerPacket`.
    pub fn create_player_packet(
        team: &PlayerTeam,
        player_name: String,
        action: TeamPacketAction,
    ) -> Self {
        Self::new(
            team.get_name().to_string(),
            if action == TeamPacketAction::Add { METHOD_JOIN } else { METHOD_LEAVE },
            None,
            vec![player_name],
        )
    }

    /// `createMultiplePlayerPacket` (Paper).
    pub fn create_multiple_player_packet(
        team: &PlayerTeam,
        players: Vec<String>,
        action: TeamPacketAction,
    ) -> Self {
        Self::new(
            team.get_name().to_string(),
            if action == TeamPacketAction::Add { METHOD_JOIN } else { METHOD_LEAVE },
            None,
            players,
        )
    }

    /// `getPlayerAction`.
    pub fn get_player_action(&self) -> Option<TeamPacketAction> {
        match self.method {
            METHOD_ADD | METHOD_JOIN => Some(TeamPacketAction::Add),
            METHOD_LEAVE => Some(TeamPacketAction::Remove),
            _ => None,
        }
    }

    /// `getTeamAction`.
    pub fn get_team_action(&self) -> Option<TeamPacketAction> {
        match self.method {
            METHOD_ADD => Some(TeamPacketAction::Add),
            METHOD_REMOVE => Some(TeamPacketAction::Remove),
            _ => None,
        }
    }

    /// `getName`.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// `getPlayers`.
    pub fn get_players(&self) -> &[String] {
        &self.players
    }

    /// `getParameters`.
    pub fn get_parameters(&self) -> Option<&TeamParameters> {
        self.parameters.as_ref()
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetPlayerTeamPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetPlayerTeamPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let name = buf.read_utf().expect("invalid utf");
                let method = buf.read_byte() as i32;
                let parameters = if should_have_parameters(method) {
                    Some(TeamParameters::read(buf))
                } else {
                    None
                };
                let players = if should_have_player_list(method) {
                    buf.read_list(|b| b.read_utf().expect("invalid utf"))
                } else {
                    Vec::new()
                };
                ClientboundSetPlayerTeamPacket::new(name, method, parameters, players)
            },
        )
    }
}

impl Packet for ClientboundSetPlayerTeamPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_player_team::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf(&self.name).expect("invalid utf");
        buf.write_byte(self.method);
        if should_have_parameters(self.method) {
            self.parameters
                .as_ref()
                .expect("Parameters not present")
                .write(buf);
        }
        if should_have_player_list(self.method) {
            buf.write_collection(&self.players, |b, s| {
                b.write_utf(s).expect("invalid utf");
            });
        }
    }
}
