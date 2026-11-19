//! Перенос `net.minecraft.world.item.component.BlockItemStateProperties`.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use std::collections::BTreeMap;

/// `BlockItemStateProperties` — `Map<String, String>`
/// (`BlockItemStateProperties.STREAM_CODEC` — map(STRING, STRING)).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BlockItemStateProperties {
    /// `properties`.
    pub properties: BTreeMap<String, String>,
}

impl BlockItemStateProperties {
    /// `new`.
    pub fn new(properties: BTreeMap<String, String>) -> Self {
        Self { properties }
    }

    /// `apply(state)` — применяет свойства по имени к
    /// `BlockState` (неизвестные имена игнорируются).
    pub fn apply(
        &self,
        state: &crate::world::level::block::state::block_state::BlockState,
    ) -> crate::world::level::block::state::block_state::BlockState {
        use crate::world::level::block::state::property::StateValue;
        let mut new_state = state.clone();
        for (name, value) in &self.properties {
            // Проверяем, что свойство существует в state.
            if new_state.definition.get_property(name).is_none() {
                continue;
            }
            let v = if let Ok(i) = value.parse::<i32>() {
                StateValue::Int(i)
            } else if let Ok(b) = value.parse::<bool>() {
                StateValue::Bool(b)
            } else {
                StateValue::Str(value.clone())
            };
            new_state.values.set_erased(name, v);
        }
        new_state
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.map(Object2ObjectOpenHashMap,
    /// STRING_UTF8, STRING_UTF8)` — varint size + пары.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, BlockItemStateProperties> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &BlockItemStateProperties| {
                buf.write_var_int(v.properties.len() as i32);
                for (k, val) in &v.properties {
                    buf.write_utf(k).expect("property key");
                    buf.write_utf(val).expect("property value");
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let count = buf.read_var_int().max(0) as usize;
                let mut properties = BTreeMap::new();
                for _ in 0..count {
                    let k = buf.read_utf().expect("property key");
                    let val = buf.read_utf().expect("property value");
                    properties.insert(k, val);
                }
                BlockItemStateProperties { properties }
            },
        )
    }
}
