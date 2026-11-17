//! Типы `DataComponentPredicate` — структуры и их NBT-кодеки
//! (`Codec` → NBT `Tag`, как `fromCodecWithRegistries`).

use crate::advancements::criterion::collection_predicate::{
    CollectionPredicate, CollectionTest,
};
use crate::advancements::criterion::enchantment_predicate::EnchantmentPredicate;
use crate::advancements::criterion::holder_set_nbt;
use crate::advancements::criterion::item_predicate::ItemPredicate;
use crate::advancements::criterion::min_max_bounds::{MinMaxBoundsDoubles, MinMaxBoundsInts};
use crate::advancements::criterion::nbt_predicate::NbtPredicate;
use crate::advancements::criterion::nbt_util;
use crate::core::component::data_component_getter::DataComponentGetter;
use crate::core::component::data_component_type::{
    data_component_type_by_identifier, DataComponentType,
};
use crate::core::component::predicates::PredicateType;
use crate::core::holder::{Holder, HolderKind};
use crate::core::holder_set::HolderSet;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::network::chat::component::Component;
use crate::network::chat::component_json::{component_from_json, component_to_json};
use crate::resources::identifier::Identifier;
use crate::server::network::filterable::Filterable;
use crate::entity::ai::attributes::attribute::Attribute;
use crate::entity::ai::attributes::attribute_modifier::{
    AttributeModifier, AttributeModifierOperation,
};
use crate::entity::equipment_slot_group::EquipmentSlotGroup;
use crate::world::item::alchemy::potion::Potion;
use crate::world::item::component::firework_explosion::{FireworkExplosion, FireworkExplosionShape};
use crate::world::item::component::fireworks::Fireworks;
use crate::world::item::component::item_attribute_modifiers::{
    ItemAttributeModifiers, ItemAttributeModifiersDisplay, ItemAttributeModifiersEntry,
};
use crate::world::item::component::potion_contents::PotionContents;
use crate::world::item::component::typed_entity_data::TypedEntityData;
use crate::world::item::equipment::trim::armor_trim::ArmorTrim;
use crate::world::item::equipment::trim::trim_material::TrimMaterial;
use crate::world::item::equipment::trim::trim_pattern::TrimPattern;
use crate::world::item::item_stack::ItemStack;
use crate::world::item::jukebox_song::JukeboxSong;
use crate::world::item::component::jukebox_playable::JukeboxPlayable;
use std::sync::Arc;

/// Резолв `DataComponentType` по имени (после bootstrap).
fn component_type(name: &str) -> Option<DataComponentType> {
    data_component_type_by_identifier(
        &Identifier::parse(&format!("minecraft:{name}")).ok()?,
    )
}

/// Type-erased `getter.get` → owned `T` (клон значения).
fn component_value<T: Clone + 'static>(
    getter: &dyn DataComponentGetter,
    ty: &DataComponentType,
) -> Option<T> {
    getter
        .get_raw(ty)
        .and_then(|v| v.as_any().downcast_ref::<T>().cloned())
}

// ------------------------------------------------------------ AnyValue

/// `AnyValue` — предикат присутствия компонента любого значения.
#[derive(Clone, Debug, PartialEq)]
pub struct AnyValue {
    /// `type`.
    pub type_: Arc<DataComponentType>,
}

impl AnyValue {
    /// `matches` — `componentGetter.get(type) != null`.
    pub fn matches(&self, getter: &dyn DataComponentGetter) -> bool {
        // get::<T> требует конкретный T; presence-проверка по id —
        // используем has через get с dyn? DataComponentGetter не
        // имеет has — проверяем через get::<ComponentValue> нельзя.
        // Добавляем presence через downcast-нейтральный путь:
        getter.has_component(&self.type_)
    }
}

// ------------------------------------------------------------ Damage

/// `DamagePredicate` — (durability?, damage?).
#[derive(Clone, Debug, PartialEq)]
pub struct DamagePredicate {
    /// `durability`.
    pub durability: MinMaxBoundsInts,
    /// `damage`.
    pub damage: MinMaxBoundsInts,
}

impl DamagePredicate {
    /// `new`.
    pub fn new(durability: MinMaxBoundsInts, damage: MinMaxBoundsInts) -> Self {
        Self { durability, damage }
    }

