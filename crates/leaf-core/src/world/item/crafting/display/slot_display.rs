//! Перенос `net.minecraft.world.item.crafting.display.SlotDisplay`.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::vec_delta_codec;
use crate::resources::identifier::Identifier;
use crate::tags::tag_key::TagKey;
use crate::world::item::item::Item;
use crate::world::item::item_stack::ItemStack;
use crate::world::item::equipment::trim::trim_pattern::TrimPattern;

/// `SlotDisplay` — отображение слота в рецепте.
#[derive(Clone, Debug)]
pub enum SlotDisplay {
    /// `Empty`.
    Empty,
    /// `AnyFuel`.
    AnyFuel,
    /// `ItemSlotDisplay`.
    Item(Holder<Item>),
    /// `ItemStackSlotDisplay`.
    ItemStack(ItemStack),
    /// `TagSlotDisplay`.
    Tag(TagKey<Item>),
    /// `SmithingTrimDemoSlotDisplay`.
    SmithingTrim {
        /// `base`.
        base: Box<SlotDisplay>,
        /// `material`.
        material: Box<SlotDisplay>,
        /// `pattern`.
        pattern: Holder<TrimPattern>,
    },
    /// `WithRemainder`.
    WithRemainder {
        /// `input`.
        input: Box<SlotDisplay>,
        /// `remainder`.
        remainder: Box<SlotDisplay>,
    },
    /// `Composite`.
    Composite(Vec<SlotDisplay>),
}

///  — ResourceKey для item registry.
fn item_tag_key() -> crate::resources::resource_key::ResourceKey<crate::core::registry::Registry<crate::world::item::item::Item>> {
    crate::resources::resource_key::ResourceKey::create_registry_key(
        crate::resources::identifier::Identifier::minecraft("item")
    )
}

impl SlotDisplay {
    /// `type()` — registry id.
    pub fn type_id(&self) -> i32 {
        match self {
            SlotDisplay::Empty => 0,
            SlotDisplay::AnyFuel => 1,
            SlotDisplay::Item(_) => 2,
            SlotDisplay::ItemStack(_) => 3,
            SlotDisplay::Tag(_) => 4,
            SlotDisplay::SmithingTrim { .. } => 5,
            SlotDisplay::WithRemainder { .. } => 6,
            SlotDisplay::Composite(_) => 7,
        }
    }

    /// `STREAM_CODEC` — encode.
    pub fn encode(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.type_id());
        match self {
            SlotDisplay::Empty | SlotDisplay::AnyFuel => {}
            SlotDisplay::Item(item) => {
                crate::world::item::item::stream_codec().encode(buf, item);
            }
            SlotDisplay::ItemStack(stack) => {
                ItemStack::stream_codec().encode(buf, stack);
            }
            SlotDisplay::Tag(tag) => {
                buf.write_identifier(&tag.location).ok();
            }
            SlotDisplay::SmithingTrim { base, material, pattern } => {
                base.encode(buf);
                material.encode(buf);
                crate::world::item::equipment::trim::trim_pattern::TrimPattern::stream_codec().encode(buf, pattern);
            }
            SlotDisplay::WithRemainder { input, remainder } => {
                input.encode(buf);
                remainder.encode(buf);
            }
            SlotDisplay::Composite(contents) => {
                buf.write_collection(contents, |b, c| c.encode(b));
            }
        }
    }

    /// `STREAM_CODEC` — decode.
    pub fn decode(buf: &mut FriendlyByteBuf) -> Self {
        let type_id = buf.read_var_int();
        match type_id {
            0 => SlotDisplay::Empty,
            1 => SlotDisplay::AnyFuel,
            2 => SlotDisplay::Item(crate::world::item::item::stream_codec().decode(buf)),
            3 => SlotDisplay::ItemStack(ItemStack::stream_codec().decode(buf)),
            4 => SlotDisplay::Tag(TagKey::create(item_tag_key(), buf.read_identifier().unwrap_or_else(|_| crate::resources::identifier::Identifier::from_namespace_and_path("minecraft", "unknown").unwrap()))),
            5 => {
                let base = Box::new(Self::decode(buf));
                let material = Box::new(Self::decode(buf));
                let pattern = crate::world::item::equipment::trim::trim_pattern::TrimPattern::stream_codec().decode(buf);
                SlotDisplay::SmithingTrim { base, material, pattern }
            }
            6 => {
                let input = Box::new(Self::decode(buf));
                let remainder = Box::new(Self::decode(buf));
                SlotDisplay::WithRemainder { input, remainder }
            }
            7 => {
                let contents = buf.read_collection(|n| Vec::with_capacity(n), |b| Self::decode(b));
                SlotDisplay::Composite(contents)
            }
            _ => SlotDisplay::Empty,
        }
    }

    /// `resolveForFirstStack` — упрощённо возвращает первый ItemStack.
    pub fn resolve_for_first_stack(&self) -> ItemStack {
        match self {
            SlotDisplay::ItemStack(stack) => stack.clone(),
            SlotDisplay::Item(item) => ItemStack::new(item.clone(), 1),
            SlotDisplay::Composite(contents) => {
                contents.iter().find_map(|c| {
                    let s = c.resolve_for_first_stack();
                    if s.is_empty() { None } else { Some(s) }
                }).unwrap_or_else(|| ItemStack::empty())
            }
            _ => ItemStack::empty(),
        }
    }
}
