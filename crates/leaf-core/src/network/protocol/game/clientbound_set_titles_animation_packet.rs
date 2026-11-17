//! Перенос `ClientboundSetTitlesAnimationPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetTitlesAnimationPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundSetTitlesAnimationPacket {
    /// `fadeIn`.
    pub fade_in: i32,
    /// `stay`.
    pub stay: i32,
    /// `fadeOut`.
    pub fade_out: i32,
}

impl ClientboundSetTitlesAnimationPacket {
    pub fn new(fade_in: i32, stay: i32, fade_out: i32) -> Self {
        Self { fade_in, stay, fade_out }
    }

    /// `getFadeIn`.
    pub fn get_fade_in(&self) -> i32 {
        self.fade_in
    }

    /// `getStay`.
    pub fn get_stay(&self) -> i32 {
        self.stay
    }

    /// `getFadeOut`.
    pub fn get_fade_out(&self) -> i32 {
        self.fade_out
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetTitlesAnimationPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetTitlesAnimationPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let fade_in = buf.read_int();
                let stay = buf.read_int();
                let fade_out = buf.read_int();

                ClientboundSetTitlesAnimationPacket::new(fade_in, stay, fade_out)
            },
        )
    }
}

impl Packet for ClientboundSetTitlesAnimationPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_titles_animation::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_int(self.fade_in);
        buf.write_int(self.stay);
        buf.write_int(self.fade_out);
    }
}
