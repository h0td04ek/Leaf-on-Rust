//! Перенос `net.minecraft.network.protocol.ProtocolCodecBuilder`.

use crate::network::codec::id_dispatch_codec::IdDispatchCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::network::protocol::protocol_info::PacketCodec;
use std::sync::Arc;

/// `ProtocolCodecBuilder<ByteBuf, L>` — сборка IdDispatchCodec
/// для одного направления (flow).
pub struct ProtocolCodecBuilder {
    flow: PacketFlow,
    dispatch_builder: IdDispatchCodecBuilder,
}

/// Type-erased builder (V=Box<dyn Packet>, T=PacketTypePtr).
type IdDispatchCodecBuilder =
    crate::network::codec::id_dispatch_codec::IdDispatchCodecBuilder<
        Box<dyn Packet>,
        PacketTypePtr,
    >;

impl ProtocolCodecBuilder {
    /// `new ProtocolCodecBuilder(flow)`.
    pub fn new(flow: PacketFlow) -> Self {
        Self {
            flow,
            dispatch_builder: IdDispatchCodec::builder(|p: &Box<dyn Packet>| p.packet_type()),
        }
    }

    /// `add(packetType, codec)` — codec уже в виде `PacketCodec`
    /// (обёртка конкретного пакетного кодека — см. `packet_codec_wrap`).
    pub fn add(
        &mut self,
        packet_type: PacketTypePtr,
        codec: PacketCodec,
    ) -> &mut Self {
        assert_eq!(
            packet_type.flow,
            self.flow,
            "Invalid packet flow for packet {}, expected {}",
            packet_type,
            self.flow
        );
        self.dispatch_builder.add(packet_type, codec);
        self
    }

    /// `build()` → кодек протокола.
    pub fn build(self) -> PacketCodec {
        Arc::new(self.dispatch_builder.build())
    }
}

/// Обёртка `StreamCodec<ByteBuf, P>` → `StreamCodec<ByteBuf, Box<dyn Packet>>`.
/// Аналог подтипа `Packet<? super T>` в Java-дженериках.
pub fn packet_codec_wrap<P, S>(serializer: S) -> PacketCodecWrap<P, S>
where
    P: Packet + 'static,
    S: crate::network::codec::stream_codec::StreamCodec<FriendlyByteBuf, P> + 'static,
{
    PacketCodecWrap { serializer, _p: std::marker::PhantomData }
}

/// Адаптер конкретного кодека пакета к `PacketCodec`.
pub struct PacketCodecWrap<P, S> {
    serializer: S,
    _p: std::marker::PhantomData<fn() -> P>,
}

impl<P, S> crate::network::codec::stream_codec::StreamCodec<FriendlyByteBuf, Box<dyn Packet>>
    for PacketCodecWrap<P, S>
where
    P: Packet + 'static,
    S: crate::network::codec::stream_codec::StreamCodec<FriendlyByteBuf, P>,
{
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &Box<dyn Packet>) {
        let concrete = value
            .as_any()
            .downcast_ref::<P>()
            .expect("packet type mismatch in codec");
        self.serializer.encode(buf, concrete);
    }

    fn decode(&self, buf: &mut FriendlyByteBuf) -> Box<dyn Packet> {
        Box::new(self.serializer.decode(buf))
    }
}
