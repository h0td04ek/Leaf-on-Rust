//! Перенос `net.minecraft.world.item.component.ItemAttributeModifiers`.

use crate::core::holder::Holder;
use crate::entity::ai::attributes::attribute::Attribute;
use crate::entity::ai::attributes::attribute_modifier::AttributeModifier;
use crate::entity::equipment_slot_group::EquipmentSlotGroup;
use crate::network::chat::component_json::component_stream_codec;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `ItemAttributeModifiers.Display` — вариант отображения модификатора.
#[derive(Clone, Debug, PartialEq)]
pub enum ItemAttributeModifiersDisplay {
    /// `Display.Default` — idMapper id=0.
    Default,
    /// `Display.Hidden` — id=1.
    Hidden,
    /// `Display.OverrideText` — id=2 + Component.
    OverrideText(crate::network::chat::component::Component),
}

impl ItemAttributeModifiersDisplay {
    /// `Type` id для `Display.Type.STREAM_CODEC` (idMapper).
    pub fn type_id(&self) -> i32 {
        match self {
            ItemAttributeModifiersDisplay::Default => 0,
            ItemAttributeModifiersDisplay::Hidden => 1,
            ItemAttributeModifiersDisplay::OverrideText(_) => 2,
        }
    }

    /// `STREAM_CODEC` — `Display.Type` (idMapper) + optional(Component)
    /// для OverrideText.
    /// В Java `Display.STREAM_CODEC` = composite(Type, optional(Component))
    /// с маппингом по типу.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Self> + Send + Sync + 'static {
        let comp_enc = byte_buf_codecs::optional(component_stream_codec());
        let comp_dec = byte_buf_codecs::optional(component_stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &Self| {
                buf.write_var_int(v.type_id());
                match v {
                    ItemAttributeModifiersDisplay::OverrideText(c) => {
                        comp_enc.encode(buf, &Some(c.clone()));
                    }
                    _ => {
                        comp_enc.encode(buf, &None::<crate::network::chat::component::Component>);
                    }
                }
            },
            move |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                match id {
                    0 => ItemAttributeModifiersDisplay::Default,
                    1 => ItemAttributeModifiersDisplay::Hidden,
                    2 => {
                        let c = comp_dec
                            .decode(buf)
                            .expect("override text component");
                        ItemAttributeModifiersDisplay::OverrideText(c)
                    }
                    _ => ItemAttributeModifiersDisplay::Default,
                }
            },
        )
    }
}

/// `ItemAttributeModifiers.Entry` — (attribute, modifier, slot,
/// display).
#[derive(Clone, Debug, PartialEq)]
pub struct ItemAttributeModifiersEntry {
    /// `attribute`.
    pub attribute: Holder<Attribute>,
    /// `modifier`.
    pub modifier: AttributeModifier,
    /// `slot`.
    pub slot: EquipmentSlotGroup,
    /// `display`.
    pub display: ItemAttributeModifiersDisplay,
}

impl ItemAttributeModifiersEntry {
    /// `new`.
    pub fn new(
        attribute: Holder<Attribute>,
        modifier: AttributeModifier,
        slot: EquipmentSlotGroup,
        display: ItemAttributeModifiersDisplay,
    ) -> Self {
        Self { attribute, modifier, slot, display }
    }

    /// `Entry.STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Self> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &Self| {
                Attribute::stream_codec().encode(buf, &v.attribute);
                AttributeModifier::stream_codec().encode(buf, &v.modifier);
                EquipmentSlotGroup::stream_codec().encode(buf, &v.slot);
                ItemAttributeModifiersDisplay::stream_codec().encode(buf, &v.display);
            },
            |buf: &mut FriendlyByteBuf| {
                let attribute = Attribute::stream_codec().decode(buf);
                let modifier = AttributeModifier::stream_codec().decode(buf);
                let slot = EquipmentSlotGroup::stream_codec().decode(buf);
                let display = ItemAttributeModifiersDisplay::stream_codec().decode(buf);
                ItemAttributeModifiersEntry::new(attribute, modifier, slot, display)
            },
        )
    }
}

/// `ItemAttributeModifiers` — `List<Entry>`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemAttributeModifiers {
    /// `modifiers`.
    pub modifiers: Vec<ItemAttributeModifiersEntry>,
}

impl ItemAttributeModifiers {
    /// `EMPTY`.
    pub fn empty() -> Self {
        Self { modifiers: Vec::new() }
    }

    /// `new`.
    pub fn new(modifiers: Vec<ItemAttributeModifiersEntry>) -> Self {
        Self { modifiers }
    }

    /// `STREAM_CODEC` — list(Entry).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ItemAttributeModifiers> + Send + Sync + 'static {
        let list_enc = byte_buf_codecs::list(ItemAttributeModifiersEntry::stream_codec());
        let list_dec = byte_buf_codecs::list(ItemAttributeModifiersEntry::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &ItemAttributeModifiers| {
                list_enc.encode(buf, &v.modifiers);
            },
            move |buf: &mut FriendlyByteBuf| ItemAttributeModifiers {
                modifiers: list_dec.decode(buf),
            },
        )
    }
}
