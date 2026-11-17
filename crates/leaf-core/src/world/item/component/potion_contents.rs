//! Перенос `net.minecraft.world.item.alchemy.PotionContents`.

use crate::core::holder::Holder;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::effect::mob_effect_instance::MobEffectInstance;
use crate::world::item::alchemy::potion::Potion;

/// `PotionContents` — (potion?, customColor?, customEffects,
/// customName?).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PotionContents {
    /// `potion`.
    pub potion: Option<Holder<Potion>>,
    /// `customColor`.
    pub custom_color: Option<i32>,
    /// `customEffects`.
    pub custom_effects: Vec<MobEffectInstance>,
    /// `customName`.
    pub custom_name: Option<String>,
}

impl PotionContents {
    /// `EMPTY`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// `BASE_POTION_COLOR`.
    pub const BASE_POTION_COLOR: i32 = -13083194;

    /// `new(potion)`.
    pub fn of_potion(potion: Holder<Potion>) -> Self {
        Self {
            potion: Some(potion),
            custom_color: None,
            custom_effects: Vec::new(),
            custom_name: None,
        }
    }

    /// `new(potion?, color?, effects, name?)`.
    pub fn new(
        potion: Option<Holder<Potion>>,
        custom_color: Option<i32>,
        custom_effects: Vec<MobEffectInstance>,
        custom_name: Option<String>,
    ) -> Self {
        Self { potion, custom_color, custom_effects, custom_name }
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.potion.is_none()
            && self.custom_color.is_none()
            && self.custom_effects.is_empty()
            && self.custom_name.is_none()
    }

    /// `STREAM_CODEC` — optional(Potion holder) + optional(INT) +
    /// list(MobEffectInstance) + optional(STRING_UTF8).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, PotionContents> + Send + Sync + 'static {
        let potion_enc = byte_buf_codecs::optional(Potion::stream_codec());
        let potion_dec = byte_buf_codecs::optional(Potion::stream_codec());
        let color_enc = byte_buf_codecs::optional(byte_buf_codecs::INT);
        let color_dec = byte_buf_codecs::optional(byte_buf_codecs::INT);
        let effects_enc = byte_buf_codecs::list(MobEffectInstance::stream_codec());
        let effects_dec = byte_buf_codecs::list(MobEffectInstance::stream_codec());
        let name_enc = byte_buf_codecs::optional(byte_buf_codecs::STRING_UTF8);
        let name_dec = byte_buf_codecs::optional(byte_buf_codecs::STRING_UTF8);
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &PotionContents| {
                potion_enc.encode(buf, &v.potion);
                color_enc.encode(buf, &v.custom_color);
                effects_enc.encode(buf, &v.custom_effects);
                name_enc.encode(buf, &v.custom_name);
            },
            move |buf: &mut FriendlyByteBuf| {
                let potion = potion_dec.decode(buf);
                let custom_color = color_dec.decode(buf);
                let custom_effects = effects_dec.decode(buf);
                let custom_name = name_dec.decode(buf);
                PotionContents::new(potion, custom_color, custom_effects, custom_name)
            },
        )
    }
}
