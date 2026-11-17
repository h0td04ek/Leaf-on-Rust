//! Перенос `net.minecraft.world.item.enchantment.ItemEnchantments`.

use crate::core::holder::Holder;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::enchantment::enchantment::Enchantment;

/// `ItemEnchantments` — `Object2IntAVLTreeMap<Holder<Enchantment>,
/// Integer>` (уровень 1..=32767).
/// Порядок сортировки (`ENCHANTMENT_ORDER` по имени) влияет только
/// на детерминизм NBT; на проводе — map(code: varint size + пары).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemEnchantments {
    /// `enchantments` — пары (holder, level).
    pub enchantments: Vec<(Holder<Enchantment>, i32)>,
}

impl ItemEnchantments {
    /// `EMPTY`.
    pub fn empty() -> Self {
        Self { enchantments: Vec::new() }
    }

    /// `new` — проверяет диапазон уровня.
    pub fn new(enchantments: Vec<(Holder<Enchantment>, i32)>) -> Self {
        for (_, level) in &enchantments {
            assert!((1..=32767).contains(level), "Enchantment has invalid level {level}");
        }
        Self { enchantments }
    }

    /// `getLevel`.
    pub fn get_level(&self, enchantment: &Holder<Enchantment>) -> i32 {
        self.enchantments
            .iter()
            .find(|(h, _)| h == enchantment)
            .map(|(_, l)| *l)
            .unwrap_or(0)
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.enchantments.is_empty()
    }

    /// `STREAM_CODEC` — map(Enchantment.STREAM_CODEC, VAR_INT).
    /// `ByteBufCodecs.map` пишет count + пары key/value.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ItemEnchantments> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &ItemEnchantments| {
                buf.write_var_int(v.enchantments.len() as i32);
                for (holder, level) in &v.enchantments {
                    Enchantment::stream_codec().encode(buf, holder);
                    buf.write_var_int(*level);
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let count = buf.read_var_int().max(0) as usize;
                let mut enchantments = Vec::with_capacity(count);
                for _ in 0..count {
                    let holder = Enchantment::stream_codec().decode(buf);
                    let level = buf.read_var_int();
                    enchantments.push((holder, level));
                }
                ItemEnchantments::new(enchantments)
            },
        )
    }
}
