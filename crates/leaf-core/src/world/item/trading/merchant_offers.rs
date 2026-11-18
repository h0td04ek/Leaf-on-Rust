//! `MerchantOffers` — список предложений трейда.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::item_stack::ItemStack;
use crate::world::item::trading::merchant_offer::MerchantOffer;

/// `MerchantOffers` — `Vec<MerchantOffer>`.
#[derive(Clone, Debug, Default)]
pub struct MerchantOffers {
    /// `offers`.
    pub offers: Vec<MerchantOffer>,
}

impl MerchantOffers {
    /// `new`.
    pub fn new() -> Self {
        Self { offers: Vec::new() }
    }

    /// `with_capacity`.
    pub fn with_capacity(size: usize) -> Self {
        Self { offers: Vec::with_capacity(size) }
    }

    /// `getRecipeFor(stackA, stackB, index)`.
    pub fn get_recipe_for(&self, stack_a: &ItemStack, stack_b: &ItemStack, index: i32) -> Option<&MerchantOffer> {
        if index > 0 && (index as usize) < self.offers.len() {
            let offer = &self.offers[index as usize];
            return if offer.satisfied_by(stack_a, stack_b) { Some(offer) } else { None };
        }
        for offer in &self.offers {
            if offer.satisfied_by(stack_a, stack_b) {
                return Some(offer);
            }
        }
        None
    }

    /// `copy`.
    pub fn copy(&self) -> Self {
        Self { offers: self.offers.iter().map(|o| o.copy()).collect() }
    }

    /// `STREAM_CODEC` — `MerchantOffer.STREAM_CODEC.apply(ByteBufCodecs.collection(MerchantOffers::new))`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, MerchantOffers> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, offers: &MerchantOffers| {
                buf.write_var_int(offers.offers.len() as i32);
                for offer in &offers.offers {
                    MerchantOffer::stream_codec().encode(buf, offer);
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let len = buf.read_var_int();
                let mut offers = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    offers.push(MerchantOffer::stream_codec().decode(buf));
                }
                Self { offers }
            },
        )
    }
}

impl std::ops::Deref for MerchantOffers {
    type Target = Vec<MerchantOffer>;
    fn deref(&self) -> &Self::Target {
        &self.offers
    }
}

impl std::ops::DerefMut for MerchantOffers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.offers
    }
}