    /// `DamagePredicate.CODEC` → NBT.
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        let mut c = CompoundTag::new();
        if self.durability != MinMaxBoundsInts::ANY {
            c.put("durability", self.durability.encode());
        }
        if self.damage != MinMaxBoundsInts::ANY {
            c.put("damage", self.damage.encode());
        }
        Ok(Tag::Compound(c))
    }

    /// `DamagePredicate.CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = nbt_util::as_compound(tag, "damage predicate")?;
        let durability = match c.get("durability") {
            Some(t) => MinMaxBoundsInts::decode(t)?,
            None => MinMaxBoundsInts::ANY,
        };
        let damage = match c.get("damage") {
            Some(t) => MinMaxBoundsInts::decode(t)?,
            None => MinMaxBoundsInts::ANY,
        };
        Ok(Self::new(durability, damage))
    }

    /// `matches` — `damage`/`max_damage` компоненты (int).
    pub fn matches(&self, getter: &dyn DataComponentGetter) -> bool {
        let damage_ty = match component_type("damage") {
            Some(t) => t,
            None => return false,
        };
        let damage: Option<i32> = component_value(getter, &damage_ty);
        let Some(damage) = damage else { return false };
        let max_ty = component_type("max_damage");
        let max_damage: i32 = max_ty
            .and_then(|t| component_value::<i32>(getter, &t))
            .unwrap_or(0);
        self.durability.matches(max_damage - damage) && self.damage.matches(damage)
    }
}

// ------------------------------------------------------------ Enchantments

/// `EnchantmentsPredicate` — `List<EnchantmentPredicate>`
/// (enchantments | stored_enchantments — одна структура).
#[derive(Clone, Debug, PartialEq)]
pub struct EnchantmentsPredicate {
    /// `enchantments`.
    pub enchantments: Vec<EnchantmentPredicate>,
}

impl EnchantmentsPredicate {
    /// `CODEC` → NBT: `List<EnchantmentPredicate.CODEC>` —
    /// компактный список (как `Codec.listOf` через
    /// `ExtraCodecs.compactListCodec`? В Java — `listOf`;
    /// single-element коллапс не применяется).
    fn encode_list_nbt(&self) -> Result<Tag, String> {
        Ok(nbt_util::encode_list(&self.enchantments, |p| p.encode_nbt()))
    }

    fn decode_list_nbt(tag: &Tag) -> Result<Self, String> {
        let items = nbt_util::get_list(tag, "enchantments predicate")?;
        let mut out = Vec::with_capacity(items.len());
        for t in items {
            out.push(EnchantmentPredicate::decode_nbt(t)?);
        }
        Ok(Self { enchantments: out })
    }

    /// `encode_enchantments_nbt` (enchantments).
    pub fn encode_enchantments_nbt(&self) -> Result<Tag, String> {
        self.encode_list_nbt()
    }

    /// `decode_enchantments` (enchantments).
    pub fn decode_enchantments(tag: &Tag) -> Result<Self, String> {
        Self::decode_list_nbt(tag)
    }

    /// `encode_stored_nbt` (stored_enchantments — тот же CODEC).
    pub fn encode_stored_nbt(&self) -> Result<Tag, String> {
        self.encode_list_nbt()
    }

    /// `decode_stored`.
    pub fn decode_stored(tag: &Tag) -> Result<Self, String> {
        Self::decode_list_nbt(tag)
    }

    /// `matches` — enchantments компонента.
    pub fn matches(&self, getter: &dyn DataComponentGetter, component: DataComponentType) -> bool {
        let Some(enchantments) = component_value::<crate::world::item::enchantment::item_enchantments::ItemEnchantments>(getter, &component) else {
            return false;
        };
        enchantments.enchantments.iter().all(|_| true) && {
            self.enchantments.iter().all(|p| p.contained_in(&enchantments.enchantments))
        }
    }
}

// ------------------------------------------------------------ Potions

/// `PotionsPredicate` — `HolderSet<Potion>`.
#[derive(Clone, Debug, PartialEq)]
pub struct PotionsPredicate {
    /// `potions`.
    pub potions: HolderSet<Potion>,
}

