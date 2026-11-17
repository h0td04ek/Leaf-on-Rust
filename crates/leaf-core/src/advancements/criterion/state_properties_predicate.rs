//! Перенос `net.minecraft.advancements.criterion.StatePropertiesPredicate`
//! — предикат значений свойств состояния блока/жидкости.
//!
//! `STREAM_CODEC` — `list(PropertyMatcher)`: имя свойства (UTF8) +
//! `ValueMatcher` (`either(ExactMatcher UTF8, RangedMatcher
//! optional(UTF8)+optional(UTF8))`).

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::fluid_state::FluidState;
use crate::world::level::block::state::property::{PropertyTrait, StateValue};
use std::sync::Arc;

/// `StatePropertiesPredicate.ValueMatcher` — точное или
/// диапазонное совпадение.
#[derive(Clone, Debug, PartialEq)]
pub enum ValueMatcher {
    /// `ExactMatcher(value)`.
    Exact(String),
    /// `RangedMatcher(min?, max?)`.
    Ranged { min: Option<String>, max: Option<String> },
}

impl ValueMatcher {
    /// `match(stateHolder, property)` — сравнение через
    /// сериализованные значения (эквивалент `Comparable` для
    /// Int/Bool/Str).
    pub fn matches(&self, value: Option<&StateValue>, property: &dyn PropertyTrait) -> bool {
        let Some(stored) = value else { return false };
        match self {
            ValueMatcher::Exact(expected) => {
                property
                    .parse(expected)
                    .is_some_and(|v| &v == stored)
            }
            ValueMatcher::Ranged { min, max } => {
                if let Some(min_s) = min {
                    let Some(min_v) = property.parse(min_s) else { return false };
                    if stored < &min_v {
                        return false;
                    }
                }
                if let Some(max_s) = max {
                    let Some(max_v) = property.parse(max_s) else { return false };
                    if stored > &max_v {
                        return false;
                    }
                }
                true
            }
        }
    }
}

/// `PropertyMatcher(name, valueMatcher)`.
#[derive(Clone, Debug, PartialEq)]
pub struct PropertyMatcher {
    /// `name` — `Property.getName`.
    pub name: String,
    /// `valueMatcher`.
    pub value_matcher: ValueMatcher,
}

impl PropertyMatcher {
    /// `match(StateDefinition, state)` — свойство существует и
    /// значение совпадает.
    pub fn matches_block_state(&self, state: &BlockState) -> bool {
        match state.definition.get_property(&self.name) {
            Some(prop) => self.value_matcher.matches(state.values.get(prop.as_ref()).as_ref(), prop.as_ref()),
            None => false,
        }
    }

    /// `match` для `FluidState`.
    pub fn matches_fluid_state(&self, state: &FluidState) -> bool {
        match state.fluid.state_definition.as_ref().and_then(|d| d.get_property(&self.name)) {
            Some(prop) => self.value_matcher.matches(state.values.get(prop.as_ref()).as_ref(), prop.as_ref()),
            None => false,
        }
    }

    /// `checkState(StateDefinition)` — `Some(name)` если свойства
    /// нет в определении.
    pub fn check_state(&self, definition: &crate::world::level::block::state::state_definition::StateDefinition) -> Option<String> {
        definition.get_property(&self.name).is_none().then(|| self.name.clone())
    }
}

/// `StatePropertiesPredicate` — список `PropertyMatcher`.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct StatePropertiesPredicate {
    /// `properties`.
    pub properties: Vec<PropertyMatcher>,
}

impl StatePropertiesPredicate {
    /// `new`.
    pub fn new(properties: Vec<PropertyMatcher>) -> Self {
        Self { properties }
    }

    /// `matches(BlockState)`.
    pub fn matches_block_state(&self, state: &BlockState) -> bool {
        self.properties.iter().all(|p| p.matches_block_state(state))
    }

    /// `matches(FluidState)`.
    pub fn matches_fluid_state(&self, state: &FluidState) -> bool {
        self.properties.iter().all(|p| p.matches_fluid_state(state))
    }

    /// `checkState` — первая отсутствующая property.
    pub fn check_state(&self, definition: &crate::world::level::block::state::state_definition::StateDefinition) -> Option<String> {
        self.properties.iter().find_map(|p| p.check_state(definition))
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, StatePropertiesPredicate> + Send + Sync + 'static {
        let enc = byte_buf_codecs::list(property_matcher_stream_codec());
        let dec = byte_buf_codecs::list(property_matcher_stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &StatePropertiesPredicate| {
                enc.encode(buf, &v.properties);
            },
            move |buf: &mut FriendlyByteBuf| StatePropertiesPredicate {
                properties: dec.decode(buf),
            },
        )
    }
}

/// `ValueMatcher.STREAM_CODEC` — `either(ExactMatcher, RangedMatcher)`.
fn value_matcher_stream_codec() -> impl StreamCodec<FriendlyByteBuf, ValueMatcher> + Send + Sync + 'static {
    let base = byte_buf_codecs::either(byte_buf_codecs::STRING_UTF8, ranged_matcher_stream_codec());
    crate::network::codec::stream_codec::map(
        |e: &crate::util::either::Either<String, (Option<String>, Option<String>)>| match e {
            crate::util::either::Either::Left(s) => ValueMatcher::Exact(s.clone()),
            crate::util::either::Either::Right((min, max)) => ValueMatcher::Ranged {
                min: min.clone(),
                max: max.clone(),
            },
        },
        |v: &ValueMatcher| match v {
            ValueMatcher::Exact(s) => crate::util::either::Either::Left(s.clone()),
            ValueMatcher::Ranged { min, max } => {
                crate::util::either::Either::Right((min.clone(), max.clone()))
            }
        },
        base,
    )
}

fn ranged_matcher_stream_codec() -> impl StreamCodec<FriendlyByteBuf, (Option<String>, Option<String>)> + Send + Sync + 'static {
    let opt_enc = byte_buf_codecs::optional(byte_buf_codecs::STRING_UTF8);
    let opt_dec = byte_buf_codecs::optional(byte_buf_codecs::STRING_UTF8);
    sync_fn(
        move |buf: &mut FriendlyByteBuf, v: &(Option<String>, Option<String>)| {
            opt_enc.encode(buf, &v.0);
            opt_enc.encode(buf, &v.1);
        },
        move |buf: &mut FriendlyByteBuf| (opt_dec.decode(buf), opt_dec.decode(buf)),
    )
}

/// `PropertyMatcher.STREAM_CODEC` — `composite(STRING_UTF8,
/// ValueMatcher.STREAM_CODEC)`.
fn property_matcher_stream_codec() -> impl StreamCodec<FriendlyByteBuf, PropertyMatcher> + Send + Sync + 'static {
    let name_enc = byte_buf_codecs::STRING_UTF8;
    let name_dec = byte_buf_codecs::STRING_UTF8;
    let vm_enc = value_matcher_stream_codec();
    let vm_dec = value_matcher_stream_codec();
    sync_fn(
        move |buf: &mut FriendlyByteBuf, m: &PropertyMatcher| {
            name_enc.encode(buf, &m.name);
            vm_enc.encode(buf, &m.value_matcher);
        },
        move |buf: &mut FriendlyByteBuf| PropertyMatcher {
            name: name_dec.decode(buf),
            value_matcher: vm_dec.decode(buf),
        },
    )
}
