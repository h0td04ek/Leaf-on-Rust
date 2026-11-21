//! Перенос `net.minecraft.world.item.component.BundleContents`.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::util::fraction::Fraction;
use crate::world::item::item_stack::ItemStack;

/// `BundleContents` — (items, weight, selectedItem).
/// `BUNDLE_IN_BUNDLE_WEIGHT = 1/16`.
#[derive(Clone, Debug, PartialEq)]
pub struct BundleContents {
    /// `items`.
    pub items: Vec<ItemStack>,
    /// `weight` — суммарный вес содержимого.
    pub weight: Fraction,
    /// `selectedItem` (-1 = нет).
    pub selected_item: i32,
}

impl BundleContents {
    /// `EMPTY`.
    pub fn empty() -> Self {
        Self { items: Vec::new(), weight: Fraction::ZERO, selected_item: -1 }
    }

    /// `NO_SELECTED_ITEM_INDEX`.
    pub const NO_SELECTED_ITEM_INDEX: i32 = -1;

    /// `BUNDLE_IN_BUNDLE_WEIGHT`.
    pub const BUNDLE_IN_BUNDLE_WEIGHT: Fraction = Fraction { numerator: 1, denominator: 16 };

    /// `new(items)` — вычисляет weight, selectedItem=-1.
    pub fn new(items: Vec<ItemStack>) -> Self {
        let weight = Self::compute_content_weight(&items);
        Self { items, weight, selected_item: Self::NO_SELECTED_ITEM_INDEX }
    }

    pub(crate) fn from_persistent_items(items: Vec<ItemStack>) -> Result<Self, String> {
        fn checked(weight: Fraction) -> Result<Fraction, String> {
            if weight.numerator > i32::MAX as i64 || weight.denominator > i32::MAX as i64 {
                return Err("excessive total bundle weight".into());
            }
            Ok(weight)
        }

        let mut weight = Fraction::ZERO;
        for stack in &items {
            let item_weight = checked(Self::get_weight(stack))?;
            let stack_weight = checked(item_weight.multiply(i64::from(stack.get_count())))?;
            weight = checked(weight + stack_weight)?;
        }
        Ok(Self {
            items,
            weight,
            selected_item: Self::NO_SELECTED_ITEM_INDEX,
        })
    }

    /// `computeContentWeight` — сумма `stack.getWeight()`
    /// (`BundleContents.getWeight` × count).
    pub fn compute_content_weight(items: &[ItemStack]) -> Fraction {
        let mut total = Fraction::ZERO;
        for stack in items {
            total = total
                + Self::get_weight(stack)
                    .multiply_by(Fraction::new(stack.get_count() as i64, 1));
        }
        total
    }

    /// `getWeight(stack)` — `BUNDLE_IN_BUNDLE_WEIGHT + weight` если
    /// внутри бандл; `1` если есть пчёлы; иначе `1/maxStackSize`.
    pub fn get_weight(stack: &ItemStack) -> Fraction {
        let bundle_id = crate::resources::identifier::Identifier::parse("bundle_contents")
            .expect("bundle_contents id");
        if let Some(contents) = stack
            .components
            .get_by_id::<BundleContents>(&bundle_id)
        {
            return Self::BUNDLE_IN_BUNDLE_WEIGHT.add(contents.weight);
        }
        let bees_id = crate::resources::identifier::Identifier::parse("bees")
            .expect("bees id");
        if let Some(bees) = stack.components.get_by_id::<crate::world::item::component::bees::Bees>(&bees_id) {
            if !bees.bees.is_empty() {
                return Fraction::ONE;
            }
        }
        Fraction::new(1, stack.get_max_stack_size() as i64)
    }

    /// `STREAM_CODEC` — list(256, ItemStack) (weight/selected —
    /// не на проводе).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, BundleContents> + Send + Sync + 'static {
        let list_enc = byte_buf_codecs::list_limited(256, ItemStack::stream_codec());
        let list_dec = byte_buf_codecs::list_limited(256, ItemStack::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &BundleContents| {
                list_enc.encode(buf, &v.items);
            },
            move |buf: &mut FriendlyByteBuf| BundleContents::new(list_dec.decode(buf)),
        )
    }
}
