//! Перенос `net.minecraft.world.item.ItemStack`.
//!
//! Модель: `item: Option<Holder<Item>>` (None = EMPTY-item),
//! `count`, `components: PatchedDataComponentMap` (prototype + patch).
//!
//! Wire-кодеки (`ItemStack.java`):
//! - `OPTIONAL_STREAM_CODEC`: VarInt count (<=0 → EMPTY), затем
//!   `Item.STREAM_CODEC` (holderRegistry) + `DataComponentPatch.STREAM_CODEC`.
//! - `OPTIONAL_UNTRUSTED_STREAM_CODEC`: то же, но patch —
//!   `DELIMITED_STREAM_CODEC` (length-prefixed значения).
//! - `STREAM_CODEC`: optional + запрет пустого стека.

use crate::core::component::{DataComponentPatch, PatchedDataComponentMap};
use crate::core::holder::Holder;
use crate::core::registry::registries;
use crate::nbt::{CompoundTag, Tag};
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::{Identifier, ResourceKey};
use crate::world::item::item;
use std::cell::Cell;
use std::hash::{Hash, Hasher};

thread_local! {
    static ITEM_NBT_DEPTH: Cell<usize> = const { Cell::new(0) };
}

fn with_item_nbt_depth<T>(operation: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    ITEM_NBT_DEPTH.with(|depth| {
        let previous = depth.get();
        // Each nested stack adds its own compound and a components compound.
        if previous >= Tag::MAX_DEPTH / 2 {
            return Err("item stack NBT is too deeply nested".into());
        }

        struct RestoreDepth<'a>(&'a Cell<usize>, usize);
        impl Drop for RestoreDepth<'_> {
            fn drop(&mut self) {
                self.0.set(self.1);
            }
        }

        depth.set(previous + 1);
        let _restore = RestoreDepth(depth, previous);
        operation()
    })
}

/// `ItemStack` — стопка предметов.
#[derive(Clone, Debug)]
pub struct ItemStack {
    /// `item` — holder предмета (`None` только у EMPTY-подобных).
    pub item: Option<Holder<item::Item>>,
    /// `count`.
    pub count: i32,
    /// `components` — `PatchedDataComponentMap` (prototype + patch).
    pub components: PatchedDataComponentMap,
}

