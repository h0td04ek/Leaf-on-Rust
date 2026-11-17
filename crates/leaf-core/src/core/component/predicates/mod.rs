//! Перенос `net.minecraft.core.component.predicates` —
//! `DataComponentPredicate` и реестр `DATA_COMPONENT_PREDICATE_TYPE`.
//!
//! Stream-кодек значения каждого типа — `ByteBufCodecs
//! .fromCodecWithRegistries(codec)` → значение сериализуется в NBT
//! (`Tag`) и пишется `tagCodec` (см. `nbt_codec`-функции типов).
//!
//! `Type.STREAM_CODEC` = `either(registry(PREDICATE_TYPE),
//! registry(DATA_COMPONENT_TYPE))`: bool + VarInt id;
//! left — ConcreteType (id 0..13 в порядке `DataComponentPredicates`),
//! right — `AnyValueType` (id типа data component).

pub mod types;

use crate::core::component::data_component_exact_predicate::DataComponentExactPredicate;
use crate::core::component::data_component_getter::DataComponentGetter;
use crate::core::component::data_component_type::{
    data_component_type_id, DataComponentType,
};
use crate::nbt::tag::Tag;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use std::sync::Arc;

pub use types::*;

/// `DataComponentPredicate.Type` — ссылка на тип предиката:
/// зарегистрированный `ConcreteType` или динамический `AnyValueType`.
#[derive(Clone, Debug, PartialEq)]
pub enum PredicateType {
    /// `ConcreteType` — зарегистрированный тип (id в
    /// `DATA_COMPONENT_PREDICATE_TYPE`).
    Concrete(u8),
    /// `AnyValueType` — любой `DataComponentType` (проверка
    /// присутствия/совпадения через `AnyValue`).
    AnyValue(Arc<DataComponentType>),
}

/// `DataComponentPredicate.Single` — (type, predicate).
#[derive(Clone, Debug, PartialEq)]
pub struct Single {
    /// `type`.
    pub type_: PredicateType,
    /// `predicate`.
    pub predicate: DataComponentPredicate,
}

/// Порядок регистрации `DataComponentPredicates` — wire-id.
pub const CONCRETE_TYPE_COUNT: u8 = 14;

/// id ConcreteType по типу предиката.
pub fn concrete_type_id(p: &DataComponentPredicate) -> u8 {
    match p {
        DataComponentPredicate::Damage(_) => 0,
        DataComponentPredicate::Enchantments(_) => 1,
        DataComponentPredicate::StoredEnchantments(_) => 2,
        DataComponentPredicate::Potions(_) => 3,
        DataComponentPredicate::CustomData(_) => 4,
        DataComponentPredicate::Container(_) => 5,
        DataComponentPredicate::Bundle(_) => 6,
        DataComponentPredicate::FireworkExplosion(_) => 7,
        DataComponentPredicate::Fireworks(_) => 8,
        DataComponentPredicate::WritableBook(_) => 9,
        DataComponentPredicate::WrittenBook(_) => 10,
        DataComponentPredicate::AttributeModifiers(_) => 11,
        DataComponentPredicate::Trim(_) => 12,
        DataComponentPredicate::JukeboxPlayable(_) => 13,
        DataComponentPredicate::AnyValue(_) => unreachable!(),
    }
}

/// ConcreteType по wire-id.
pub fn concrete_type_by_id(id: u8) -> Option<fn(&Tag) -> Result<DataComponentPredicate, String>> {
    Some(match id {
        0 => |t| types::DamagePredicate::decode_nbt(t).map(DataComponentPredicate::Damage),
        1 => |t| types::EnchantmentsPredicate::decode_enchantments(t).map(DataComponentPredicate::Enchantments),
        2 => |t| types::EnchantmentsPredicate::decode_stored(t).map(DataComponentPredicate::StoredEnchantments),
        3 => |t| types::PotionsPredicate::decode_nbt(t).map(DataComponentPredicate::Potions),
        4 => |t| types::CustomDataPredicate::decode_nbt(t).map(DataComponentPredicate::CustomData),
        5 => |t| types::ContainerPredicate::decode_nbt(t).map(DataComponentPredicate::Container),
        6 => |t| types::BundlePredicate::decode_nbt(t).map(DataComponentPredicate::Bundle),
        7 => |t| types::FireworkExplosionPredicate::decode_nbt(t).map(DataComponentPredicate::FireworkExplosion),
        8 => |t| types::FireworksPredicate::decode_nbt(t).map(DataComponentPredicate::Fireworks),
        9 => |t| types::WritableBookPredicate::decode_nbt(t).map(DataComponentPredicate::WritableBook),
        10 => |t| types::WrittenBookPredicate::decode_nbt(t).map(DataComponentPredicate::WrittenBook),
        11 => |t| types::AttributeModifiersPredicate::decode_nbt(t).map(DataComponentPredicate::AttributeModifiers),
        12 => |t| types::TrimPredicate::decode_nbt(t).map(DataComponentPredicate::Trim),
        13 => |t| types::JukeboxPlayablePredicate::decode_nbt(t).map(DataComponentPredicate::JukeboxPlayable),
        _ => return None,
    })
}

