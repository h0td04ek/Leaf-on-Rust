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
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use crate::world::item::item;
use std::hash::{Hash, Hasher};

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
        Self {
            item: Some(item),
            count,
            components: PatchedDataComponentMap::from_patch(
                Default::default(),
                &patch,
            ),
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

    /// `save(tag)` — сериализация в NBT (id, count, components).
    pub fn save(&self, mut tag: crate::nbt::compound::CompoundTag) -> crate::nbt::tag::Tag {
        if let Some(id) = self.item_identifier() {
            tag.put("id", crate::nbt::tag::Tag::String(id.to_string()));
        }
        tag.put("count", crate::nbt::tag::Tag::Int(self.get_count()));
        // components — сохраняем по id типа компонента
        let mut comps = crate::nbt::compound::CompoundTag::new();
        for tc in self.components.iter() {
            let key = crate::core::component::data_component_type::data_component_type_id(&tc.component_type)
                .map(|id| id.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            comps.put(&key, crate::nbt::tag::Tag::String("component".to_string()));
        }
        tag.put("components", crate::nbt::tag::Tag::Compound(comps));
        crate::nbt::tag::Tag::Compound(tag)
    }

    /// `getUseDuration(entity)` — `Item.getUseDuration(stack, entity)`
    /// базовая реализация: `consumable.consumeTicks()`, иначе 72000
    /// при `blocks_attacks`/`kinetic_weapon`, иначе 0.
    pub fn get_use_duration_ticks(&self) -> i32 {
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
        match self.item_holder().and_then(|h| h.key().map(|k| k.id().clone())) {
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
        use crate::resources::identifier::Identifier;
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
