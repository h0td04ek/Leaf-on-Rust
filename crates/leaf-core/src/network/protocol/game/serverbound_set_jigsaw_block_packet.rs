//! Перенос `ServerboundSetJigsawBlockPacket`.

use crate::core::block_pos::BlockPos;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::resources::Identifier;
use crate::world::level::block::entity::jigsaw_block_entity::JigsawJointType;

/// `ServerboundSetJigsawBlockPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundSetJigsawBlockPacket {
    /// `pos`.
    pub pos: BlockPos,
    /// `name`.
    pub name: Identifier,
    /// `target`.
    pub target: Identifier,
    /// `pool`.
    pub pool: Identifier,
    /// `finalState`.
    pub final_state: String,
    /// `joint`.
    pub joint: JigsawJointType,
    /// `selectionPriority`.
    pub selection_priority: i32,
    /// `placementPriority`.
    pub placement_priority: i32,
}

impl ServerboundSetJigsawBlockPacket {
    /// `new`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pos: BlockPos,
        name: Identifier,
        target: Identifier,
        pool: Identifier,
        final_state: String,
        joint: JigsawJointType,
        selection_priority: i32,
        placement_priority: i32,
    ) -> Self {
        Self {
            pos,
            name,
            target,
            pool,
            final_state,
            joint,
            selection_priority,
            placement_priority,
        }
    }

    /// Геттеры.
    pub fn pos(&self) -> BlockPos {
        self.pos
    }
    /// `name`.
    pub fn name(&self) -> &Identifier {
        &self.name
    }
    /// `target`.
    pub fn target(&self) -> &Identifier {
        &self.target
    }
    /// `pool`.
    pub fn pool(&self) -> &Identifier {
        &self.pool
    }
    /// `finalState`.
    pub fn final_state(&self) -> &str {
        &self.final_state
    }
    /// `joint`.
    pub fn joint(&self) -> JigsawJointType {
        self.joint
    }
    /// `selectionPriority`.
    pub fn selection_priority(&self) -> i32 {
        self.selection_priority
    }
    /// `placementPriority`.
    pub fn placement_priority(&self) -> i32 {
        self.placement_priority
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundSetJigsawBlockPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundSetJigsawBlockPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let pos = buf.read_block_pos();
                let name = buf.read_identifier().expect("invalid identifier");
                let target = buf.read_identifier().expect("invalid identifier");
                let pool = buf.read_identifier().expect("invalid identifier");
                let final_state = buf.read_utf().expect("invalid utf");
                let joint_name = buf.read_utf().expect("invalid utf");
                let joint = JigsawJointType::by_name_or_default(&joint_name);
                let selection_priority = buf.read_var_int();
                let placement_priority = buf.read_var_int();
                ServerboundSetJigsawBlockPacket::new(
                    pos,
                    name,
                    target,
                    pool,
                    final_state,
                    joint,
                    selection_priority,
                    placement_priority,
                )
            },
        )
    }
}

impl Packet for ServerboundSetJigsawBlockPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_set_jigsaw_block::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_block_pos(&self.pos);
        buf.write_identifier(&self.name).expect("invalid identifier");
        buf.write_identifier(&self.target).expect("invalid identifier");
        buf.write_identifier(&self.pool).expect("invalid identifier");
        buf.write_utf(&self.final_state).expect("invalid utf");
        buf.write_utf(self.joint.get_serialized_name()).expect("invalid utf");
        buf.write_var_int(self.selection_priority);
        buf.write_var_int(self.placement_priority);
    }
}
