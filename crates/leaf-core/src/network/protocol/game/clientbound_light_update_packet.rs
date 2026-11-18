//! `ClientboundLightUpdatePacket` — обновление освещения чанка.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::game::clientbound_light_update_packet_data::ClientboundLightUpdatePacketData;

/// `ClientboundLightUpdatePacket`.
#[derive(Clone, Debug)]
pub struct ClientboundLightUpdatePacket {
    /// `x` — chunk x.
    pub x: i32,
    /// `z` — chunk z.
    pub z: i32,
    /// `lightData`.
    pub light_data: ClientboundLightUpdatePacketData,
}

impl ClientboundLightUpdatePacket {
    /// `new(x, z, lightData)`.
    pub fn new(x: i32, z: i32, light_data: ClientboundLightUpdatePacketData) -> Self {
        Self { x, z, light_data }
    }

    /// `STREAM_CODEC` — `Packet.codec(write, new)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundLightUpdatePacket> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, pkt: &ClientboundLightUpdatePacket| {
                buf.write_var_int(pkt.x);
                buf.write_var_int(pkt.z);
                pkt.light_data.write(buf);
            },
            |buf: &mut FriendlyByteBuf| {
                let x = buf.read_var_int();
                let z = buf.read_var_int();
                let light_data = ClientboundLightUpdatePacketData::read(buf);
                Self::new(x, z, light_data)
            },
        )
    }

    /// `getX`.
    pub fn get_x(&self) -> i32 {
        self.x
    }

    /// `getZ`.
    pub fn get_z(&self) -> i32 {
        self.z
    }

    /// `getLightData`.
    pub fn get_light_data(&self) -> &ClientboundLightUpdatePacketData {
        &self.light_data
    }
}

impl Packet for ClientboundLightUpdatePacket {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_light_update::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.x);
        buf.write_var_int(self.z);
        self.light_data.write(buf);
    }
}