impl ItemStack {
    /// `ItemStack.EMPTY`.
    /// Статический EMPTY-стек (аналог `ItemStack.EMPTY`).
    pub fn empty_static() -> &'static ItemStack {
        static EMPTY: std::sync::OnceLock<ItemStack> = std::sync::OnceLock::new();
        EMPTY.get_or_init(ItemStack::empty)
    }

    pub fn empty() -> Self {
        Self { item: None, count: 0, components: PatchedDataComponentMap::empty() }
    }

    /// `new(itemHolder, count)` — с пустым prototype компонентов.
    pub fn new(item: Holder<item::Item>, count: i32) -> Self {
        Self {
            item: Some(item),
            count,
            components: PatchedDataComponentMap::empty(),
        }
    }

    /// `new(item, count, patch)` — Java-конструктор из optional-кодека.
    pub fn with_patch(item: Holder<item::Item>, count: i32, patch: DataComponentPatch) -> Self {
        let mut components = PatchedDataComponentMap::empty();
        // Удаления из сохранения сохраняются до загрузки прототипов предметов.
        components.restore_patch(&patch);
        Self {
            item: Some(item),
            count,
            components,
        }
    }

    /// `isEmpty` — `count <= 0` или нет предмета.
    pub fn is_empty(&self) -> bool {
        self.count <= 0 || self.item.is_none()
    }

    /// `getCount`.
    pub fn get_count(&self) -> i32 {
        self.count
    }

    /// `getItem` — registry-ключ предмета (для Direct — его id,
    /// для Reference — по реестру).
    pub fn item_identifier(&self) -> Option<crate::resources::identifier::Identifier> {
        match self.item.as_ref()? {
            Holder::Direct(i) => Some(i.id.clone()),
            Holder::Reference(id) => item::item_by_id(*id).map(|i| i.id.clone()),
            Holder::Key(k) => Some(k.identifier.clone()),
        }
    }

    /// `getItemHolder`.
    pub fn item_holder(&self) -> Option<&Holder<item::Item>> {
        self.item.as_ref()
    }

    /// `copy`.
    pub fn copy(&self) -> ItemStack {
        self.clone()
    }

    /// Persistent-формат Java. Пустая стопка использует пустой compound.
    pub fn save(&self, tag: CompoundTag) -> Result<Tag, String> {
        with_item_nbt_depth(|| self.save_within_depth(tag))
    }

    fn save_within_depth(&self, mut tag: CompoundTag) -> Result<Tag, String> {
        if self.is_empty() {
            for key in ["id", "count", "components"] {
                tag.remove(key);
            }
            return Ok(Tag::Compound(tag));
        }

        let id = self.item_identifier().ok_or("item has no registered identifier")?;
        if id == Identifier::minecraft("air") {
            return Err("cannot save air as an item stack".into());
        }
        let count = checked_stack_count(self.count)?;
        let components = self.components.as_patch().to_nbt()?;
        tag.put_string("id", &id.to_string());
        tag.put_int("count", count);
        if components.is_empty() {
            tag.remove("components");
        } else {
            tag.put("components", Tag::Compound(components));
        }
        Ok(Tag::Compound(tag))
    }

    pub fn from_nbt(tag: &CompoundTag) -> Result<Self, String> {
        with_item_nbt_depth(|| Self::decode_within_depth(tag))
    }

    fn decode_within_depth(tag: &CompoundTag) -> Result<Self, String> {
        if tag.is_empty() || (!tag.contains("id") && tag.get_int("count") == Some(0)) {
            return Ok(Self::empty());
        }
        let id = Identifier::parse(tag.get_string("id").ok_or("expected an item id")?)
            .map_err(|error| error.to_string())?;
        if id == Identifier::minecraft("air") {
            return Err("cannot decode air as an item stack".into());
        }
        let count = match tag.get("count") {
            Some(data) => checked_stack_count(
                data.as_long().and_then(|value| i32::try_from(value).ok()).ok_or("expected an item count")?
            )?,
            None => 1,
        };
        let patch = match tag.get("components") {
            Some(data) => DataComponentPatch::from_nbt(data.as_compound().ok_or("expected components compound")?)?,
            None => DataComponentPatch::empty(),
        };
        let item = item::item_holder_by_identifier(&id).unwrap_or_else(|| {
            Holder::Key(ResourceKey::create_with_registry_name(registries::item(), id))
        });
        Ok(Self::with_patch(item, count, patch))
    }

    /// `getUseDuration(entity)` — `Item.getUseDuration(stack, entity)`.
    /// Предметные переопределения (Bow/Crossbow/Trident → 72000,
    /// Brush → 200, Spyglass → 1200, EnderEye → 0, Instrument →
    /// `instrument.useDuration * 20`), иначе `consumable.consumeTicks()`,
    /// иначе 72000 при `blocks_attacks`/`kinetic_weapon`, иначе 0.
    pub fn get_use_duration_ticks(&self) -> i32 {
        if let Some(id) = self.item_identifier() {
            let path = id.get_path();
            match path {
                "bow" | "crossbow" | "trident" => return 72000,
                "brush" => return 200,
                "spyglass" => return 1200,
                "ender_eye" => return 0,
                _ => {}
            }
        }
        if let Some(i) = self.get_component::<crate::world::item::component::instrument_component::InstrumentComponent>(
            &crate::resources::identifier::Identifier::minecraft("instrument"),
        ) {
            return (i.instrument.value().map(|v| v.use_duration).unwrap_or(0.0) * 20.0).floor() as i32;
        }
        if let Some(c) = self.get_component::<crate::world::item::component::consumable::Consumable>(
            &Identifier::minecraft("consumable"),
        ) {
            return (c.consume_seconds * 20.0).floor() as i32;
        }
        if self.has_component(&Identifier::minecraft("blocks_attacks"))
            || self.has_component(&Identifier::minecraft("kinetic_weapon"))
        {
            72000
        } else {
            0
        }
    }

    /// `getHoverName()` — имя для подсказки (custom_name компонент или item id).
    pub fn get_hover_name(&self) -> crate::network::chat::component::Component {
        let id = crate::resources::identifier::Identifier::parse("custom_name")
            .expect("custom_name id");
        if let Some(c) = self.components.get_by_id::<crate::network::chat::component::Component>(&id) {
            return c.clone();
        }
        match self.item_identifier() {
            Some(id) => crate::network::chat::component::Component::translatable(
                format!("item.{}.{}", id.get_namespace(), id.get_path()),
                vec![],
            ),
            None => crate::network::chat::component::Component::literal("item"),
        }
    }

    /// `getDisplayName()` — синоним `getHoverName`.
    pub fn get_display_name(&self) -> crate::network::chat::component::Component {
        self.get_hover_name()
    }

    /// `getMaxStackSize()` — `item.getMaxStackSize()`:
    /// `MAX_STACK_SIZE` компонент предмета, default 64.
    pub fn get_max_stack_size(&self) -> i32 {
        let id = crate::resources::identifier::Identifier::parse("max_stack_size")
            .expect("max_stack_size id");
        if let Some(max) = self.components.get_by_id::<i32>(&id) {
            return *max;
        }
        64
    }


    /// `isSameItem(a, b)` — одинаковый item (без count/компонентов).
    pub fn is_same_item(a: &ItemStack, b: &ItemStack) -> bool {
        a.item == b.item
    }

    /// `isSameItemSameComponents` — сравнение без размера стопки.
    pub fn is_same_item_same_components(a: &ItemStack, b: &ItemStack) -> bool {
        Self::is_same_item(a, b) && a.components == b.components
    }

    /// `copyWithCount(count)`.
    pub fn copy_with_count(&self, count: i32) -> ItemStack {
        let mut s = self.copy();
        s.set_count(count);
        s
    }

    /// `matches(a, b)` — одинаковый item и count (без компонентов).
    pub fn matches(a: &ItemStack, b: &ItemStack) -> bool {
        a.item == b.item && a.count == b.count
    }

    /// `hashItemAndComponents`.
    pub fn hash_item_and_components(&self) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.item.hash(&mut h);
        self.count.hash(&mut h);
        for c in self.components.iter() {
            c.component_type.id.hash(&mut h);
            c.value.type_id().hash(&mut h);
        }
        h.finish()
    }

    /// `shrink(amount)` — уменьшить count.
    pub fn shrink(&mut self, amount: i32) {
        self.count -= amount;
        if self.count <= 0 {
            *self = ItemStack::empty();
        }
    }

    /// `split(amount)` — отщепить часть стопки.
    pub fn split(&mut self, amount: i32) -> ItemStack {
        let i = self.count.min(amount);
        let mut result = self.copy();
        result.count = i;
        self.shrink(i);
        result
    }

    /// `grow(amount)` — увеличить count.
    pub fn grow(&mut self, amount: i32) {
        self.count += amount;
    }

    /// `setCount(count)` — прямая установка count (с обрезкой <=0
    /// в empty, как в Java).
    pub fn set_count(&mut self, count: i32) {
        self.count = count;
        if self.count <= 0 {
            *self = ItemStack::empty();
        }
    }

    /// `limitSize(max)` — `count = min(count, max)`.
    pub fn limit_size(&mut self, max: i32) {
        if self.count > max {
            self.count = max;
        }
    }

    /// `get(component)` — значение компонента по registry-id.
    pub fn get_component<T: Clone + 'static>(
        &self,
        component_id: &crate::resources::identifier::Identifier,
    ) -> Option<T> {
        self.components
            .get_by_id::<T>(component_id)
            .cloned()
    }

    /// `has(component)` по registry-id.
    pub fn has_component(&self, component_id: &crate::resources::identifier::Identifier) -> bool {
        self.components.has_by_id(component_id)
    }

    /// `set(component, value)` — записать значение компонента.
    /// `None` равносильно удалению (как Java `set`/`remove`).
    pub fn set_component<T>(
        &mut self,
        component_id: &Identifier,
        value: Option<T>,
    ) where
        T: crate::core::component::data_component_type::ComponentValueData + 'static,
    {
        if let Some(ty) = crate::core::component::data_component_type::data_component_type_by_identifier(component_id) {
            self.components.set(&ty, value);
        }
    }

    /// `getMaxDamage()` — `max_damage` компонент (default 0).
    pub fn get_max_damage(&self) -> i32 {
        self.get_component::<i32>(
            &crate::resources::identifier::Identifier::minecraft("max_damage"),
        )
        .unwrap_or(0)
    }

    /// `getDamageValue()` — `damage` компонент (default 0).
    pub fn get_damage_value(&self) -> i32 {
        self.get_component::<i32>(
            &crate::resources::identifier::Identifier::minecraft("damage"),
        )
        .unwrap_or(0)
    }

    /// `setDamageValue(value)`.
    pub fn set_damage_value(&mut self, value: i32) {
        self.set_component(&Identifier::minecraft("damage"), Some(value));
    }

    /// `isDamageableItem()` — `max_damage > 0` и нет `unbreakable`.
    pub fn is_damageable_item(&self) -> bool {
        !self.has_component(&Identifier::minecraft("unbreakable"))
            && self.get_max_damage() > 0
    }

    /// `canBeHurtBy(damageSource)` — `damage_resistant` компонент.
    pub fn can_be_hurt_by(
        &self,
        damage_source: &crate::world::damagesource::damage_source::DamageSource,
    ) -> bool {
        match self.get_component::<crate::world::item::component::damage_resistant::DamageResistant>(
            &Identifier::minecraft("damage_resistant"),
        ) {
            Some(dr) => !damage_source.is_tag(&dr.types),
            None => true,
        }
    }

    /// `is(item)` — совпадение по registry-id предмета или тегу.
    pub fn is(&self, item_id: &crate::resources::identifier::Identifier) -> bool {
        if self.is_empty() {
            return false;
        }
        if let Some(id) = self.item_identifier() {
            return id == *item_id;
        }
        false
    }

    /// `is(TagKey)` — членство предмета в тегe (`item` реестр).
    /// Теги предметов грузятся через `tags::item_tags::register_tag`.
    pub fn is_in_item_tag(&self, tag: &crate::tags::tag_key::TagKey<item::Item>) -> bool {
        if self.is_empty() {
            return false;
        }
        match self.item_identifier() {
            Some(id) => crate::tags::damage_type_tags::is_member_of_tag(
                tag.location(),
                &id,
            ),
            None => false,
        }
    }

    /// `getDamageSource(attacker, fallback)` — источник урона
    /// предмета: компонент `damage_type` (EitherHolder) →
    /// `item.getItemDamageSource` → `fallback`.
    pub fn get_damage_source(
        &self,
        attacker: &crate::world::entity::living_entity::LivingEntity,
        fallback: impl FnOnce() -> crate::world::damagesource::damage_source::DamageSource,
    ) -> crate::world::damagesource::damage_source::DamageSource {
        use crate::resources::{Identifier, ResourceKey};
        use crate::util::either::Either;
        use crate::world::damagesource::damage_type::DamageType;
        use crate::world::item::either_holder::EitherHolder;
        let damage_type_id = Identifier::minecraft("damage_type");
        if let Some(either) =
            self.components.get_by_id::<EitherHolder<DamageType>>(&damage_type_id)
        {
            if let Either::Left(holder) = &either.contents {
                let mut source = crate::world::damagesource::damage_source::DamageSource::new(
                    holder.clone(),
                );
                source.causing_entity = Some(attacker.entity.clone());
                return source;
            }
        }
        if let Some(item_holder) = &self.item {
            if let Some(item) = item_holder.value() {
                if let Some(source) = item.get_item_damage_source(attacker) {
                    return source;
                }
            }
        }
        fallback()
    }

    /// `getAttackDamageBonus(target, damageAmount, source)` —
    /// делегирует `Item.getAttackDamageBonus`.
    pub fn get_attack_damage_bonus(
        &self,
        target: &crate::world::entity::entity::Entity,
        damage_amount: f32,
        damage_source: &crate::world::damagesource::damage_source::DamageSource,
    ) -> f32 {
        self.item
            .as_ref()
            .and_then(|h| h.value())
            .map(|i| i.get_attack_damage_bonus(target, damage_amount, damage_source))
            .unwrap_or(0.0)
    }

    /// `hurtEnemy(enemy, attacker)` — `Item.hurtEnemy` хук.
    pub fn hurt_enemy(
        &self,
        enemy: &crate::world::entity::living_entity::LivingEntity,
        attacker: &crate::world::entity::living_entity::LivingEntity,
    ) {
        if let Some(item) = self.item.as_ref().and_then(|h| h.value()) {
            item.hurt_enemy(self, enemy, attacker);
        }
    }

    /// `postHurtEnemy(target, attacker)` — `Item.postHurtEnemy`.
    pub fn post_hurt_enemy(
        &self,
        target: &crate::world::entity::living_entity::LivingEntity,
        attacker: &crate::world::entity::living_entity::LivingEntity,
    ) {
        if let Some(item) = self.item.as_ref().and_then(|h| h.value()) {
            item.post_hurt_enemy(self, target, attacker);
        }
    }

    /// `hurtAndBreak(amount, entity, slot)` — износ предмета.
    ///
    /// Java `ItemStack.hurtAndBreak(int, LivingEntity, EquipmentSlot)`:
    /// прочность уменьшается на `amount`, при разрушении предмет
    /// заменяется пустым и вызывается `entity.onEquippedItemBroken`.
    pub fn hurt_and_break(
        &mut self,
        amount: i32,
        entity: &mut crate::world::entity::living_entity::LivingEntity,
        slot: Option<crate::entity::equipment_slot::EquipmentSlot>,
    ) {
        if !self.is_damageable_item() {
            return;
        }
        if amount <= 0 {
            return;
        }
        let max_damage = self.get_max_damage();
        let new_damage = (self.get_damage_value() + amount).clamp(0, max_damage);
        self.set_damage_value(new_damage);
        if new_damage >= max_damage {
            let stack = self.copy();
            self.shrink(1);
            entity.on_equipped_item_broken(&stack, slot);
        }
    }

    /// Общий decode optional-кодека.
    fn decode_optional(buf: &mut FriendlyByteBuf, delimited: bool) -> ItemStack {
        let count = buf.read_var_int();
        if count <= 0 {
            return ItemStack::empty();
        }
        let item_holder = item::stream_codec().decode(buf);
        let patch = if delimited {
            DataComponentPatch::delimited_stream_codec().decode(buf)
        } else {
            DataComponentPatch::stream_codec().decode(buf)
        };
        ItemStack::with_patch(item_holder, count, patch)
    }

    /// Общий encode optional-кодека.
    fn encode_optional(stack: &ItemStack, buf: &mut FriendlyByteBuf, delimited: bool) {
        if stack.is_empty() {
            buf.write_var_int(0);
            return;
        }
        buf.write_var_int(stack.count);
        item::stream_codec().encode(buf, stack.item.as_ref().expect("item"));
        let patch = stack.components.as_patch();
        if delimited {
            DataComponentPatch::delimited_stream_codec().encode(buf, &patch);
        } else {
            DataComponentPatch::stream_codec().encode(buf, &patch);
        }
    }

    /// `OPTIONAL_STREAM_CODEC`.
    pub fn optional_stream_codec(
    ) -> impl StreamCodec<FriendlyByteBuf, ItemStack> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, s: &ItemStack| {
                Self::encode_optional(s, buf, false);
            },
            |buf: &mut FriendlyByteBuf| Self::decode_optional(buf, false),
        )
    }

    /// `OPTIONAL_UNTRUSTED_STREAM_CODEC` — delimited patch.
    pub fn optional_untrusted_stream_codec(
    ) -> impl StreamCodec<FriendlyByteBuf, ItemStack> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, s: &ItemStack| {
                Self::encode_optional(s, buf, true);
            },
            |buf: &mut FriendlyByteBuf| Self::decode_optional(buf, true),
        )
    }

    /// `STREAM_CODEC` — optional + non-empty.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ItemStack> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, s: &ItemStack| {
                if s.is_empty() {
                    panic!("Empty ItemStack not allowed");
                }
                Self::encode_optional(s, buf, false);
            },
            |buf: &mut FriendlyByteBuf| {
                let s = Self::decode_optional(buf, false);
                if s.is_empty() {
                    panic!("Empty ItemStack not allowed");
                }
                s
            },
        )
    }

    /// `OPTIONAL_LIST_STREAM_CODEC`.
    pub fn optional_list_stream_codec(
    ) -> impl StreamCodec<FriendlyByteBuf, Vec<ItemStack>> + Send + Sync + 'static {
        crate::network::codec::byte_buf_codecs::list(Self::optional_stream_codec())
    }
}

