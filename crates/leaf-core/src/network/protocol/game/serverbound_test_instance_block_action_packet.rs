//! Перенос `ServerboundTestInstanceBlockActionPacket`.

use crate::core::block_pos::BlockPos;
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::level::block::entity::test_instance_block_entity::{
    TestInstanceBlockData, TestInstanceBlockStatus,
};

/// `ServerboundTestInstanceBlockActionPacket.Action`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TestInstanceBlockAction {
    /// `INIT`.
    INIT,
    /// `QUERY`.
    QUERY,
    /// `SET`.
    SET,
    /// `RESET`.
    RESET,
    /// `SAVE`.
    SAVE,
    /// `EXPORT`.
    EXPORT,
    /// `RUN`.
    RUN,
}

impl TestInstanceBlockAction {
    /// `values()` — порядок = id.
    pub const VALUES: [TestInstanceBlockAction; 7] = [
        TestInstanceBlockAction::INIT,
        TestInstanceBlockAction::QUERY,
        TestInstanceBlockAction::SET,
        TestInstanceBlockAction::RESET,
        TestInstanceBlockAction::SAVE,
        TestInstanceBlockAction::EXPORT,
        TestInstanceBlockAction::RUN,
    ];

    /// `id`.
    pub fn id(&self) -> i32 {
        *self as i32
    }

    /// `ByIdMap.continuous(..., ZERO)` — вне диапазона → `INIT`.
    pub fn by_id(id: i32) -> TestInstanceBlockAction {
        if (0..7).contains(&id) {
            Self::VALUES[id as usize]
        } else {
            Self::VALUES[0]
        }
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.idMapper` (VarInt id).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, TestInstanceBlockAction> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, v: &TestInstanceBlockAction| {
                buf.write_var_int(v.id());
            },
            |buf: &mut FriendlyByteBuf| TestInstanceBlockAction::by_id(buf.read_var_int()),
        )
    }
}

/// `ServerboundTestInstanceBlockActionPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundTestInstanceBlockActionPacket {
    /// `pos`.
    pub pos: BlockPos,
    /// `action`.
    pub action: TestInstanceBlockAction,
    /// `data`.
    pub data: TestInstanceBlockData,
}

impl ServerboundTestInstanceBlockActionPacket {
    /// `new`.
    pub fn new(pos: BlockPos, action: TestInstanceBlockAction, data: TestInstanceBlockData) -> Self {
        Self { pos, action, data }
    }

    /// Удобный конструктор с полями `Data` (status=CLEARED).
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_fields(
        pos: BlockPos,
        action: TestInstanceBlockAction,
        test: Option<crate::resources::ResourceKey<crate::world::gametest::GameTestInstance>>,
        size: crate::core::vec3i::Vec3i,
        rotation: crate::world::level::block::Rotation,
        ignore_entities: bool,
    ) -> Self {
        Self::new(
            pos,
            action,
            TestInstanceBlockData {
                test,
                size,
                rotation,
                ignore_entities,
                status: TestInstanceBlockStatus::CLEARED,
                error_message: None,
            },
        )
    }

    /// Геттеры.
    pub fn pos(&self) -> BlockPos {
        self.pos
    }
    /// `action`.
    pub fn action(&self) -> TestInstanceBlockAction {
        self.action
    }
    /// `data`.
    pub fn data(&self) -> &TestInstanceBlockData {
        &self.data
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundTestInstanceBlockActionPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundTestInstanceBlockActionPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let pos = buf.read_block_pos();
                let action = TestInstanceBlockAction::stream_codec().decode(buf);
                let data = TestInstanceBlockData::stream_codec().decode(buf);
                ServerboundTestInstanceBlockActionPacket::new(pos, action, data)
            },
        )
    }
}

impl Packet for ServerboundTestInstanceBlockActionPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_test_instance_block_action::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_block_pos(&self.pos);
        TestInstanceBlockAction::stream_codec().encode(buf, &self.action);
        TestInstanceBlockData::stream_codec().encode(buf, &self.data);
    }
}
