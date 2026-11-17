//! Перенос `net.minecraft.world.item.component.InstrumentComponent`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::either_holder::EitherHolder;
use crate::world::item::instrument::Instrument;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;

/// `InstrumentComponent` — `EitherHolder<Instrument>`.
#[derive(Clone, Debug, PartialEq)]
pub struct InstrumentComponent {
    /// `instrument`.
    pub instrument: EitherHolder<Instrument>,
}

impl InstrumentComponent {
    /// `new(holder)`.
    pub fn new(instrument: EitherHolder<Instrument>) -> Self {
        Self { instrument }
    }

    /// `new(key)`.
    pub fn of_key(key: ResourceKey<Instrument>) -> Self {
        Self { instrument: EitherHolder::of_key(key) }
    }

    /// `STREAM_CODEC` — `EitherHolder.streamCodec(INSTRUMENT,
    /// Instrument.STREAM_CODEC)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, InstrumentComponent> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, v: &InstrumentComponent| {
                EitherHolder::stream_codec(
                    Identifier::minecraft("instrument"),
                    Instrument::stream_codec(),
                )
                .encode(buf, &v.instrument);
            },
            |buf: &mut FriendlyByteBuf| {
                let instrument = EitherHolder::stream_codec(
                    Identifier::minecraft("instrument"),
                    Instrument::stream_codec(),
                )
                .decode(buf);
                InstrumentComponent::new(instrument)
            },
        )
    }
}