impl PotionsPredicate {
    /// `CODEC` → NBT: `homogeneousList(POTION)` —
    /// either(tag, compactList).
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        holder_set_nbt::holder_set_encode(
            &self.potions,
            &Identifier::minecraft("potion"),
            |_| None,
        )
    }

    /// `CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let potions = holder_set_nbt::holder_set_decode::<Potion>(
            tag,
            Identifier::minecraft("potion"),
        )?;
        Ok(Self { potions })
    }

    /// `matches` — `PotionContents.potion` ∈ set.
    pub fn matches(&self, getter: &dyn DataComponentGetter, component: DataComponentType) -> bool {
        let Some(contents) = component_value::<PotionContents>(getter, &component) else { return false };
        let Some(holder) = &contents.potion else { return false };
        holder_set_nbt::holder_set_contains(&self.potions, holder)
            || matches!(self.potions, HolderSet::Named(_))
    }
}

// ------------------------------------------------------------ CustomData

/// `CustomDataPredicate` — `NbtPredicate` по `custom_data`.
#[derive(Clone, Debug, PartialEq)]
pub struct CustomDataPredicate {
    /// `value`.
    pub value: NbtPredicate,
}

impl CustomDataPredicate {
    /// `CODEC` → NBT: сам тег предиката.
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        Ok(self.value.encode_nbt())
    }

    /// `CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        Ok(Self { value: NbtPredicate::decode_nbt(tag)? })
    }

    /// `matches` — `custom_data` компонент vs NBT.
    pub fn matches(&self, getter: &dyn DataComponentGetter) -> bool {
        let ty = match component_type("custom_data") {
            Some(t) => t,
            None => return false,
        };
        let custom_data = component_value::<crate::world::item::component::custom_data::CustomData>(getter, &ty);
        match custom_data {
            Some(cd) => self.value.matches_custom_data(Some(&cd.tag)),
            None => self.value.tag.is_empty(),
        }
    }
}

// ------------------------------------------------------------ Container/Bundle

/// `ContainerPredicate` — `items?: CollectionPredicate<ItemStack,
/// ItemPredicate>`.
#[derive(Clone, Debug, PartialEq)]
pub struct ContainerPredicate {
    /// `items`.
    pub items: Option<CollectionPredicate<ItemStack, ItemPredicate>>,
}

impl ContainerPredicate {
    /// `CODEC` → NBT `{items?}`.
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        let mut c = CompoundTag::new();
        if let Some(items) = &self.items {
            c.put("items", items.encode_nbt());
        }
        Ok(Tag::Compound(c))
    }

    /// `CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = nbt_util::as_compound(tag, "container predicate")?;
        let items = match c.get("items") {
            Some(t) => Some(CollectionPredicate::<ItemStack, ItemPredicate>::decode_nbt(t)?),
            None => None,
        };
        Ok(Self { items })
    }

    /// `matches` — `ItemContainerContents.nonEmptyItems`.
    pub fn matches(&self, getter: &dyn DataComponentGetter, component: DataComponentType) -> bool {
        let Some(contents) = component_value::<crate::world::item::component::item_container_contents::ItemContainerContents>(getter, &component) else {
            return self.items.is_none();
        };
        match &self.items {
            None => true,
            Some(pred) => pred.test(contents.non_empty_items().cloned()),
        }
    }
}

/// `BundlePredicate` — `items?: CollectionPredicate<ItemStack,
/// ItemPredicate>`.
#[derive(Clone, Debug, PartialEq)]
pub struct BundlePredicate {
    /// `items`.
    pub items: Option<CollectionPredicate<ItemStack, ItemPredicate>>,
}

impl BundlePredicate {
    /// `CODEC` → NBT `{items?}`.
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        let mut c = CompoundTag::new();
        if let Some(items) = &self.items {
            c.put("items", items.encode_nbt());
        }
        Ok(Tag::Compound(c))
    }

    /// `CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = nbt_util::as_compound(tag, "bundle predicate")?;
        let items = match c.get("items") {
            Some(t) => Some(CollectionPredicate::<ItemStack, ItemPredicate>::decode_nbt(t)?),
            None => None,
        };
        Ok(Self { items })
    }

    /// `matches` — `BundleContents.items`.
    pub fn matches(&self, getter: &dyn DataComponentGetter, component: DataComponentType) -> bool {
        let Some(contents) = component_value::<crate::world::item::component::bundle_contents::BundleContents>(getter, &component) else {
            return self.items.is_none();
        };
        match &self.items {
            None => true,
            Some(pred) => pred.test(contents.items.iter().cloned()),
        }
    }
}

// ------------------------------------------------------------ FireworkExplosion