fn checked_stack_count(count: i32) -> Result<i32, String> {
    if (1..=99).contains(&count) {
        Ok(count)
    } else {
        Err(format!("item count {count} is outside 1..=99"))
    }
}

impl PartialEq for ItemStack {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item && self.count == other.count
    }
}

impl crate::core::component::data_component_getter::DataComponentGetter for ItemStack {
    fn get_raw(&self, component: &crate::core::component::data_component_type::DataComponentType) -> Option<crate::core::component::data_component_type::ComponentValue> {
        self.components.effective_raw(&component.id)
    }

    fn has_component(&self, component: &crate::core::component::data_component_type::DataComponentType) -> bool {
        self.components.has(component)
    }

    fn iter_raw(&self) -> Vec<(crate::resources::identifier::Identifier, Option<crate::core::component::data_component_type::ComponentValue>)> {
        self.components
            .iter()
            .into_iter()
            .map(|t| (t.component_type.id.clone(), Some(t.value)))
            .collect()
    }
}

#[cfg(test)]
mod persistence_tests {
    use super::*;
    use crate::core::component::data_component_type::data_component_type_by_identifier;
    use crate::world::item::component::item_lore::ItemLore;

    #[test]
    fn restores_typed_item_components_and_removed_patch_entries() {
        let id = Identifier::parse("fixture:durable_item").unwrap();
        let wire_id = item::register_item(item::Item::new(id.clone()));
        let mut styled = CompoundTag::new();
        styled.put_string("text", "Second");
        styled.put_bool("bold", true);
        let mut patch = CompoundTag::new();
        patch.put_int("damage", 7);
        patch.put_string("custom_name", "Leaf blade");
        patch.put("minecraft:!invalid", Tag::Compound(CompoundTag::new()));
        assert!(DataComponentPatch::from_nbt(&patch).is_err());
        patch.remove("minecraft:!invalid");
        patch.put("!minecraft:unbreakable", Tag::Compound(CompoundTag::new()));
        patch.put("lore", Tag::List(vec![Tag::String("First".into()), Tag::Compound(styled)]));
        let mut nbt = CompoundTag::new();
        nbt.put_string("id", &id.to_string());
        nbt.put("components", Tag::Compound(patch));

        let stack = ItemStack::from_nbt(&nbt).unwrap();
        assert_eq!(stack.item_holder(), Some(&Holder::Reference(wire_id)));
        assert_eq!(stack.get_count(), 1);
        assert_eq!(stack.get_component::<i32>(&Identifier::minecraft("damage")), Some(7));
        let lore = stack.get_component::<ItemLore>(&Identifier::minecraft("lore")).unwrap();
        assert_eq!(lore.lines.len(), 2);
        assert_eq!(lore.lines[1].style.bold, Some(true));

        let saved = stack.save(CompoundTag::new()).unwrap();
        let components = saved.as_compound().unwrap().get_compound("components").unwrap();
        assert_eq!(components.get_int("minecraft:damage"), Some(7));
        assert!(components.get_compound("!minecraft:unbreakable").unwrap().is_empty());
        assert_eq!(components.get_string("minecraft:custom_name"), Some("Leaf blade"));
        let bytes = crate::nbt::io::write(saved.as_compound().unwrap()).unwrap();
        let reread = crate::nbt::io::read_bytes_unlimited(&bytes).unwrap();
        let reread = ItemStack::from_nbt(&reread).unwrap();
        assert_eq!(reread.components.as_patch().to_nbt().unwrap(), *components);
    }

