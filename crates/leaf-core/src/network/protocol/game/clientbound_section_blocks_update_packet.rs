//! Перенос `ClientboundSectionBlocksUpdatePacket`.

use crate::core::section_pos::SectionPos;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

const POS_IN_SECTION_BITS: i64 = 12;
const POS_IN_SECTION_MASK: i64 = (1 << POS_IN_SECTION_BITS) - 1;

/// `ClientboundSectionBlocksUpdatePacket`.
///
/// `states` — id'ы в глобальной палитре `Block.BLOCK_STATE_REGISTRY`
/// (реестр блоков портируется отдельно; формат — VarLong
/// `(stateId << 12) | packedSectionPos`).
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSectionBlocksUpdatePacket {
    /// `sectionPos`.
    pub section_pos: SectionPos,
    /// `positions` — packed short (x | y<<4 | z<<8 внутри секции).
    pub positions: Vec<i16>,
    /// `states` — block state ids.
    pub states: Vec<i32>,
}

impl ClientboundSectionBlocksUpdatePacket {
    /// `ClientboundSectionBlocksUpdatePacket::new` (CraftBukkit-вариант).
    pub fn new(section_pos: SectionPos, positions: Vec<i16>, states: Vec<i32>) -> Self {
        assert_eq!(positions.len(), states.len());
        Self { section_pos, positions, states }
    }

    /// `positions()`/`states()`.
    pub fn positions(&self) -> &[i16] {
        &self.positions
    }

    /// `states()`.
    pub fn states(&self) -> &[i32] {
        &self.states
    }

    /// `sectionPos()`.
    pub fn section_pos(&self) -> SectionPos {
        self.section_pos
    }

    /// `runUpdates` — для каждого изменения вызвать callback с
    /// абсолютным `BlockPos` и block state id.
    pub fn run_updates(&self, mut consumer: impl FnMut(crate::core::block_pos::BlockPos, i32)) {
        for (i, &s) in self.positions.iter().enumerate() {
            let pos = crate::core::block_pos::BlockPos::new(
                self.section_pos.relative_to_block_x(s),
                self.section_pos.relative_to_block_y(s),
                self.section_pos.relative_to_block_z(s),
            );
            consumer(pos, self.states[i]);
        }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSectionBlocksUpdatePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSectionBlocksUpdatePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let section_pos = buf.read_section_pos();
                let count = buf.read_var_int();
                let mut positions = Vec::with_capacity(count.max(0) as usize);
                let mut states = Vec::with_capacity(count.max(0) as usize);
                for _ in 0..count {
                    let var_long = buf.read_var_long();
                    positions.push((var_long & POS_IN_SECTION_MASK) as i16);
                    states.push((var_long >> POS_IN_SECTION_BITS) as i32);
                }
                ClientboundSectionBlocksUpdatePacket::new(section_pos, positions, states)
            },
        )
    }
}

impl Packet for ClientboundSectionBlocksUpdatePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_section_blocks_update::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_section_pos(&self.section_pos);
        buf.write_var_int(self.positions.len() as i32);
        for (i, &pos) in self.positions.iter().enumerate() {
            buf.write_var_long(((self.states[i] as i64) << POS_IN_SECTION_BITS) | (pos as i64 & POS_IN_SECTION_MASK));
        }
    }
}