/// `FireworkExplosionPredicate.FireworkPredicate` —
/// (shape?, has_twinkle?, has_trail?).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FireworkPredicate {
    /// `shape`.
    pub shape: Option<FireworkExplosionShape>,
    /// `twinkle`.
    pub twinkle: Option<bool>,
    /// `trail`.
    pub trail: Option<bool>,
}

impl FireworkPredicate {
    /// `CODEC` → NBT.
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        let mut c = CompoundTag::new();
        if let Some(shape) = self.shape {
            c.put("shape", Tag::String(shape.get_serialized_name().to_string()));
        }
        if let Some(t) = self.twinkle {
            c.put("has_twinkle", Tag::Byte(t as i8));
        }
        if let Some(t) = self.trail {
            c.put("has_trail", Tag::Byte(t as i8));
        }
        Ok(Tag::Compound(c))
    }

    /// `CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = nbt_util::as_compound(tag, "firework predicate")?;
        let shape = match c.get("shape") {
            Some(Tag::String(s)) => Some(parse_firework_shape(s)?),
            Some(_) => return Err("shape must be string".into()),
            None => None,
        };
        let twinkle = c.get("has_twinkle").map(|t| nbt_util::get_bool(t, "has_twinkle")).transpose()?;
        let trail = c.get("has_trail").map(|t| nbt_util::get_bool(t, "has_trail")).transpose()?;
        Ok(Self { shape, twinkle, trail })
    }

    /// `test(FireworkExplosion)`.
    pub fn test(&self, v: &FireworkExplosion) -> bool {
        self.shape.map_or(true, |s| s == v.shape)
            && self.twinkle.map_or(true, |t| t == v.has_twinkle)
            && self.trail.map_or(true, |t| t == v.has_trail)
    }
}

fn parse_firework_shape(s: &str) -> Result<FireworkExplosionShape, String> {
    Ok(match s {
        "small_ball" => FireworkExplosionShape::SMALL_BALL,
        "large_ball" => FireworkExplosionShape::LARGE_BALL,
        "star" => FireworkExplosionShape::STAR,
        "creeper" => FireworkExplosionShape::CREEPER,
        "burst" => FireworkExplosionShape::BURST,
        _ => return Err(format!("unknown firework shape '{s}'")),
    })
}

impl CollectionTest<FireworkExplosion> for FireworkPredicate {
    fn test(&self, value: &FireworkExplosion) -> bool {
        FireworkPredicate::test(self, value)
    }
    fn encode_nbt(&self) -> Tag {
        FireworkPredicate::encode_nbt(self).unwrap_or(Tag::End)
    }
    fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        FireworkPredicate::decode_nbt(tag)
    }
}

/// `FireworkExplosionPredicate` — `FireworkPredicate` напрямую.
#[derive(Clone, Debug, PartialEq)]
pub struct FireworkExplosionPredicate {
    /// `predicate`.
    pub predicate: FireworkPredicate,
}

impl FireworkExplosionPredicate {
    /// `CODEC` → NBT (xmap — та же форма).
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        self.predicate.encode_nbt()
    }

    /// `CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        Ok(Self { predicate: FireworkPredicate::decode_nbt(tag)? })
    }

    /// `matches` — `firework_explosion` компонент.
    pub fn matches(&self, getter: &dyn DataComponentGetter, component: DataComponentType) -> bool {
        let Some(value) = component_value::<FireworkExplosion>(getter, &component) else { return false };
        self.predicate.test(&value)
    }
}

// ------------------------------------------------------------ Fireworks

/// `FireworksPredicate` — (explosions?, flightDuration).
#[derive(Clone, Debug, PartialEq)]
pub struct FireworksPredicate {
    /// `explosions`.
    pub explosions: Option<CollectionPredicate<FireworkExplosion, FireworkPredicate>>,
    /// `flightDuration` — `MinMaxBounds.Ints` (default ANY).
    pub flight_duration: MinMaxBoundsInts,
}

