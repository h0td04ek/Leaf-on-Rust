//! Перенос `net.minecraft.advancements.criterion.DataComponentMatchers`
//! — `exact: DataComponentExactPredicate` +
//! `partial: Map<DataComponentPredicate.Type, DataComponentPredicate>`.

use crate::advancements::criterion::nbt_util;
use crate::core::component::data_component_exact_predicate::DataComponentExactPredicate;
use crate::core::component::data_component_getter::DataComponentGetter;
use crate::core::component::predicates::{predicates_stream_codec, DataComponentPredicate, Single};
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `DataComponentMatchers` — точные + частичные предикаты
/// компонентов. `partial` хранится упорядоченным списком
/// `Single` (wire-формат — `list(64, SINGLE)` → map; дубликаты
/// типов на encode недопустимы — при decode сохраняем порядок
/// и дедуплицируем по ключу типа, как `Map` в Java).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DataComponentMatchers {
    /// `exact`.
    pub exact: DataComponentExactPredicate,
    /// `partial` — список (type, predicate).
    pub partial: Vec<Single>,
}

impl DataComponentMatchers {
    /// `ANY`.
    pub const ANY: DataComponentMatchers = DataComponentMatchers {
        exact: DataComponentExactPredicate::EMPTY,
        partial: Vec::new(),
    };

    /// `new`.
    pub fn new(exact: DataComponentExactPredicate, partial: Vec<Single>) -> Self {
        Self { exact, partial }
    }

    /// `test(DataComponentGetter)`.
    pub fn test(&self, getter: &dyn DataComponentGetter) -> bool {
        if !self.exact.test(getter) {
            return false;
        }
        self.partial.iter().all(|s| s.predicate.matches(getter))
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.exact.is_empty() && self.partial.is_empty()
    }

    /// `components()` — builder.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// `STREAM_CODEC` — composite(ExactPredicate.STREAM_CODEC,
    /// Predicate.STREAM_CODEC(map)).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, DataComponentMatchers> + Send + Sync + 'static {
        let exact_enc = DataComponentExactPredicate::stream_codec();
        let exact_dec = DataComponentExactPredicate::stream_codec();
        let partial_enc = predicates_stream_codec();
        let partial_dec = predicates_stream_codec();
        sync_fn(
            move |buf: &mut FriendlyByteBuf, m: &DataComponentMatchers| {
                exact_enc.encode(buf, &m.exact);
                partial_enc.encode(buf, &m.partial);
            },
            move |buf: &mut FriendlyByteBuf| {
                let exact = exact_dec.decode(buf);
                let partial = dedup_partial(partial_dec.decode(buf));
                DataComponentMatchers { exact, partial }
            },
        )
    }

    /// NBT `CODEC` → compound `{components?, predicates?}`.
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        let mut c = CompoundTag::new();
        if !self.exact.is_empty() {
            c.put("components", self.exact.encode_nbt()?);
        }
        if !self.partial.is_empty() {
            let mut p = CompoundTag::new();
            for single in &self.partial {
                let key = predicate_type_nbt_key(&single.type_);
                p.put(&key, single.predicate.encode_nbt()?);
            }
            c.put("predicates", Tag::Compound(p));
        }
        Ok(Tag::Compound(c))
    }

    /// NBT `CODEC` ← compound.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = nbt_util::as_compound(tag, "component matchers")?;
        let exact = match c.get("components") {
            Some(t) => DataComponentExactPredicate::decode_nbt(t)?,
            None => DataComponentExactPredicate::EMPTY,
        };
        let mut partial = Vec::new();
        if let Some(Tag::Compound(p)) = c.get("predicates") {
            for (key, value) in p.iter() {
                let type_ = predicate_type_from_nbt_key(key)?;
                let predicate = DataComponentPredicate::decode_nbt(&type_, value)?;
                partial.push(Single { type_, predicate });
            }
        }
        Ok(Self::new(exact, dedup_partial(partial)))
    }
}

fn dedup_partial(mut partial: Vec<Single>) -> Vec<Single> {
    // Map-семантика Java: поздний тип перезаписывает ранний,
    // порядок — порядок первого вхождения ключа не важен для
    // теста (AND по всем значениям).
    let mut seen = std::collections::HashSet::new();
    partial.retain(|s| seen.insert(predicate_type_key(&s.type_)));
    partial
}

