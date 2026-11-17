//! Перенос `ServerboundSeenAdvancementsPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::resources::identifier::Identifier;

/// `ServerboundSeenAdvancementsPacket.Action`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeenAdvancementsAction {
    /// `OPENED_TAB`.
    OpenedTab = 0,
    /// `CLOSED_SCREEN`.
    ClosedScreen = 1,
}

impl SeenAdvancementsAction {
    /// `values()`-порядок.
    pub const VALUES: [SeenAdvancementsAction; 2] = [
        SeenAdvancementsAction::OpenedTab,
        SeenAdvancementsAction::ClosedScreen,
    ];

    /// `byId` — ordinal.
    pub fn by_id(id: i32) -> Self {
        Self::VALUES[id.rem_euclid(2) as usize]
    }

    /// `ordinal`.
    pub fn ordinal(&self) -> i32 {
        *self as i32
    }
}

/// `ServerboundSeenAdvancementsPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundSeenAdvancementsPacket {
    /// `action`.
    pub action: SeenAdvancementsAction,
    /// `tab` — `Some` только для `OPENED_TAB`.
    pub tab: Option<Identifier>,
}

impl ServerboundSeenAdvancementsPacket {
    /// `openedTab(advancement)` — tab = advancement id.
    pub fn opened_tab(tab: Identifier) -> Self {
        Self { action: SeenAdvancementsAction::OpenedTab, tab: Some(tab) }
    }

    /// `closedScreen()`.
    pub fn closed_screen() -> Self {
        Self { action: SeenAdvancementsAction::ClosedScreen, tab: None }
    }

    /// `getAction`.
    pub fn get_action(&self) -> SeenAdvancementsAction {
        self.action
    }

    /// `getTab`.
    pub fn get_tab(&self) -> Option<&Identifier> {
        self.tab.as_ref()
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundSeenAdvancementsPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundSeenAdvancementsPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let action = SeenAdvancementsAction::by_id(buf.read_var_int());
                let tab = if action == SeenAdvancementsAction::OpenedTab {
                    Some(buf.read_identifier().expect("invalid tab id"))
                } else {
                    None
                };
                ServerboundSeenAdvancementsPacket { action, tab }
            },
        )
    }
}

impl Packet for ServerboundSeenAdvancementsPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_seen_advancements::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.action.ordinal());
        if self.action == SeenAdvancementsAction::OpenedTab {
            buf.write_identifier(self.tab.as_ref().expect("tab required")).expect("invalid identifier");
        }
    }
}