impl FireworksPredicate {
    /// `CODEC` → NBT.
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        let mut c = CompoundTag::new();
        if let Some(e) = &self.explosions {
            c.put("explosions", e.encode_nbt());
        }
        if self.flight_duration != MinMaxBoundsInts::ANY {
            c.put("flight_duration", self.flight_duration.encode());
        }
        Ok(Tag::Compound(c))
    }

    /// `CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = nbt_util::as_compound(tag, "fireworks predicate")?;
        let explosions = match c.get("explosions") {
            Some(t) => Some(CollectionPredicate::<FireworkExplosion, FireworkPredicate>::decode_nbt(t)?),
            None => None,
        };
        let flight_duration = match c.get("flight_duration") {
            Some(t) => MinMaxBoundsInts::decode(t)?,
            None => MinMaxBoundsInts::ANY,
        };
        Ok(Self { explosions, flight_duration })
    }

    /// `matches` — `fireworks` компонент.
    pub fn matches(&self, getter: &dyn DataComponentGetter, component: DataComponentType) -> bool {
        let Some(value) = component_value::<Fireworks>(getter, &component) else { return false };
        self.flight_duration.matches(value.flight_duration)
            && self.explosions.as_ref().map_or(true, |p| p.test(value.explosions.iter().cloned()))
    }
}

// ------------------------------------------------------------ Books

/// `WritableBookPredicate.PagePredicate` — `String contents`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WritableBookPagePredicate {
    /// `contents`.
    pub contents: String,
}

impl WritableBookPagePredicate {
    /// `CODEC` → NBT: строка.
    pub fn encode_nbt(&self) -> Tag {
        Tag::String(self.contents.clone())
    }

    /// `CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        Ok(Self { contents: nbt_util::get_string(tag, "page contents")? })
    }
}

impl CollectionTest<Filterable<String>> for WritableBookPagePredicate {
    fn test(&self, value: &Filterable<String>) -> bool {
        value.raw == self.contents
    }
    fn encode_nbt(&self) -> Tag {
        WritableBookPagePredicate::encode_nbt(self)
    }
    fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        WritableBookPagePredicate::decode_nbt(tag)
    }
}

/// `WritableBookPredicate` — `pages?: CollectionPredicate<
/// Filterable<String>, PagePredicate>`.
#[derive(Clone, Debug, PartialEq)]
pub struct WritableBookPredicate {
    /// `pages`.
    pub pages: Option<CollectionPredicate<Filterable<String>, WritableBookPagePredicate>>,
}

impl WritableBookPredicate {
    /// `CODEC` → NBT `{pages?}`.
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        let mut c = CompoundTag::new();
        if let Some(p) = &self.pages {
            c.put("pages", p.encode_nbt());
        }
        Ok(Tag::Compound(c))
    }

    /// `CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = nbt_util::as_compound(tag, "writable book predicate")?;
        let pages = match c.get("pages") {
            Some(t) => Some(CollectionPredicate::<Filterable<String>, WritableBookPagePredicate>::decode_nbt(t)?),
            None => None,
        };
        Ok(Self { pages })
    }

    /// `matches` — `writable_book_content`.
    pub fn matches(&self, getter: &dyn DataComponentGetter, component: DataComponentType) -> bool {
        let Some(value) = component_value::<crate::world::item::component::writable_book_content::WritableBookContent>(getter, &component) else {
            return self.pages.is_none();
        };
        self.pages.as_ref().map_or(true, |p| p.test(value.pages.iter().cloned()))
    }
}

/// `WrittenBookPredicate.PagePredicate` — `Component contents`.
#[derive(Clone, Debug, PartialEq)]
pub struct WrittenBookPagePredicate {
    /// `contents`.
    pub contents: Component,
}

impl WrittenBookPagePredicate {
    /// `CODEC` → NBT: Component codec (JSON→NBT).
    pub fn encode_nbt(&self) -> Tag {
        let json = component_to_json(&self.contents);
        crate::nbt::json::tag_from_json(&json).unwrap_or(Tag::End)
    }

    /// `CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let json = crate::nbt::json::json_from_tag(tag).map_err(|e| e.to_string())?;
        let contents = component_from_json(&json)?;
        Ok(Self { contents })
    }
}

impl CollectionTest<Filterable<Component>> for WrittenBookPagePredicate {
    fn test(&self, value: &Filterable<Component>) -> bool {
        value.raw == self.contents
    }
    fn encode_nbt(&self) -> Tag {
        WrittenBookPagePredicate::encode_nbt(self)
    }
    fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        WrittenBookPagePredicate::decode_nbt(tag)
    }
}

