//! Перенос `net.minecraft.world.damagesource.DamageType` — тип урона.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use crate::world::damagesource::damage_effects::DamageEffects;
use crate::world::damagesource::damage_scaling::DamageScaling;
use crate::world::damagesource::death_message_type::DeathMessageType;

/// `DamageType` — тип урона (record: msgId, scaling, exhaustion, effects, deathMessageType).
#[derive(Clone, Debug, PartialEq)]
pub struct DamageType {
    /// `id` — registry-ключ (`minecraft:in_fire`).
    pub id: Identifier,
    /// `msgId` — ключ локализации сообщения.
    pub msg_id: String,
    /// `scaling` — масштабирование.
    pub scaling: DamageScaling,
    /// `exhaustion` — истощение голода.
    pub exhaustion: f32,
    /// `effects` — эффекты.
    pub effects: DamageEffects,
    /// `deathMessageType` — тип сообщения о смерти.
    pub death_message_type: DeathMessageType,
}

impl DamageType {
    /// `new(msgId, scaling, exhaustion, effects, deathMessageType)`.
    pub fn new(
        id: Identifier,
        msg_id: impl Into<String>,
        scaling: DamageScaling,
        exhaustion: f32,
        effects: DamageEffects,
        death_message_type: DeathMessageType,
    ) -> Self {
        Self {
            id,
            msg_id: msg_id.into(),
            scaling,
            exhaustion,
            effects,
            death_message_type,
        }
    }

    /// `new(msgId, scaling, exhaustion)` — effects=HURT, deathMessageType=DEFAULT.
    pub fn simple(id: Identifier, msg_id: impl Into<String>, scaling: DamageScaling, exhaustion: f32) -> Self {
        Self::new(id, msg_id, scaling, exhaustion, DamageEffects::Hurt, DeathMessageType::Default)
    }

    /// `new(msgId, scaling, exhaustion, effects)` — deathMessageType=DEFAULT.
    pub fn with_effects(
        id: Identifier,
        msg_id: impl Into<String>,
        scaling: DamageScaling,
        exhaustion: f32,
        effects: DamageEffects,
    ) -> Self {
        Self::new(id, msg_id, scaling, exhaustion, effects, DeathMessageType::Default)
    }

    /// `new(msgId, exhaustion, effects)` — scaling=WHEN_CAUSED_BY_LIVING_NON_PLAYER.
    pub fn with_scaling_default(
        id: Identifier,
        msg_id: impl Into<String>,
        exhaustion: f32,
        effects: DamageEffects,
    ) -> Self {
        Self::new(id, msg_id, DamageScaling::WhenCausedByLivingNonPlayer, exhaustion, effects, DeathMessageType::Default)
    }

    /// `new(msgId, exhaustion)` — scaling=WHEN_CAUSED_BY_LIVING_NON_PLAYER, effects=HURT.
    pub fn exhaustion_only(id: Identifier, msg_id: impl Into<String>, exhaustion: f32) -> Self {
        Self::new(id, msg_id, DamageScaling::WhenCausedByLivingNonPlayer, exhaustion, DamageEffects::Hurt, DeathMessageType::Default)
    }

    /// `STREAM_CODEC` — `holderRegistry(DAMAGE_TYPE)`: plain VarInt registry-id.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<DamageType>> + Send + Sync + 'static {
        crate::core::holder::holder_registry_stream_codec::<DamageType>()
    }
}

impl std::fmt::Display for DamageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Eq for DamageType {}

impl std::hash::Hash for DamageType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.msg_id.hash(state);
        self.scaling.hash(state);
        self.exhaustion.to_bits().hash(state);
        self.effects.hash(state);
        self.death_message_type.hash(state);
    }
}
