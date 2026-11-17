//! Перенос `ServerboundUseItemPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::interaction_hand::InteractionHand;

/// `ServerboundUseItemPacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ServerboundUseItemPacket {
    /// `hand`.
    pub hand: InteractionHand,
    /// `sequence`.
    pub sequence: i32,
    /// `yRot`.
    pub y_rot: f32,
    /// `xRot`.
    pub x_rot: f32,
}

impl ServerboundUseItemPacket {
    pub fn new(hand: InteractionHand, sequence: i32, y_rot: f32, x_rot: f32) -> Self {
        Self { hand, sequence, y_rot, x_rot }
    }

    /// `getHand`.
    pub fn get_hand(&self) -> InteractionHand {
        self.hand
    }

    /// `getSequence`.
    pub fn get_sequence(&self) -> i32 {
        self.sequence
    }

    /// `getYRot`.
    pub fn get_y_rot(&self) -> f32 {
        self.y_rot
    }

    /// `getXRot`.
    pub fn get_x_rot(&self) -> f32 {
        self.x_rot
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundUseItemPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundUseItemPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let hand = InteractionHand::by_id(buf.read_var_int());
                let sequence = buf.read_var_int();
                let y_rot = buf.read_float();
                let x_rot = buf.read_float();

                ServerboundUseItemPacket::new(hand, sequence, y_rot, x_rot)
            },
        )
    }
}

impl Packet for ServerboundUseItemPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_use_item::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.hand.ordinal());
        buf.write_var_int(self.sequence);
        buf.write_float(self.y_rot);
        buf.write_float(self.x_rot);
    }
}