/// `WrittenBookPredicate` — (pages?, author?, title?, generation,
/// resolved?).
#[derive(Clone, Debug, PartialEq)]
pub struct WrittenBookPredicate {
    /// `pages`.
    pub pages: Option<CollectionPredicate<Filterable<Component>, WrittenBookPagePredicate>>,
    /// `author`.
    pub author: Option<String>,
    /// `title`.
    pub title: Option<String>,
    /// `generation` — `MinMaxBounds.Ints` (default ANY).
    pub generation: MinMaxBoundsInts,
    /// `resolved`.
    pub resolved: Option<bool>,
}

impl WrittenBookPredicate {
    /// `CODEC` → NBT.
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        let mut c = CompoundTag::new();
        if let Some(p) = &self.pages {
            c.put("pages", p.encode_nbt());
        }
        if let Some(a) = &self.author {
            c.put("author", Tag::String(a.clone()));
        }
        if let Some(t) = &self.title {
            c.put("title", Tag::String(t.clone()));
        }
        if self.generation != MinMaxBoundsInts::ANY {
            c.put("generation", self.generation.encode());
        }
        if let Some(r) = self.resolved {
            c.put("resolved", Tag::Byte(r as i8));
        }
        Ok(Tag::Compound(c))
    }

    /// `CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = nbt_util::as_compound(tag, "written book predicate")?;
        let pages = match c.get("pages") {
            Some(t) => Some(CollectionPredicate::<Filterable<Component>, WrittenBookPagePredicate>::decode_nbt(t)?),
            None => None,
        };
        let author = c.get("author").map(|t| nbt_util::get_string(t, "author")).transpose()?;
        let title = c.get("title").map(|t| nbt_util::get_string(t, "title")).transpose()?;
        let generation = match c.get("generation") {
            Some(t) => MinMaxBoundsInts::decode(t)?,
            None => MinMaxBoundsInts::ANY,
        };
        let resolved = c.get("resolved").map(|t| nbt_util::get_bool(t, "resolved")).transpose()?;
        Ok(Self { pages, author, title, generation, resolved })
    }

    /// `matches` — `written_book_content`.
    pub fn matches(&self, getter: &dyn DataComponentGetter, component: DataComponentType) -> bool {
        let Some(value) = component_value::<crate::world::item::component::written_book_content::WrittenBookContent>(getter, &component) else {
            return false;
        };
        self.author.as_ref().map_or(true, |a| *a == value.author)
            && self.title.as_ref().map_or(true, |t| *t == value.title.as_ref().map(|f| f.raw.clone()).unwrap_or_default())
            && self.generation.matches(value.generation)
            && self.resolved.map_or(true, |r| r == value.resolved)
            && self.pages.as_ref().map_or(true, |p| p.test(value.pages.iter().cloned()))
    }
}

// ------------------------------------------------------------ AttributeModifiers

/// `AttributeModifiersPredicate.EntryPredicate` —
/// (attribute?, id?, amount: Doubles, operation?, slot?).
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeEntryPredicate {
    /// `attribute` — `HolderSet<Attribute>`.
    pub attribute: Option<HolderSet<Attribute>>,
    /// `id` — `Identifier`.
    pub id: Option<Identifier>,
    /// `amount` — `MinMaxBounds.Doubles` (default ANY).
    pub amount: MinMaxBoundsDoubles,
    /// `operation`.
    pub operation: Option<AttributeModifierOperation>,
    /// `slot` — `EquipmentSlotGroup`.
    pub slot: Option<EquipmentSlotGroup>,
}