/// `DataComponentPredicate` — сумма типов.
#[derive(Clone, Debug, PartialEq)]
pub enum DataComponentPredicate {
    /// `AnyValue` — presence/exact для произвольного типа.
    AnyValue(AnyValue),
    /// `damage`.
    Damage(types::DamagePredicate),
    /// `enchantments`.
    Enchantments(types::EnchantmentsPredicate),
    /// `stored_enchantments`.
    StoredEnchantments(types::EnchantmentsPredicate),
    /// `potion_contents`.
    Potions(types::PotionsPredicate),
    /// `custom_data`.
    CustomData(types::CustomDataPredicate),
    /// `container`.
    Container(types::ContainerPredicate),
    /// `bundle_contents`.
    Bundle(types::BundlePredicate),
    /// `firework_explosion`.
    FireworkExplosion(types::FireworkExplosionPredicate),
    /// `fireworks`.
    Fireworks(types::FireworksPredicate),
    /// `writable_book_content`.
    WritableBook(types::WritableBookPredicate),
    /// `written_book_content`.
    WrittenBook(types::WrittenBookPredicate),
    /// `attribute_modifiers`.
    AttributeModifiers(types::AttributeModifiersPredicate),
    /// `trim`.
    Trim(types::TrimPredicate),
    /// `jukebox_playable`.
    JukeboxPlayable(types::JukeboxPlayablePredicate),
}

impl DataComponentPredicate {
    /// `matches(DataComponentGetter)` — dispatch по типу.
    /// Требует `DataComponentType` констант — резолвятся через
    /// `data_components` по id (`ensure_bootstrapped`).
    pub fn matches(&self, getter: &dyn DataComponentGetter) -> bool {
        use crate::core::component::data_components as dc;
        dc::ensure_bootstrapped();
        let ty = |name: &str| {
            crate::core::component::data_component_type::data_component_type_by_identifier(
                &crate::resources::identifier::Identifier::parse(name).unwrap(),
            )
            .expect("data component type not registered")
        };
        match self {
            DataComponentPredicate::AnyValue(a) => a.matches(getter),
            DataComponentPredicate::Damage(p) => p.matches(getter),
            DataComponentPredicate::Enchantments(p) => p.matches(getter, ty("enchantments")),
            DataComponentPredicate::StoredEnchantments(p) => p.matches(getter, ty("stored_enchantments")),
            DataComponentPredicate::Potions(p) => p.matches(getter, ty("potion_contents")),
            DataComponentPredicate::CustomData(p) => p.matches(getter),
            DataComponentPredicate::Container(p) => p.matches(getter, ty("container")),
            DataComponentPredicate::Bundle(p) => p.matches(getter, ty("bundle_contents")),
            DataComponentPredicate::FireworkExplosion(p) => p.matches(getter, ty("firework_explosion")),
            DataComponentPredicate::Fireworks(p) => p.matches(getter, ty("fireworks")),
            DataComponentPredicate::WritableBook(p) => p.matches(getter, ty("writable_book_content")),
            DataComponentPredicate::WrittenBook(p) => p.matches(getter, ty("written_book_content")),
            DataComponentPredicate::AttributeModifiers(p) => p.matches(getter, ty("attribute_modifiers")),
            DataComponentPredicate::Trim(p) => p.matches(getter, ty("trim")),
            DataComponentPredicate::JukeboxPlayable(p) => p.matches(getter, ty("jukebox_playable")),
        }
    }

