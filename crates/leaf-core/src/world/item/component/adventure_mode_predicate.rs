//! Перенос `net.minecraft.world.item.component.AdventureModePredicate`
//! — компонент `can_place_on`/`can_break`: список `BlockPredicate`.
//!
//! `STREAM_CODEC` — `BlockPredicate.STREAM_CODEC.apply(list())`.

use crate::advancements::criterion::block_predicate::BlockPredicate;
use crate::network::chat::component::Component;
use crate::network::chat::chat_formatting::ChatFormatting;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::level::block::state::pattern::block_in_world::BlockInWorld;
use std::sync::Mutex;

/// `AdventureModePredicate`.
#[derive(Debug)]
pub struct AdventureModePredicate {
    /// `predicates` — `List<BlockPredicate>`.
    pub predicates: Vec<BlockPredicate>,

    /// `cachedTooltip` — кэш `tooltip()`.
    cached_tooltip: Mutex<Option<Vec<Component>>>,
    /// `lastResult`/`checksBlockEntity` — флаги последнего `test`
    /// (Java mutable fields; `lastCheckedBlock`-кэш опускаем —
    /// семантика совпадает, пересчёт чуть дороже).
    last_result: Mutex<bool>,
    checks_block_entity: Mutex<bool>,
}

impl Clone for AdventureModePredicate {
    fn clone(&self) -> Self {
        Self {
            predicates: self.predicates.clone(),
            cached_tooltip: Mutex::new(self.cached_tooltip.lock().unwrap().clone()),
            last_result: Mutex::new(*self.last_result.lock().unwrap()),
            checks_block_entity: Mutex::new(*self.checks_block_entity.lock().unwrap()),
        }
    }
}

impl PartialEq for AdventureModePredicate {
    /// Java `equals` — сравнение только `predicates`.
    fn eq(&self, other: &Self) -> bool {
        self.predicates == other.predicates
    }
}

impl AdventureModePredicate {
    /// `new`.
    pub fn new(predicates: Vec<BlockPredicate>) -> Self {
        Self {
            predicates,
            cached_tooltip: Mutex::new(None),
            last_result: Mutex::new(false),
            checks_block_entity: Mutex::new(false),
        }
    }

    /// `STREAM_CODEC` — `list(BlockPredicate.STREAM_CODEC)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, AdventureModePredicate> + Send + Sync + 'static {
        let enc = byte_buf_codecs::list(BlockPredicate::stream_codec());
        let dec = byte_buf_codecs::list(BlockPredicate::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &AdventureModePredicate| {
                enc.encode(buf, &v.predicates);
            },
            move |buf: &mut FriendlyByteBuf| {
                AdventureModePredicate::new(dec.decode(buf))
            },
        )
    }

    /// `test(BlockInWorld)` — первый совпавший предикат;
    /// `requiresNbt` поднимает `checksBlockEntity`.
    pub fn test(&self, block: &BlockInWorld) -> bool {
        for pred in &self.predicates {
            if pred.matches_block_in_world(block) {
                if pred.requires_nbt() {
                    *self.checks_block_entity.lock().unwrap() = true;
                }
                *self.last_result.lock().unwrap() = true;
                return true;
            }
        }
        *self.last_result.lock().unwrap() = false;
        false
    }

    /// `addToTooltip` — список имён блоков или UNKNOWN_USE.
    pub fn add_to_tooltip(&self, mut adder: impl FnMut(Component)) {
        for c in self.tooltip() {
            adder(c);
        }
    }

    /// `tooltip()` — `computeTooltip` с кэшем.
    fn tooltip(&self) -> Vec<Component> {
        let mut cache = self.cached_tooltip.lock().unwrap();
        if cache.is_none() {
            *cache = Some(Self::compute_tooltip(&self.predicates));
        }
        cache.clone().unwrap_or_default()
    }

    /// `computeTooltip` — если у какого-то предиката пустой
    /// `blocks` → UNKNOWN_USE; иначе distinct имена блоков
    /// (`DARK_GRAY` стиль).
    fn compute_tooltip(predicates: &[BlockPredicate]) -> Vec<Component> {
        for p in predicates {
            if p.blocks.as_ref().is_some_and(|s| s.is_empty()) {
                let mut c = Component::translatable("item.canUse.unknown", Vec::new());
                c.with_style(crate::network::chat::style::Style::empty()
                    .with_color_formatting(ChatFormatting::Gray));
                return vec![c];
            }
        }
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for p in predicates {
            if let Some(set) = &p.blocks {
                for holder in set.iter() {
                    let name = holder_block_name(holder);
                    if seen.insert(name.clone()) {
                        let mut c = Component::literal(name);
                        c.with_style(crate::network::chat::style::Style::empty()
                            .with_color_formatting(ChatFormatting::DarkGray));
                        out.push(c);
                    }
                }
            }
        }
        out
    }
}

fn holder_block_name(holder: &crate::core::holder::Holder<crate::world::level::block::Block>) -> String {
    use crate::core::holder::Holder;
    match holder {
        Holder::Direct(b) => b.id.to_string(),
        Holder::Key(k) => k.id().to_string(),
        Holder::Reference(id) => crate::world::level::block::block_by_id(*id)
            .map(|b| b.id.to_string())
            .unwrap_or_else(|| format!("#{id}")),
    }
}
