//! Перенос `ClientboundLoginFinishedPacket`.

use crate::authlib::game_profile::GameProfile;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static LOGIN_FINISHED_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(
        PacketFlow::CLIENTBOUND,
        Identifier::with_default_namespace("login_finished").unwrap(),
    )
});

/// `ClientboundLoginFinishedPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundLoginFinishedPacket {
    /// `gameProfile`.
    pub game_profile: GameProfile,
}

impl ClientboundLoginFinishedPacket {
    pub fn new(game_profile: GameProfile) -> Self {
        Self { game_profile }
    }

    /// `gameProfile()`.
    pub fn game_profile(&self) -> &GameProfile {
        &self.game_profile
    }

    /// `STREAM_CODEC` (`ByteBufCodecs.GAME_PROFILE`).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundLoginFinishedPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundLoginFinishedPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundLoginFinishedPacket::new(byte_buf_codecs::game_profile().decode(buf))
            },
        )
    }
}

impl Packet for ClientboundLoginFinishedPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        LOGIN_FINISHED_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        byte_buf_codecs::game_profile().encode(buf, &self.game_profile);
    }

    fn is_terminal(&self) -> bool {
        true
    }
}