fn predicate_type_key(t: &crate::core::component::predicates::PredicateType) -> String {
    match t {
        crate::core::component::predicates::PredicateType::Concrete(id) => format!("concrete:{id}"),
        crate::core::component::predicates::PredicateType::AnyValue(ct) => format!("any:{}", ct.id),
    }
}

fn predicate_type_nbt_key(t: &crate::core::component::predicates::PredicateType) -> String {
    match t {
        crate::core::component::predicates::PredicateType::Concrete(id) => {
            concrete_type_name(*id).map(str::to_string).unwrap_or_else(|| format!("unknown:{id}"))
        }
        crate::core::component::predicates::PredicateType::AnyValue(ct) => ct.id.to_string(),
    }
}

fn predicate_type_from_nbt_key(key: &str) -> Result<crate::core::component::predicates::PredicateType, String> {
    use crate::core::component::predicates::PredicateType;
    // Concrete-типы — по имени ("damage", "enchantments", ...).
    if let Some(id) = concrete_type_id_by_name(key) {
        return Ok(PredicateType::Concrete(id));
    }
    // AnyValue — identifier компонента.
    let id = crate::resources::identifier::Identifier::parse(key).map_err(|e| e.to_string())?;
    let ct = crate::core::component::data_component_type::data_component_type_by_identifier(&id)
        .ok_or_else(|| format!("unknown component type '{key}'"))?;
    Ok(PredicateType::AnyValue(std::sync::Arc::new(ct)))
}

fn concrete_type_id_by_name(name: &str) -> Option<u8> {
    Some(match name {
        "damage" => 0,
        "enchantments" => 1,
        "stored_enchantments" => 2,
        "potion_contents" => 3,
        "custom_data" => 4,
        "container" => 5,
        "bundle_contents" => 6,
        "firework_explosion" => 7,
        "fireworks" => 8,
        "writable_book_content" => 9,
        "written_book_content" => 10,
        "attribute_modifiers" => 11,
        "trim" => 12,
        "jukebox_playable" => 13,
        _ => return None,
    })
}

fn concrete_type_name(id: u8) -> Option<&'static str> {
    Some(match id {
        0 => "damage",
        1 => "enchantments",
        2 => "stored_enchantments",
        3 => "potion_contents",
        4 => "custom_data",
        5 => "container",
        6 => "bundle_contents",
        7 => "firework_explosion",
        8 => "fireworks",
        9 => "writable_book_content",
        10 => "written_book_content",
        11 => "attribute_modifiers",
        12 => "trim",
        13 => "jukebox_playable",
        _ => return None,
    })
}

/// `DataComponentMatchers.Builder`.
#[derive(Default)]
pub struct Builder {
    exact: Option<DataComponentExactPredicate>,
    partial: Vec<Single>,
}

impl Builder {
    /// `any(componentType)` — `AnyValueType` предикат.
    pub fn any(mut self, component_type: std::sync::Arc<crate::core::component::data_component_type::DataComponentType>) -> Self {
        let predicate = crate::core::component::predicates::DataComponentPredicate::AnyValue(
            crate::core::component::predicates::AnyValue { type_: component_type.clone() },
        );
        let type_ = crate::core::component::predicates::PredicateType::AnyValue(component_type);
        self.partial.push(Single { type_, predicate });
        self
    }

    /// `partial(type, value)` — конкретный предикат.
    pub fn partial(mut self, type_: crate::core::component::predicates::PredicateType, value: DataComponentPredicate) -> Self {
        self.partial.push(Single { type_, predicate: value });
        self
    }

    /// `exact(exact)`.
    pub fn exact(mut self, exact: DataComponentExactPredicate) -> Self {
        self.exact = Some(exact);
        self
    }

    /// `build`.
    pub fn build(self) -> DataComponentMatchers {
        DataComponentMatchers::new(
            self.exact.unwrap_or(DataComponentExactPredicate::EMPTY),
            dedup_partial(self.partial),
        )
    }
}