impl AttributeEntryPredicate {
    /// `CODEC` → NBT.
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        let mut c = CompoundTag::new();
        if let Some(set) = &self.attribute {
            let t = holder_set_nbt::holder_set_encode(
                set,
                &Identifier::minecraft("attribute"),
                |_| None,
            )?;
            c.put("attribute", t);
        }
        if let Some(id) = &self.id {
            c.put("id", Tag::String(id.to_string()));
        }
        if self.amount.bounds.min.is_some() || self.amount.bounds.max.is_some() {
            c.put("amount", self.amount.encode());
        }
        if let Some(op) = self.operation {
            c.put("operation", Tag::String(op.get_serialized_name().to_string()));
        }
        if let Some(slot) = self.slot {
            c.put("slot", Tag::String(slot.get_serialized_name().to_string()));
        }
        Ok(Tag::Compound(c))
    }

    /// `CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = nbt_util::as_compound(tag, "attribute entry predicate")?;
        let attribute = match c.get("attribute") {
            Some(t) => Some(holder_set_nbt::holder_set_decode::<Attribute>(
                t,
                Identifier::minecraft("attribute"),
            )?),
            None => None,
        };
        let id = c.get("id").map(|t| nbt_util::get_string(t, "id").and_then(|s| Identifier::parse(&s).map_err(|e| e.to_string()))).transpose()?;
        let amount = match c.get("amount") {
            Some(t) => MinMaxBoundsDoubles::decode(t)?,
            None => MinMaxBoundsDoubles::ANY,
        };
        let operation = match c.get("operation") {
            Some(Tag::String(s)) => Some(parse_operation(s)?),
            Some(_) => return Err("operation must be string".into()),
            None => None,
        };
        let slot = match c.get("slot") {
            Some(Tag::String(s)) => Some(parse_slot_group(s)?),
            Some(_) => return Err("slot must be string".into()),
            None => None,
        };
        Ok(Self { attribute, id, amount, operation, slot })
    }

    /// `test(Entry)`.
    pub fn test(&self, entry: &ItemAttributeModifiersEntry) -> bool {
        if let Some(set) = &self.attribute {
            if !holder_set_nbt::holder_set_contains(set, &entry.attribute) {
                return false;
            }
        }
        if let Some(id) = &self.id {
            if *id != entry.modifier.id {
                return false;
            }
        }
        if !self.amount.matches(entry.modifier.amount) {
            return false;
        }
        if let Some(op) = self.operation {
            if op != entry.modifier.operation {
                return false;
            }
        }
        if let Some(slot) = self.slot {
            if slot != entry.slot {
                return false;
            }
        }
        true
    }
}

fn parse_operation(s: &str) -> Result<AttributeModifierOperation, String> {
    Ok(match s {
        "add_value" => AttributeModifierOperation::AddValue,
        "add_multiplied_base" => AttributeModifierOperation::AddMultipliedBase,
        "add_multiplied_total" => AttributeModifierOperation::AddMultipliedTotal,
        _ => return Err(format!("unknown operation '{s}'")),
    })
}

fn parse_slot_group(s: &str) -> Result<EquipmentSlotGroup, String> {
    Ok(match s {
        "any" => EquipmentSlotGroup::Any,
        "mainhand" => EquipmentSlotGroup::Mainhand,
        "offhand" => EquipmentSlotGroup::Offhand,
        "hand" => EquipmentSlotGroup::Hand,
        "feet" => EquipmentSlotGroup::Feet,
        "legs" => EquipmentSlotGroup::Legs,
        "chest" => EquipmentSlotGroup::Chest,
        "head" => EquipmentSlotGroup::Head,
        "armor" => EquipmentSlotGroup::Armor,
        "body" => EquipmentSlotGroup::Body,
        "saddle" => EquipmentSlotGroup::Saddle,
        _ => return Err(format!("unknown slot group '{s}'")),
    })
}

impl CollectionTest<ItemAttributeModifiersEntry> for AttributeEntryPredicate {
    fn test(&self, value: &ItemAttributeModifiersEntry) -> bool {
        AttributeEntryPredicate::test(self, value)
    }
    fn encode_nbt(&self) -> Tag {
        AttributeEntryPredicate::encode_nbt(self).unwrap_or(Tag::End)
    }
    fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        AttributeEntryPredicate::decode_nbt(tag)
    }
}

/// `AttributeModifiersPredicate` — `modifiers?: CollectionPredicate<
/// Entry, EntryPredicate>`.
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeModifiersPredicate {
    /// `modifiers`.
    pub modifiers: Option<CollectionPredicate<ItemAttributeModifiersEntry, AttributeEntryPredicate>>,
}

impl AttributeModifiersPredicate {
    /// `CODEC` → NBT `{modifiers?}`.
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        let mut c = CompoundTag::new();
        if let Some(m) = &self.modifiers {
            c.put("modifiers", m.encode_nbt());
        }
        Ok(Tag::Compound(c))
    }

    /// `CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = nbt_util::as_compound(tag, "attribute modifiers predicate")?;
        let modifiers = match c.get("modifiers") {
            Some(t) => Some(CollectionPredicate::<ItemAttributeModifiersEntry, AttributeEntryPredicate>::decode_nbt(t)?),
            None => None,
        };
        Ok(Self { modifiers })
    }

    /// `matches` — `attribute_modifiers` компонент.
    pub fn matches(&self, getter: &dyn DataComponentGetter, component: DataComponentType) -> bool {
        let Some(value) = component_value::<ItemAttributeModifiers>(getter, &component) else { return false };
        self.modifiers.as_ref().map_or(true, |p| p.test(value.modifiers.iter().cloned()))
    }
}

