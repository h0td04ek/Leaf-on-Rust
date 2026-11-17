//! Перенос `ClientboundInitializeBorderPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundInitializeBorderPacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundInitializeBorderPacket {
    /// `newCenterX`.
    pub new_center_x: f64,
    /// `newCenterZ`.
    pub new_center_z: f64,
    /// `oldSize`.
    pub old_size: f64,
    /// `newSize`.
    pub new_size: f64,
    /// `lerpTime`.
    pub lerp_time: i64,
    /// `newAbsoluteMaxSize`.
    pub new_absolute_max_size: i32,
    /// `warningBlocks`.
    pub warning_blocks: i32,
    /// `warningTime`.
    pub warning_time: i32,
}

impl ClientboundInitializeBorderPacket {
    /// `new ClientboundInitializeBorderPacket(...)` — 8 полей.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        new_center_x: f64,
        new_center_z: f64,
        old_size: f64,
        new_size: f64,
        lerp_time: i64,
        new_absolute_max_size: i32,
        warning_blocks: i32,
        warning_time: i32,
    ) -> Self {
        Self {
            new_center_x,
            new_center_z,
            old_size,
            new_size,
            lerp_time,
            new_absolute_max_size,
            warning_blocks,
            warning_time,
        }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundInitializeBorderPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundInitializeBorderPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let new_center_x = buf.read_double();
                let new_center_z = buf.read_double();
                let old_size = buf.read_double();
                let new_size = buf.read_double();
                let lerp_time = buf.read_var_long();
                let new_absolute_max_size = buf.read_var_int();
                let warning_blocks = buf.read_var_int();
                let warning_time = buf.read_var_int();
                ClientboundInitializeBorderPacket::new(
                    new_center_x,
                    new_center_z,
                    old_size,
                    new_size,
                    lerp_time,
                    new_absolute_max_size,
                    warning_blocks,
                    warning_time,
                )
            },
        )
    }
}

impl Packet for ClientboundInitializeBorderPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_initialize_border::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_double(self.new_center_x);
        buf.write_double(self.new_center_z);
        buf.write_double(self.old_size);
        buf.write_double(self.new_size);
        buf.write_var_long(self.lerp_time);
        buf.write_var_int(self.new_absolute_max_size);
        buf.write_var_int(self.warning_blocks);
        buf.write_var_int(self.warning_time);
    }
}
