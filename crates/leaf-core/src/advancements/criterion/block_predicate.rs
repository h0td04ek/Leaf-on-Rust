//! Перенос `net.minecraft.advancements.criterion.BlockPredicate` —
//! record `(blocks?, properties?, nbt?, components)`.
//!
//! `STREAM_CODEC` — `composite(optional(holderSet(BLOCK)),
//! optional(StatePropertiesPredicate.STREAM_CODEC),
//! optional(NbtPredicate.STREAM_CODEC), DataComponentMatchers
//! .STREAM_CODEC)`.

use crate::advancements::criterion::data_component_matchers::DataComponentMatchers;
use crate::advancements::criterion::nbt_predicate::NbtPredicate;
use crate::advancements::criterion::state_properties_predicate::StatePropertiesPredicate;
use crate::core::block_pos::BlockPos;
use crate::core::holder_set::HolderSet;
use crate::nbt::tag::Tag;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::pattern::block_in_world::BlockInWorld;
use crate::world::level::block::Block;
use crate::world::level::level_reader::LevelReader;
use std::sync::Arc;

/// `BlockPredicate`.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockPredicate {
    /// `blocks` — `HolderSet<Block>`.
    pub blocks: Option<HolderSet<Block>>,
    /// `properties` — `StatePropertiesPredicate`.
    pub properties: Option<StatePropertiesPredicate>,
    /// `nbt` — `NbtPredicate` по block entity NBT.
    pub nbt: Option<NbtPredicate>,
    /// `components` — `DataComponentMatchers`.
    pub components: DataComponentMatchers,
}

impl BlockPredicate {
    /// `new`.
    pub fn new(
        blocks: Option<HolderSet<Block>>,
        properties: Option<StatePropertiesPredicate>,
        nbt: Option<NbtPredicate>,
        components: DataComponentMatchers,
    ) -> Self {
        Self { blocks, properties, nbt, components }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, BlockPredicate> + Send + Sync + 'static {
        let blocks_enc = byte_buf_codecs::optional(crate::core::holder_set::holder_set_stream_codec::<Block>());
        let blocks_dec = byte_buf_codecs::optional(crate::core::holder_set::holder_set_stream_codec::<Block>());
        let props_enc = byte_buf_codecs::optional(StatePropertiesPredicate::stream_codec());
        let props_dec = byte_buf_codecs::optional(StatePropertiesPredicate::stream_codec());
        let nbt_enc = byte_buf_codecs::optional(nbt_stream_codec());
        let nbt_dec = byte_buf_codecs::optional(nbt_stream_codec());
        let comp_enc = DataComponentMatchers::stream_codec();
        let comp_dec = DataComponentMatchers::stream_codec();
        sync_fn(
            move |buf: &mut FriendlyByteBuf, p: &BlockPredicate| {
                blocks_enc.encode(buf, &p.blocks);
                props_enc.encode(buf, &p.properties);
                nbt_enc.encode(buf, &p.nbt);
                comp_enc.encode(buf, &p.components);
            },
            move |buf: &mut FriendlyByteBuf| BlockPredicate {
                blocks: blocks_dec.decode(buf),
                properties: props_dec.decode(buf),
                nbt: nbt_dec.decode(buf),
                components: comp_dec.decode(buf),
            },
        )
    }

    /// `matches(ServerLevel, BlockPos)` — упрощённо через
    /// `LevelReader` (ServerLevel-специфика вроде isLoaded уже
    /// покрыта `is_loaded`).
    pub fn matches(&self, level: &dyn LevelReader, pos: BlockPos) -> bool {
        if !level.is_loaded(pos) {
            return false;
        }
        let Some(state) = level.get_block_state(pos) else { return false };
        if !self.matches_state(&state) {
            return false;
        }
        if self.nbt.is_some() || !self.components.is_empty() {
            let entity = level.get_block_entity(pos);
            if let Some(pred) = &self.nbt {
                let Some(entity) = entity.as_ref() else { return false };
                let access = crate::core::registry::builtin_registry_access()
                    .get()
                    .map(|a| a.clone())
                    .unwrap_or_else(|| std::sync::Arc::new(crate::core::registry::RegistryAccessMap::new()));
                let tag = entity.save_full_metadata(&access);
                if !pred.matches(Some(&Tag::Compound(tag))) {
                    return false;
                }
            }
            if !self.components.is_empty() {
                let Some(entity) = entity.as_ref() else { return false };
                if !self.components.test(entity.components()) {
                    return false;
                }
            }
        }
        true
    }

    /// `matches(BlockInWorld)`.
    pub fn matches_block_in_world(&self, block: &BlockInWorld) -> bool {
        let Some(state) = block.get_state() else { return false };
        if !self.matches_state(&state) {
            return false;
        }
        if let Some(pred) = &self.nbt {
            let Some(entity) = block.get_entity() else { return false };
            let access = crate::core::registry::builtin_registry_access()
                .get()
                .cloned()
                .unwrap_or_else(|| std::sync::Arc::new(crate::core::registry::RegistryAccessMap::new()));
            let tag = entity.save_full_metadata(&access);
            return pred.matches(Some(&Tag::Compound(tag)));
        }
        true
    }

    /// `matchesState(BlockState)`.
    fn matches_state(&self, state: &BlockState) -> bool {
        let blocks_ok = match &self.blocks {
            Some(set) => state.is(set),
            None => true,
        };
        let props_ok = match &self.properties {
            Some(p) => p.matches_block_state(state),
            None => true,
        };
        blocks_ok && props_ok
    }

    /// `requiresNbt`.
    pub fn requires_nbt(&self) -> bool {
        self.nbt.is_some()
    }
}

/// `NbtPredicate.STREAM_CODEC` — `COMPOUND_TAG` (compound без
/// обёртки-тега).
fn nbt_stream_codec() -> impl StreamCodec<FriendlyByteBuf, NbtPredicate> + Send + Sync + 'static {
    sync_fn(
        |buf: &mut FriendlyByteBuf, p: &NbtPredicate| {
            buf.write_nbt_compound(Some(&p.tag));
        },
        |buf: &mut FriendlyByteBuf| {
            let tag = buf.read_nbt_compound().expect("nbt predicate compound").unwrap_or_default();
            NbtPredicate::new(tag)
        },
    )
}