// ------------------------------------------------------------ Trim

/// `TrimPredicate` — (material?, pattern?).
#[derive(Clone, Debug, PartialEq)]
pub struct TrimPredicate {
    /// `material` — `HolderSet<TrimMaterial>`.
    pub material: Option<HolderSet<TrimMaterial>>,
    /// `pattern` — `HolderSet<TrimPattern>`.
    pub pattern: Option<HolderSet<TrimPattern>>,
}

impl TrimPredicate {
    /// `CODEC` → NBT.
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        let mut c = CompoundTag::new();
        if let Some(m) = &self.material {
            c.put("material", holder_set_nbt::holder_set_encode(
                m,
                &Identifier::minecraft("trim_material"),
                |_| None,
            )?);
        }
        if let Some(p) = &self.pattern {
            c.put("pattern", holder_set_nbt::holder_set_encode(
                p,
                &Identifier::minecraft("trim_pattern"),
                |_| None,
            )?);
        }
        Ok(Tag::Compound(c))
    }

    /// `CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = nbt_util::as_compound(tag, "trim predicate")?;
        let material = match c.get("material") {
            Some(t) => Some(holder_set_nbt::holder_set_decode::<TrimMaterial>(t, Identifier::minecraft("trim_material"))?),
            None => None,
        };
        let pattern = match c.get("pattern") {
            Some(t) => Some(holder_set_nbt::holder_set_decode::<TrimPattern>(t, Identifier::minecraft("trim_pattern"))?),
            None => None,
        };
        Ok(Self { material, pattern })
    }

    /// `matches` — `trim` (ArmorTrim) компонент.
    pub fn matches(&self, getter: &dyn DataComponentGetter, component: DataComponentType) -> bool {
        let Some(value) = component_value::<ArmorTrim>(getter, &component) else { return false };
        self.material.as_ref().map_or(true, |s| {
            holder_set_nbt::holder_set_contains(s, &value.material)
                || matches!(s, HolderSet::Named(_))
        }) && self.pattern.as_ref().map_or(true, |s| {
            holder_set_nbt::holder_set_contains(s, &value.pattern)
                || matches!(s, HolderSet::Named(_))
        })
    }
}

// ------------------------------------------------------------ JukeboxPlayable

/// `JukeboxPlayablePredicate` — `song?: HolderSet<JukeboxSong>`.
#[derive(Clone, Debug, PartialEq)]
pub struct JukeboxPlayablePredicate {
    /// `song`.
    pub song: Option<HolderSet<JukeboxSong>>,
}

impl JukeboxPlayablePredicate {
    /// `CODEC` → NBT.
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        let mut c = CompoundTag::new();
        if let Some(s) = &self.song {
            c.put("song", holder_set_nbt::holder_set_encode(
                s,
                &Identifier::minecraft("jukebox_song"),
                |_| None,
            )?);
        }
        Ok(Tag::Compound(c))
    }

    /// `CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = nbt_util::as_compound(tag, "jukebox playable predicate")?;
        let song = match c.get("song") {
            Some(t) => Some(holder_set_nbt::holder_set_decode::<JukeboxSong>(t, Identifier::minecraft("jukebox_song"))?),
            None => None,
        };
        Ok(Self { song })
    }

    /// `matches` — `jukebox_playable` компонент: хотя бы один
    /// holder set'а имеет key == song.key (Java сравнивает
    /// ResourceKey через unwrapKey).
    pub fn matches(&self, getter: &dyn DataComponentGetter, component: DataComponentType) -> bool {
        let Some(value) = component_value::<JukeboxPlayable>(getter, &component) else { return false };
        let Some(set) = &self.song else { return true };
        let value_key = value.song.key().cloned();
        match set {
            HolderSet::Named(_) => true, // без резолва считаем совпадением
            HolderSet::Direct(list) => {
                list.iter().any(|h| match (h, &value.song) {
                    (Holder::Key(k), _) => Some(k) == value_key.as_ref(),
                    (Holder::Reference(_), _) => false,
                    (Holder::Direct(_), _) => false,
                })
            }
        }
    }
}
