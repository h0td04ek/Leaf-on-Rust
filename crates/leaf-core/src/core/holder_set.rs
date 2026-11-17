//! Перенос `net.minecraft.core.HolderSet<T>` — набор holder'ов:
//! либо прямой список, либо именованный тег.
//!
//! `ByteBufCodecs.holderSet` wire-формат:
//! - VarInt `i = count + 1` (>0) → direct: `i` holders следуют
//! - VarInt `0` → named: `Identifier` тега
//! (чтение: `i = readVarInt - 1`; `i == -1` → tag).

use crate::core::holder::{holder_registry_stream_codec, Holder};
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use crate::tags::TagKey;

/// `HolderSet<T>` — `Direct` (список) или `Named` (тег).
#[derive(Clone, Debug, PartialEq)]
pub enum HolderSet<T> {
    /// `HolderSet.direct(list)`.
    Direct(Vec<Holder<T>>),
    /// `HolderSet.Named` — тег реестра.
    Named(Identifier),
}

impl<T> HolderSet<T> {
    /// `direct` из одного holder'а.
    pub fn direct_one(holder: Holder<T>) -> Self {
        Self::Direct(vec![holder])
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        match self {
            HolderSet::Direct(v) => v.is_empty(),
            HolderSet::Named(_) => false,
        }
    }

    /// `size`.
    pub fn size(&self) -> usize {
        match self {
            HolderSet::Direct(v) => v.len(),
            HolderSet::Named(_) => 0,
        }
    }

    /// Прямые holder'ы (для `Direct`); `Named` резолвится через
    /// реестр тегов — до его порта итерация пуста.
    pub fn direct_holders(&self) -> &[Holder<T>] {
        match self {
            HolderSet::Direct(v) => v,
            HolderSet::Named(_) => &[],
        }
    }

    /// Итератор по элементам (Direct; Named — пусто до резолва).
    pub fn iter(&self) -> std::slice::Iter<'_, Holder<T>> {
        self.direct_holders().iter()
    }

    /// `stream()`-эквивалент для Direct.
    pub fn as_direct(&self) -> Option<&[Holder<T>]> {
        match self {
            HolderSet::Direct(v) => Some(v),
            HolderSet::Named(_) => None,
        }
    }
}

/// `ByteBufCodecs.holderSet(registryKey)` — codec для `HolderSet<T>`,
/// где элементы — registry-references (`holderRegistry`).
pub fn holder_set_stream_codec<T>() -> impl StreamCodec<FriendlyByteBuf, HolderSet<T>> + Send + Sync + 'static
where
    T: Send + Sync + 'static,
{
    let element_enc = holder_registry_stream_codec::<T>();
    let element_dec = holder_registry_stream_codec::<T>();
    sync_fn(
        move |buf: &mut FriendlyByteBuf, set: &HolderSet<T>| match set {
            HolderSet::Direct(list) => {
                buf.write_var_int(list.len() as i32 + 1);
                for h in list {
                    element_enc.encode(buf, h);
                }
            }
            HolderSet::Named(tag) => {
                buf.write_var_int(0);
                buf.write_identifier(tag).expect("holder set tag");
            }
        },
        move |buf: &mut FriendlyByteBuf| {
            let i = buf.read_var_int() - 1;
            if i == -1 {
                let tag = buf.read_identifier().expect("holder set tag");
                HolderSet::Named(tag)
            } else {
                let mut list = Vec::with_capacity(i.max(0) as usize);
                for _ in 0..i.max(0) {
                    list.push(element_dec.decode(buf));
                }
                HolderSet::Direct(list)
            }
        },
    )
}

/// Хелпер для `TagKey`-совместимого доступа к Named-тегу.
pub fn named_tag<T>(set: &HolderSet<T>) -> Option<&Identifier> {
    match set {
        HolderSet::Named(id) => Some(id),
        _ => None,
    }
}

#[allow(dead_code)]
fn _tagkey_marker(_: &TagKey<()>) {}