    /// NBT-кодирование значения (`fromCodecWithRegistries` → `Tag`).
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        match self {
            DataComponentPredicate::AnyValue(_) => Ok(Tag::Compound(crate::nbt::compound::CompoundTag::new())),
            DataComponentPredicate::Damage(p) => p.encode_nbt(),
            DataComponentPredicate::Enchantments(p) => p.encode_enchantments_nbt(),
            DataComponentPredicate::StoredEnchantments(p) => p.encode_stored_nbt(),
            DataComponentPredicate::Potions(p) => p.encode_nbt(),
            DataComponentPredicate::CustomData(p) => p.encode_nbt(),
            DataComponentPredicate::Container(p) => p.encode_nbt(),
            DataComponentPredicate::Bundle(p) => p.encode_nbt(),
            DataComponentPredicate::FireworkExplosion(p) => p.encode_nbt(),
            DataComponentPredicate::Fireworks(p) => p.encode_nbt(),
            DataComponentPredicate::WritableBook(p) => p.encode_nbt(),
            DataComponentPredicate::WrittenBook(p) => p.encode_nbt(),
            DataComponentPredicate::AttributeModifiers(p) => p.encode_nbt(),
            DataComponentPredicate::Trim(p) => p.encode_nbt(),
            DataComponentPredicate::JukeboxPlayable(p) => p.encode_nbt(),
        }
    }

    /// NBT-декодирование значения по типу.
    pub fn decode_nbt(type_: &PredicateType, tag: &Tag) -> Result<Self, String> {
        match type_ {
            PredicateType::Concrete(id) => {
                let decode = concrete_type_by_id(*id)
                    .ok_or_else(|| format!("unknown predicate type id {id}"))?;
                decode(tag)
            }
            PredicateType::AnyValue(_) => Ok(DataComponentPredicate::AnyValue(AnyValue {
                type_: match type_ {
                    PredicateType::AnyValue(t) => t.clone(),
                    _ => unreachable!(),
                },
            })),
        }
    }
}

/// `DataComponentPredicate.STREAM_CODEC` —
/// `list(64, Single)` → `Map<Type, Predicate>`; у нас —
/// `Vec<Single>` (порядок сохраняется, wire идентичен).
pub fn predicates_stream_codec() -> impl StreamCodec<FriendlyByteBuf, Vec<Single>> + Send + Sync + 'static {
    let list_enc = byte_buf_codecs::list_limited(64, single_stream_codec());
    let list_dec = byte_buf_codecs::list_limited(64, single_stream_codec());
    sync_fn(
        move |buf: &mut FriendlyByteBuf, v: &Vec<Single>| {
            list_enc.encode(buf, v);
        },
        move |buf: &mut FriendlyByteBuf| list_dec.decode(buf),
    )
}

/// `SINGLE_STREAM_CODEC` — Type + значение(NBT).
fn single_stream_codec() -> impl StreamCodec<FriendlyByteBuf, Single> + Send + Sync + 'static {
    let type_enc = predicate_type_stream_codec();
    let type_dec = predicate_type_stream_codec();
    sync_fn(
        move |buf: &mut FriendlyByteBuf, s: &Single| {
            type_enc.encode(buf, &s.type_);
            let tag = s.predicate.encode_nbt().expect("predicate nbt encode");
            buf.write_nbt(Some(&tag)).expect("predicate nbt write");
        },
        move |buf: &mut FriendlyByteBuf| {
            let type_ = type_dec.decode(buf);
            let tag = buf.read_nbt().expect("predicate nbt").unwrap_or(Tag::End);
            let predicate = DataComponentPredicate::decode_nbt(&type_, &tag)
                .expect("predicate decode");
            Single { type_, predicate }
        },
    )
}

/// `Type.STREAM_CODEC` — `either(registry(PREDICATE_TYPE),
/// registry(DATA_COMPONENT_TYPE))`.
/// left: bool true + VarInt concrete-id (0..13);
/// right: bool false + VarInt component-type-id.
pub fn predicate_type_stream_codec() -> impl StreamCodec<FriendlyByteBuf, PredicateType> + Send + Sync + 'static {
    sync_fn(
        |buf: &mut FriendlyByteBuf, t: &PredicateType| match t {
            PredicateType::Concrete(id) => {
                buf.write_bool(true);
                buf.write_var_int(*id as i32);
            }
            PredicateType::AnyValue(component_type) => {
                buf.write_bool(false);
                let id = data_component_type_id(component_type)
                    .expect("unregistered component type in AnyValue predicate");
                buf.write_var_int(id);
            }
        },
        |buf: &mut FriendlyByteBuf| {
            if buf.read_bool() {
                let id = buf.read_var_int();
                assert!((0..CONCRETE_TYPE_COUNT as i32).contains(&id), "bad predicate type id {id}");
                PredicateType::Concrete(id as u8)
            } else {
                let id = buf.read_var_int();
                let component_type = crate::core::component::data_component_type::data_component_type_arc_by_id(id)
                    .expect("unknown component type id in predicate");
                PredicateType::AnyValue(component_type)
            }
        },
    )
}

/// `DataComponentExactPredicate` реэкспорт для удобства.
pub use crate::core::component::data_component_exact_predicate::DataComponentExactPredicate as ExactPredicate;
