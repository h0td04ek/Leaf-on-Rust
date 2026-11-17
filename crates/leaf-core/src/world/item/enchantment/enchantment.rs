//! Перенос `net.minecraft.world.item.enchantment.Enchantment`.

use crate::core::component::data_component_map::DataComponentMap;
use crate::core::holder::Holder;
use crate::core::holder_set::HolderSet;
use crate::entity::equipment_slot_group::EquipmentSlotGroup;
use crate::network::chat::component::Component;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use crate::world::item::item::Item;
use std::sync::Arc;

/// `Enchantment.Cost` — (base, perLevelAboveFirst).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnchantmentCost {
    /// `base`.
    pub base: i32,
    /// `perLevelAboveFirst`.
    pub per_level_above_first: i32,
}

impl EnchantmentCost {
    /// `new`.
    pub fn new(base: i32, per_level_above_first: i32) -> Self {
        Self { base, per_level_above_first }
    }

    /// `calculate(level)`.
    pub fn calculate(&self, level: i32) -> i32 {
        self.base + self.per_level_above_first * (level - 1)
    }
}

/// `Enchantment.EnchantmentDefinition` — параметры зачарования.
#[derive(Clone, Debug, PartialEq)]
pub struct EnchantmentDefinition {
    /// `supportedItems`.
    pub supported_items: HolderSet<Item>,
    /// `primaryItems`.
    pub primary_items: Option<HolderSet<Item>>,
    /// `weight`.
    pub weight: i32,
    /// `maxLevel`.
    pub max_level: i32,
    /// `minCost`.
    pub min_cost: EnchantmentCost,
    /// `maxCost`.
    pub max_cost: EnchantmentCost,
    /// `anvilCost`.
    pub anvil_cost: i32,
    /// `slots`.
    pub slots: Vec<EquipmentSlotGroup>,
}

impl EnchantmentDefinition {
    /// `new`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        supported_items: HolderSet<Item>,
        primary_items: Option<HolderSet<Item>>,
        weight: i32,
        max_level: i32,
        min_cost: EnchantmentCost,
        max_cost: EnchantmentCost,
        anvil_cost: i32,
        slots: Vec<EquipmentSlotGroup>,
    ) -> Self {
        Self {
            supported_items,
            primary_items,
            weight,
            max_level,
            min_cost,
            max_cost,
            anvil_cost,
            slots,
        }
    }
}

/// `Enchantment` — (description, definition, exclusiveSet, effects).
#[derive(Clone, Debug, PartialEq)]
pub struct Enchantment {
    /// `description`.
    pub description: Component,
    /// `definition`.
    pub definition: EnchantmentDefinition,
    /// `exclusiveSet`.
    pub exclusive_set: HolderSet<Enchantment>,
    /// `effects` — component map эффектов.
    pub effects: Arc<DataComponentMap>,
}

impl Enchantment {
    /// `new`.
    pub fn new(
        description: Component,
        definition: EnchantmentDefinition,
        exclusive_set: HolderSet<Enchantment>,
        effects: DataComponentMap,
    ) -> Self {
        Self { description, definition, exclusive_set, effects: Arc::new(effects) }
    }

    /// `Enchantment.STREAM_CODEC` — `holderRegistry(ENCHANTMENT)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<Enchantment>> + Send + Sync + 'static {
        crate::core::holder::holder_registry_stream_codec::<Enchantment>()
    }
}
