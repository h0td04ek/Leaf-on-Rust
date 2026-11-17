//! Перенос `net.minecraft.world.item.Rarity`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `Rarity` — порядок = `id` (0..3).
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rarity {
    /// `COMMON`.
    COMMON,
    /// `UNCOMMON`.
    UNCOMMON,
    /// `RARE`.
    RARE,
    /// `EPIC`.
    EPIC,
}

impl Rarity {
    /// `values()`.
    pub const VALUES: [Rarity; 4] = [Rarity::COMMON, Rarity::UNCOMMON, Rarity::RARE, Rarity::EPIC];

    /// `id`.
    pub fn id(&self) -> i32 {
        *self as i32
    }

    /// `ByIdMap.continuous(..., ZERO)` — вне диапазона → `COMMON`.
    pub fn by_id(id: i32) -> Rarity {
        if (0..4).contains(&id) {
            Self::VALUES[id as usize]
        } else {
            Self::VALUES[0]
        }
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            Rarity::COMMON => "common",
            Rarity::UNCOMMON => "uncommon",
            Rarity::RARE => "rare",
            Rarity::EPIC => "epic",
        }
    }

    /// `CODEC.byName`.
    pub fn by_name(name: &str) -> Option<Rarity> {
        Self::VALUES.iter().copied().find(|r| r.get_serialized_name() == name)
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.idMapper(BY_ID, id)` (VarInt id).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Rarity> {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &Rarity| {
                buf.write_var_int(v.id());
            },
            |buf: &mut FriendlyByteBuf| Rarity::by_id(buf.read_var_int()),
        )
    }
}
