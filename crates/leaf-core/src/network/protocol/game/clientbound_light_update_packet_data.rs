//! `ClientboundLightUpdatePacketData` — данные освещения для клиента.

use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `ClientboundLightUpdatePacketData`.
#[derive(Clone, Debug)]
pub struct ClientboundLightUpdatePacketData {
    /// `skyYMask` — BitSet для sky light.
    pub sky_y_mask: Vec<bool>,
    /// `blockYMask` — BitSet для block light.
    pub block_y_mask: Vec<bool>,
    /// `emptySkyYMask` — BitSet для пустого sky light.
    pub empty_sky_y_mask: Vec<bool>,
    /// `emptyBlockYMask` — BitSet для пустого block light.
    pub empty_block_y_mask: Vec<bool>,
    /// `skyUpdates` — массивы данных sky light.
    pub sky_updates: Vec<Vec<u8>>,
    /// `blockUpdates` — массивы данных block light.
    pub block_updates: Vec<Vec<u8>>,
}

impl ClientboundLightUpdatePacketData {
    /// `new(...)` — создаёт из light engine (упрощённо).
    pub fn new(
        sky_y_mask: Vec<bool>,
        block_y_mask: Vec<bool>,
        empty_sky_y_mask: Vec<bool>,
        empty_block_y_mask: Vec<bool>,
        sky_updates: Vec<Vec<u8>>,
        block_updates: Vec<Vec<u8>>,
    ) -> Self {
        Self {
            sky_y_mask,
            block_y_mask,
            empty_sky_y_mask,
            empty_block_y_mask,
            sky_updates,
            block_updates,
        }
    }

    /// `write` — сериализация.
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        // BitSets — write as byte arrays
        Self::write_bitset(buf, &self.sky_y_mask);
        Self::write_bitset(buf, &self.block_y_mask);
        Self::write_bitset(buf, &self.empty_sky_y_mask);
        Self::write_bitset(buf, &self.empty_block_y_mask);
        // Arrays — write count then each array
        buf.write_var_int(self.sky_updates.len() as i32);
        for arr in &self.sky_updates {
            buf.write_var_int(arr.len() as i32);
            buf.write_bytes(arr);
        }
        buf.write_var_int(self.block_updates.len() as i32);
        for arr in &self.block_updates {
            buf.write_var_int(arr.len() as i32);
            buf.write_bytes(arr);
        }
    }

    /// `read` — десериализация.
    pub fn read(buf: &mut FriendlyByteBuf, _x: i32, _z: i32) -> Self {
        let sky_y_mask = Self::read_bitset(buf);
        let block_y_mask = Self::read_bitset(buf);
        let empty_sky_y_mask = Self::read_bitset(buf);
        let empty_block_y_mask = Self::read_bitset(buf);
        let sky_updates = Self::read_byte_arrays(buf);
        let block_updates = Self::read_byte_arrays(buf);
        Self::new(sky_y_mask, block_y_mask, empty_sky_y_mask, empty_block_y_mask, sky_updates, block_updates)
    }

    fn write_bitset(buf: &mut FriendlyByteBuf, bits: &[bool]) {
        // BitSet serialization — write as long array
        let longs = (bits.len() + 63) / 64;
        buf.write_var_int(longs as i32);
        for i in 0..longs {
            let mut val: i64 = 0;
            for j in 0..64 {
                let idx = i * 64 + j;
                if idx < bits.len() && bits[idx] {
                    val |= 1i64 << j;
                }
            }
            buf.source.write_i64(val);
        }
    }

    fn read_bitset(buf: &mut FriendlyByteBuf) -> Vec<bool> {
        let longs = buf.read_var_int();
        let mut bits = Vec::new();
        for _ in 0..longs {
            let val = buf.source.read_i64();
            for j in 0..64 {
                bits.push((val >> j) & 1 == 1);
            }
        }
        bits
    }

    fn read_byte_arrays(buf: &mut FriendlyByteBuf) -> Vec<Vec<u8>> {
        let count = buf.read_var_int();
        let mut arrays = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let len = buf.read_var_int();
            arrays.push(buf.read_bytes_vec(len as usize));
        }
        arrays
    }
}
