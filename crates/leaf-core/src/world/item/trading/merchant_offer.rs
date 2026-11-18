//! `MerchantOffer` — предложение в трейде.

use std::sync::Arc;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::item_stack::ItemStack;
use crate::world::item::trading::item_cost::ItemCost;

/// `MerchantOffer` — `{ baseCostA, costB, result, uses, maxUses, rewardExp, specialPriceDiff, demand, priceMultiplier, xp, ignoreDiscounts }`.
#[derive(Clone, Debug)]
pub struct MerchantOffer {
    /// `baseCostA`.
    pub base_cost_a: ItemCost,
    /// `costB` — `Optional<ItemCost>`.
    pub cost_b: Option<ItemCost>,
    /// `result`.
    pub result: ItemStack,
    /// `uses`.
    pub uses: i32,
    /// `maxUses`.
    pub max_uses: i32,
    /// `rewardExp`.
    pub reward_exp: bool,
    /// `specialPriceDiff`.
    pub special_price_diff: i32,
    /// `demand`.
    pub demand: i32,
    /// `priceMultiplier`.
    pub price_multiplier: f32,
    /// `xp`.
    pub xp: i32,
    /// `ignoreDiscounts` — Paper.
    pub ignore_discounts: bool,
}

impl MerchantOffer {
    /// `new` — все поля.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        base_cost_a: ItemCost,
        cost_b: Option<ItemCost>,
        result: ItemStack,
        uses: i32,
        max_uses: i32,
        reward_exp: bool,
        special_price_diff: i32,
        demand: i32,
        price_multiplier: f32,
        xp: i32,
        ignore_discounts: bool,
    ) -> Self {
        Self {
            base_cost_a,
            cost_b,
            result,
            uses,
            max_uses,
            reward_exp,
            special_price_diff,
            demand,
            price_multiplier,
            xp,
            ignore_discounts,
        }
    }

    /// `copy`.
    pub fn copy(&self) -> Self {
        self.clone()
    }

    /// `satisfiedBy(stackA, stackB)`.
    pub fn satisfied_by(&self, stack_a: &ItemStack, stack_b: &ItemStack) -> bool {
        if !self.base_cost_a.test(stack_a) {
            return false;
        }
        match &self.cost_b {
            Some(cost_b) => cost_b.test(stack_b),
            None => stack_b.is_empty(),
        }
    }

    /// `writeToStream`.
    fn write_to_stream(&self, buf: &mut FriendlyByteBuf) {
        ItemCost::stream_codec().encode(buf, &self.base_cost_a);
        ItemCost::optional_stream_codec().encode(buf, &self.cost_b);
        ItemStack::optional_stream_codec().encode(buf, &self.result);
        buf.write_var_int(self.uses);
        buf.write_var_int(self.max_uses);
        buf.write_bool(self.reward_exp);
        buf.write_var_int(self.special_price_diff);
        buf.write_var_int(self.demand);
        buf.write_float(self.price_multiplier);
        buf.write_var_int(self.xp);
        buf.write_bool(self.ignore_discounts);
    }

    /// `createFromStream`.
    fn create_from_stream(buf: &mut FriendlyByteBuf) -> Self {
        let base_cost_a = ItemCost::stream_codec().decode(buf);
        let cost_b = ItemCost::optional_stream_codec().decode(buf);
        let result = ItemStack::optional_stream_codec().decode(buf);
        let uses = buf.read_var_int();
        let max_uses = buf.read_var_int();
        let reward_exp = buf.read_bool();
        let special_price_diff = buf.read_var_int();
        let demand = buf.read_var_int();
        let price_multiplier = buf.read_float();
        let xp = buf.read_var_int();
        let ignore_discounts = buf.read_bool();
        Self::new(
            base_cost_a, cost_b, result, uses, max_uses, reward_exp,
            special_price_diff, demand, price_multiplier, xp, ignore_discounts,
        )
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, MerchantOffer> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, offer: &MerchantOffer| offer.write_to_stream(buf),
            |buf: &mut FriendlyByteBuf| Self::create_from_stream(buf),
        )
    }
}
