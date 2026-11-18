//! Перенос `net.minecraft.world.entity.ai.attributes.Attribute` —
//! тип атрибута для wire-формата.
//!
//! `Attribute.STREAM_CODEC = ByteBufCodecs.holderRegistry(ATTRIBUTE)` —
//! на проводе только VarInt holder-id реестра (как у `MobEffect`).
//! Полная модель атрибута (диапазоны, `toColumn`, снапшоты) —
//! с портом `AttributeInstance`/`AttributeMap`.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// `Attribute.Sentiment`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sentiment {
    /// `POSITIVE`.
    Positive,
    /// `NEUTRAL`.
    Neutral,
    /// `NEGATIVE`.
    Negative,
}

impl Sentiment {
    /// `getStyle(isPositive)` — ChatFormatting-цвет.
    pub fn get_style(&self, is_positive: bool) -> &'static str {
        match self {
            Sentiment::Positive => if is_positive { "blue" } else { "red" },
            Sentiment::Neutral => "gray",
            Sentiment::Negative => if is_positive { "red" } else { "blue" },
        }
    }
}

/// `Attribute` — тип атрибута (`net.minecraft.world.entity.ai.attributes.Attribute`).
#[derive(Clone, Debug)]
pub struct Attribute {
    /// Registry-ключ атрибута (`minecraft:generic.max_health` и т.п.).
    pub id: Identifier,
    /// `defaultValue`.
    pub default_value: f64,
    /// `syncable` — синхронизируется с клиентом.
    pub syncable: bool,
    /// `descriptionId`.
    pub description_id: String,
    /// `sentiment`.
    pub sentiment: Sentiment,
}

impl Attribute {
    /// `new(descriptionId, defaultValue)`.
    pub fn new(id: Identifier, description_id: impl Into<String>, default_value: f64) -> Self {
        Self {
            id,
            default_value,
            syncable: false,
            description_id: description_id.into(),
            sentiment: Sentiment::Positive,
        }
    }

    /// `getDefaultValue()`.
    pub fn get_default_value(&self) -> f64 {
        self.default_value
    }

    /// `isClientSyncable()`.
    pub fn is_client_syncable(&self) -> bool {
        self.syncable
    }

    /// `setSyncable(syncable)`.
    pub fn set_syncable(&mut self, syncable: bool) -> &mut Self {
        self.syncable = syncable;
        self
    }

    /// `setSentiment(sentiment)`.
    pub fn set_sentiment(&mut self, sentiment: Sentiment) -> &mut Self {
        self.sentiment = sentiment;
        self
    }

    /// `sanitizeValue(value)` — базовая: без изменений.
    pub fn sanitize_value(&self, value: f64) -> f64 {
        value
    }

    /// `getDescriptionId()`.
    pub fn get_description_id(&self) -> &str {
        &self.description_id
    }

    /// `getStyle(isPositive)`.
    pub fn get_style(&self, is_positive: bool) -> &'static str {
        self.sentiment.get_style(is_positive)
    }

    /// `Attribute.STREAM_CODEC` — holderRegistry: только VarInt id.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<Attribute>> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, holder: &Holder<Attribute>| match holder {
                Holder::Reference(id) => {
                    buf.write_var_int(id + 1);
                }
                Holder::Key(_) => {
                    panic!("holderRegistry codec cannot encode unbound key")
                }
                Holder::Direct(attr) => {
                    panic!("holderRegistry codec cannot encode direct holder: {}", attr.id);
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let i = buf.read_var_int();
                assert!(i != 0, "invalid attribute holder id");
                Holder::Reference(i - 1)
            },
        )
    }
}

impl PartialEq for Attribute {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Attribute {}
impl std::hash::Hash for Attribute {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