    #[test]
    fn restores_nested_items_with_sparse_slots_and_bundle_weight() {
        use crate::world::item::component::{
            bundle_contents::BundleContents, charged_projectiles::ChargedProjectiles,
            item_container_contents::ItemContainerContents, use_remainder::UseRemainder,
        };
        use crate::util::fraction::Fraction;
        let nbt = crate::nbt::snbt_parse::parse_compound(
            r#"{id:"minecraft:shulker_box",components:{"minecraft:container":[
                {slot:4,item:{id:"minecraft:bundle",components:{"minecraft:bundle_contents":[
                    {id:"minecraft:stone",count:2,"components":{"minecraft:max_stack_size":32}}
                ]}}},
                {slot:255,item:{id:"minecraft:crossbow",components:{"minecraft:charged_projectiles":[
                    {id:"minecraft:arrow",components:{"minecraft:use_remainder":{id:"minecraft:stick"}}}
                ]}}}
            ]}}"#,
        ).unwrap();
        let stack = ItemStack::from_nbt(&nbt).unwrap();
        let container = stack.get_component::<ItemContainerContents>(&Identifier::minecraft("container")).unwrap();
        assert_eq!(container.items.len(), 256);
        assert!(container.items[0].is_empty());
        let bundle = container.items[4].get_component::<BundleContents>(&Identifier::minecraft("bundle_contents")).unwrap();
        assert_eq!(bundle.weight, Fraction::new(1, 16));
        assert_eq!(bundle.selected_item, -1);
        let projectiles = container.items[255].get_component::<ChargedProjectiles>(&Identifier::minecraft("charged_projectiles")).unwrap();
        let remainder = projectiles.items[0].get_component::<UseRemainder>(&Identifier::minecraft("use_remainder")).unwrap();
        assert_eq!(remainder.convert_into.item_identifier(), Some(Identifier::minecraft("stick")));

        let saved = stack.save(CompoundTag::new()).unwrap();
        let slots = saved.as_compound().unwrap().get_compound("components").unwrap()
            .get("minecraft:container").unwrap().as_list().unwrap();
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[1].as_compound().unwrap().get_int("slot"), Some(255));
        assert_eq!(ItemStack::from_nbt(saved.as_compound().unwrap()).unwrap(), stack);
    }

    #[test]
    fn rejects_invalid_nested_item_shapes_and_unrepresentable_bundle_weight() {
        use crate::nbt::snbt_parse::parse_compound;

        for (contents, error) in [
            (r#"container:[{slot:256,item:{id:stone}}]"#, "slot in 0..=255"),
            (r#"charged_projectiles:[{}]"#, "nonempty item stack"),
            (r#"use_remainder:{id:stone,count:0}"#, "item count"),
            (
                r#"bundle_contents:[{id:stone,components:{max_stack_size:89}},
                    {id:stone,components:{max_stack_size:83}},
                    {id:stone,components:{max_stack_size:79}},
                    {id:stone,components:{max_stack_size:73}},
                    {id:stone,components:{max_stack_size:71}},
                    {id:stone,components:{max_stack_size:67}}]"#,
                "excessive total bundle weight",
            ),
        ] {
            let item = parse_compound(&format!("{{id:stone,components:{{{contents}}}}}")).unwrap();
            let message = ItemStack::from_nbt(&item).unwrap_err();
            assert!(message.contains(error), "{message}");
        }

        let mut item = parse_compound("{id:stone}").unwrap();
        for _ in 0..Tag::MAX_DEPTH / 2 {
            let mut parent = parse_compound("{id:stone}").unwrap();
            let mut components = CompoundTag::new();
            components.put("use_remainder", Tag::Compound(item));
            parent.put("components", Tag::Compound(components));
            item = parent;
        }
        let message = ItemStack::from_nbt(&item).unwrap_err();
        assert!(message.contains("too deeply nested"), "{message}");
        assert!(ItemStack::from_nbt(&parse_compound("{id:stone}").unwrap()).is_ok());
    }

    #[test]
    fn invalid_component_values_report_errors_without_serializing_placeholders() {
        let component = data_component_type_by_identifier(&Identifier::minecraft("damage")).unwrap();
        let mut patch = DataComponentPatch::builder();
        patch.set(&component, -5_i32);
        let stack = ItemStack::with_patch(
            Holder::Direct(item::Item::new(Identifier::minecraft("stone"))), 1, patch.build(),
        );
        assert!(stack.save(CompoundTag::new()).unwrap_err().contains("minecraft:damage"));
        assert!(ItemStack::from_nbt(&CompoundTag::new()).unwrap().is_empty());
    }
}
