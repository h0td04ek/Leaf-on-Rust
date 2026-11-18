//! `ClientboundLightUpdatePacketData` — данные обновления освещения.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `ClientboundLightUpdatePacketData`.
#[derive(Clone, Debug)]
pub struct ClientboundLightUpdatePacketData {
    /// `skyYMask` — BitSet (Vec<i64>).
    pub sky_y_mask: Vec<i64>,
    /// `blockYMask`.
    pub block_y_mask: Vec<i64>,
    /// `emptySkyYMask`.
    pub empty_sky_y_mask: Vec<i64>,
    /// `emptyBlockYMask`.
    pub empty_block_y_mask: Vec<i64>,
    /// `skyUpdates` — `byte[2048]` массивы.
    pub sky_updates: Vec<Vec<u8>>,
    /// `blockUpdates`.
    pub block_updates: Vec<Vec<u8>>,
    /// `skyUpdateCount`.
    pub sky_update_count: usize,
    /// `blockUpdateCount`.
    pub block_update_count: usize,
}

impl ClientboundLightUpdatePacketData {
    /// `new` — из готовых данных.
    pub fn new(
        sky_y_mask: Vec<i64>,
        block_y_mask: Vec<i64>,
        empty_sky_y_mask: Vec<i64>,
        empty_block_y_mask: Vec<i64>,
        sky_updates: Vec<Vec<u8>>,
        block_updates: Vec<Vec<u8>>,
    ) -> Self {
        let sky_update_count = sky_updates.len();
        let block_update_count = block_updates.len();
        Self {
            sky_y_mask,
            block_y_mask,
            empty_sky_y_mask,
            empty_block_y_mask,
            sky_updates,
            block_updates,
            sky_update_count,
            block_update_count,
        }
    }

    /// `DATA_LAYER_STREAM_CODEC` — `ByteBufCodecs.byteArray(2048)`.
    fn data_layer_stream_codec() -> impl StreamCodec<FriendlyByteBuf, Vec<u8>> + Send + Sync + 'static {
        crate::network::codec::byte_buf_codecs::byte_array(2048)
    }

    /// `write` — запись в буфер.
    pub fn write(&self, buffer: &mut FriendlyByteBuf) {
        buffer.write_bit_set(&self.sky_y_mask);
        buffer.write_bit_set(&self.block_y_mask);
        buffer.write_bit_set(&self.empty_sky_y_mask);
        buffer.write_bit_set(&self.empty_block_y_mask);
        // skyUpdates
        buffer.write_var_int(self.sky_update_count as i32);
        for data in self.sky_updates.iter().take(self.sky_update_count) {
            Self::data_layer_stream_codec().encode(buffer, data);
        }
        // blockUpdates
        buffer.write_var_int(self.block_update_count as i32);
        for data in self.block_updates.iter().take(self.block_update_count) {
            Self::data_layer_stream_codec().encode(buffer, data);
        }
    }

    /// `read` — чтение из буфера.
    pub fn read(buffer: &mut FriendlyByteBuf) -> Self {
        let sky_y_mask = buffer.read_bit_set();
        let block_y_mask = buffer.read_bit_set();
        let empty_sky_y_mask = buffer.read_bit_set();
        let empty_block_y_mask = buffer.read_bit_set();
        // skyUpdates
        let sky_count = buffer.read_var_int() as usize;
        let mut sky_updates = Vec::with_capacity(sky_count);
        for _ in 0..sky_count {
            sky_updates.push(Self::data_layer_stream_codec().decode(buffer));
        }
        // blockUpdates
        let block_count = buffer.read_var_int() as usize;
        let mut block_updates = Vec::with_capacity(block_count);
        for _ in 0..block_count {
            block_updates.push(Self::data_layer_stream_codec().decode(buffer));
        }
        Self::new(
            sky_y_mask,
            block_y_mask,
            empty_sky_y_mask,
            empty_block_y_mask,
            sky_updates,
            block_updates,
        )
    }

    /// `getSkyYMask`.
    pub fn get_sky_y_mask(&self) -> &[i64] {
        &self.sky_y_mask
    }

    /// `getEmptySkyYMask`.
    pub fn get_empty_sky_y_mask(&self) -> &[i64] {
        &self.empty_sky_y_mask
    }

    /// `getSkyUpdates`.
    pub fn get_sky_updates(&self) -> &[Vec<u8>] {
        &self.sky_updates[..self.sky_update_count]
    }

    /// `getBlockYMask`.
    pub fn get_block_y_mask(&self) -> &[i64] {
        &self.block_y_mask
    }

    /// `getEmptyBlockYMask`.
    pub fn get_empty_block_y_mask(&self) -> &[i64] {
        &self.empty_block_y_mask
    }

    /// `getBlockUpdates`.
    pub fn get_block_updates(&self) -> &[Vec<u8>] {
        &self.block_updates[..self.block_update_count]
    }
}
