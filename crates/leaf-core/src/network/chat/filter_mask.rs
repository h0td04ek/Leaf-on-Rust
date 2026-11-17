//! Перенос `net.minecraft.network.chat.FilterMask` (+ `Type`).

use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `FilterMask.Type` — строковые имена + порядковые id (writeEnum/readEnum).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterMaskType {
    /// `pass_through`.
    PassThrough = 0,
    /// `fully_filtered`.
    FullyFiltered = 1,
    /// `partially_filtered`.
    PartiallyFiltered = 2,
}

impl FilterMaskType {
    /// `values()`-порядок.
    pub const VALUES: [FilterMaskType; 3] = [
        FilterMaskType::PassThrough,
        FilterMaskType::FullyFiltered,
        FilterMaskType::PartiallyFiltered,
    ];

    /// `byId` — ordinal.
    pub fn by_id(id: i32) -> Self {
        Self::VALUES[id.rem_euclid(3) as usize]
    }

    /// `ordinal` (`writeEnum`).
    pub fn ordinal(&self) -> i32 {
        *self as i32
    }

    /// `getSerializedName`.
    pub fn serialized_name(&self) -> &'static str {
        match self {
            FilterMaskType::PassThrough => "pass_through",
            FilterMaskType::FullyFiltered => "fully_filtered",
            FilterMaskType::PartiallyFiltered => "partially_filtered",
        }
    }
}

/// `FilterMask` — маска отфильтрованных символов.
/// `mask` — bitset-слова (little-endian биты по Java BitSet).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilterMask {
    /// `mask` — биты (PartiallyFiltered); для остальных типов пусто.
    pub mask: Vec<u64>,
    /// `type`.
    pub mask_type: FilterMaskType,
}

/// `PASS_THROUGH`.
pub fn pass_through() -> FilterMask {
    FilterMask { mask: Vec::new(), mask_type: FilterMaskType::PassThrough }
}

/// `FULLY_FILTERED`.
pub fn fully_filtered() -> FilterMask {
    FilterMask { mask: Vec::new(), mask_type: FilterMaskType::FullyFiltered }
}

impl FilterMask {
    /// `new FilterMask(size)` — частичная маска заданного размера.
    pub fn partially_filtered(size: usize) -> Self {
        Self {
            mask: vec![0u64; (size + 63) / 64],
            mask_type: FilterMaskType::PartiallyFiltered,
        }
    }

    /// `read`.
    pub fn read(buffer: &mut FriendlyByteBuf) -> Self {
        let mask_type = FilterMaskType::by_id(buffer.read_var_int());
        match mask_type {
            FilterMaskType::PassThrough => pass_through(),
            FilterMaskType::FullyFiltered => fully_filtered(),
            FilterMaskType::PartiallyFiltered => Self {
                mask: buffer.read_bit_set().into_iter().map(|w| w as u64).collect(),
                mask_type,
            },
        }
    }

    /// `write`.
    pub fn write(&self, buffer: &mut FriendlyByteBuf) {
        buffer.write_var_int(self.mask_type.ordinal());
        if self.mask_type == FilterMaskType::PartiallyFiltered {
            buffer.write_bit_set(&self.mask.iter().map(|w| *w as i64).collect::<Vec<_>>());
        }
    }

    /// `setFiltered(bitIndex)`.
    pub fn set_filtered(&mut self, bit_index: usize) {
        let word = bit_index / 64;
        let bit = bit_index % 64;
        if self.mask.len() <= word {
            self.mask.resize(word + 1, 0);
        }
        self.mask[word] |= 1u64 << bit;
    }

    /// `mask.get(i)`.
    pub fn is_filtered(&self, i: usize) -> bool {
        let word = i / 64;
        let bit = i % 64;
        self.mask.get(word).map_or(false, |w| (w >> bit) & 1 == 1)
    }

    /// `apply(text)` — `None` = полностью отфильтровано; иначе текст с `#`.
    pub fn apply(&self, text: &str) -> Option<String> {
        match self.mask_type {
            FilterMaskType::PassThrough => Some(text.to_string()),
            FilterMaskType::FullyFiltered => None,
            FilterMaskType::PartiallyFiltered => {
                let mut chars: Vec<char> = text.chars().collect();
                let len = chars.len().min(self.mask.iter().map(|w| w.count_ones() as usize * 64).sum::<usize>().max(chars.len()));
                let _ = len;
                for i in 0..chars.len() {
                    if i < self.mask_bit_length() && self.is_filtered(i) {
                        chars[i] = '#';
                    }
                }
                Some(chars.into_iter().collect())
            }
        }
    }

    fn mask_bit_length(&self) -> usize {
        // Java BitSet.length() — индекс старшего установленного бита + 1
        for (i, w) in self.mask.iter().enumerate().rev() {
            if *w != 0 {
                return i * 64 + (64 - w.leading_zeros() as usize);
            }
        }
        0
    }
}
