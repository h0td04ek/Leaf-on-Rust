//! Перенос `net.minecraft.network.protocol.ProtocolInfoBuilder`.
//!
//! Дженерики Java `<T extends PacketListener, B extends ByteBuf, C>`
//! свёрнуты: listener тип не нужен (пакеты type-erased), `B` =
//! `FriendlyByteBuf`, `C` — параметр билдера (контекст протокола).

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::connection_protocol::ConnectionProtocol;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::bundler_info::{BundlerInfo, CreateForPacketBundlerInfo};
use crate::network::protocol::codec_modifier::{CodecModifier, CodecModifierPtr};
use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::network::protocol::protocol_codec_builder::{
    packet_codec_wrap, ProtocolCodecBuilder,
};
use crate::network::protocol::protocol_info::{
    PacketCodec, ProtocolInfo, ProtocolInfoDetails, ProtocolInfoDetailsProvider,
    SharedProtocolInfoDetails,
};
use crate::network::protocol::unbound_protocol::{SimpleUnboundProtocol, UnboundProtocol};
use std::sync::Arc;

/// `ProtocolInfoBuilder.CodecEntry` — запись пакетного кодека.
struct CodecEntry<C> {
    type_: PacketTypePtr,
    /// Кодек уже обёрнутый до `PacketCodec` (Box<dyn Packet>).
    serializer: PacketCodec,
    modifier: Option<CodecModifierPtr<C>>,
}

/// `ProtocolInfoBuilder<T, B, C>`.
pub struct ProtocolInfoBuilder<C> {
    protocol: ConnectionProtocol,
    flow: PacketFlow,
    codecs: Vec<CodecEntry<C>>,
    bundler_info: Option<Arc<dyn BundlerInfo>>,
}

impl<C: 'static> ProtocolInfoBuilder<C> {
    /// `new ProtocolInfoBuilder(protocol, flow)`.
    pub fn new(protocol: ConnectionProtocol, flow: PacketFlow) -> Self {
        Self { protocol, flow, codecs: Vec::new(), bundler_info: None }
    }

    /// `addPacket(type, serializer)` — serializer: StreamCodec пакета P.
    pub fn add_packet<P, S>(&mut self, type_: PacketType<P>, serializer: S) -> &mut Self
    where
        P: Packet + 'static,
        S: StreamCodec<FriendlyByteBuf, P> + 'static,
    {
        self.codecs.push(CodecEntry {
            type_: PacketTypePtr::new(type_.flow, type_.id.clone()),
            serializer: Arc::new(packet_codec_wrap(serializer)),
            modifier: None,
        });
        self
    }

    /// `addPacket(type, serializer, modifier)`.
    pub fn add_packet_with_modifier<P, S, M>(
        &mut self,
        type_: PacketType<P>,
        serializer: S,
        modifier: M,
    ) -> &mut Self
    where
        P: Packet + 'static,
        S: StreamCodec<FriendlyByteBuf, P> + 'static,
        M: CodecModifier<C> + 'static,
    {
        self.codecs.push(CodecEntry {
            type_: PacketTypePtr::new(type_.flow, type_.id.clone()),
            serializer: Arc::new(packet_codec_wrap(serializer)),
            modifier: Some(Arc::new(modifier)),
        });
        self
    }

    /// `withBundlePacket(type, bundler, packet)`.
    /// `bundler`: Vec<sub-packets> → bundle packet.
    /// `delimiter_type`/`delimiter_factory`: тип и фабрика delimiter'а
    /// (в Java — `StreamCodec.unit(packet)` + `packet.type()`).
    pub fn with_bundle_packet(
        &mut self,
        bundle_type: PacketTypePtr,
        bundler: impl Fn(Vec<Box<dyn Packet>>) -> Box<dyn Packet> + Send + Sync + 'static,
        delimiter_type: PacketTypePtr,
        delimiter_factory: impl Fn() -> Box<dyn Packet> + Send + Sync + 'static,
    ) -> &mut Self {
        let factory: Arc<dyn Fn() -> Box<dyn Packet> + Send + Sync> = Arc::new(delimiter_factory);
        let codec_factory = factory.clone();
        self.codecs.push(CodecEntry {
            type_: delimiter_type.clone(),
            serializer: Arc::new(BundleDelimiterCodec { factory: codec_factory }),
            modifier: None,
        });
        let bundler_factory = factory.clone();
        self.bundler_info = Some(Arc::new(CreateForPacketBundlerInfo::new_with_delimiter_type(
            bundle_type,
            delimiter_type,
            bundler,
            move || bundler_factory(),
        )));
        self
    }

    /// `buildPacketCodec` — сборка IdDispatchCodec для flow.
    fn build_packet_codec(&self, context: &C) -> PacketCodec {
        let mut builder = ProtocolCodecBuilder::new(self.flow);
        for entry in &self.codecs {
            let mut codec = entry.serializer.clone();
            if let Some(modifier) = &entry.modifier {
                codec = modifier.apply(codec, context);
            }
            builder.add(entry.type_.clone(), codec);
        }
        builder.build()
    }

    /// `buildDetails` — метаданные протокола.
    fn build_details(&self) -> SharedProtocolInfoDetails {
        Arc::new(ProtocolInfoBuilderDetails {
            protocol: self.protocol,
            flow: self.flow,
            types: self.codecs.iter().map(|e| e.type_.clone()).collect(),
        })
    }

    /// `buildUnbound()` — UnboundProtocol с контекстом.
    pub fn build_unbound(&self) -> Box<dyn UnboundProtocol<C>> {
        Box::new(UnboundImpl {
            protocol: self.protocol,
            flow: self.flow,
            codecs: self
                .codecs
                .iter()
                .map(|e| (e.type_.clone(), e.serializer.clone(), e.modifier.clone()))
                .collect(),
            bundler: self.bundler_info.clone(),
            details: self.build_details(),
        })
    }

    /// `buildUnbound(Unit)` — SimpleUnboundProtocol без контекста.
    /// Требует `C: Default` (аналог `Unit.INSTANCE`).
    pub fn build_unbound_simple(&self) -> Box<dyn SimpleUnboundProtocol>
    where
        C: Default,
    {
        Box::new(SimpleUnboundImpl { unbound: self.build_unbound() })
    }
}

/// `StreamCodec.unit(packet)` для delimiter'а: encode — ничего,
/// decode — новый экземпляр из фабрики.
struct BundleDelimiterCodec {
    factory: Arc<dyn Fn() -> Box<dyn Packet> + Send + Sync>,
}

impl StreamCodec<FriendlyByteBuf, Box<dyn Packet>> for BundleDelimiterCodec {
    fn encode(&self, _buf: &mut FriendlyByteBuf, _value: &Box<dyn Packet>) {}
    fn decode(&self, _buf: &mut FriendlyByteBuf) -> Box<dyn Packet> {
        (self.factory)()
    }
}

/// Type-erased записи для UnboundImpl.
type ErasedCodecEntry<C> = (PacketTypePtr, PacketCodec, Option<CodecModifierPtr<C>>);

struct UnboundImpl<C: 'static> {
    protocol: ConnectionProtocol,
    flow: PacketFlow,
    codecs: Vec<ErasedCodecEntry<C>>,
    bundler: Option<Arc<dyn BundlerInfo>>,
    details: SharedProtocolInfoDetails,
}

struct BoundProtocolInfo {
    id: ConnectionProtocol,
    flow: PacketFlow,
    codec: PacketCodec,
    bundler_info: Option<Arc<dyn BundlerInfo>>,
    details: SharedProtocolInfoDetails,
}

impl ProtocolInfo for BoundProtocolInfo {
    fn id(&self) -> ConnectionProtocol {
        self.id
    }
    fn flow(&self) -> PacketFlow {
        self.flow
    }
    fn codec(&self) -> &PacketCodec {
        &self.codec
    }
    fn bundler_info(&self) -> Option<&dyn BundlerInfo> {
        self.bundler_info.as_deref()
    }
    fn details(&self) -> &dyn ProtocolInfoDetails {
        self.details.as_ref()
    }
}

impl<C: 'static> UnboundProtocol<C> for UnboundImpl<C> {
    fn bind(&self, context: &C) -> Box<dyn ProtocolInfo> {
        let mut builder = ProtocolCodecBuilder::new(self.flow);
        for (type_, serializer, modifier) in &self.codecs {
            let mut codec = serializer.clone();
            if let Some(m) = modifier {
                codec = m.apply(codec, context);
            }
            builder.add(type_.clone(), codec);
        }
        Box::new(BoundProtocolInfo {
            id: self.protocol,
            flow: self.flow,
            codec: builder.build(),
            bundler_info: self.bundler.clone(),
            details: self.details.clone(),
        })
    }
}

impl<C: 'static> ProtocolInfoDetailsProvider for UnboundImpl<C> {
    fn details(&self) -> SharedProtocolInfoDetails {
        self.details.clone()
    }
}

/// `ProtocolInfo.Details` — реализация билдера.
struct ProtocolInfoBuilderDetails {
    protocol: ConnectionProtocol,
    flow: PacketFlow,
    types: Vec<PacketTypePtr>,
}

impl ProtocolInfoDetails for ProtocolInfoBuilderDetails {
    fn id(&self) -> ConnectionProtocol {
        self.protocol
    }
    fn flow(&self) -> PacketFlow {
        self.flow
    }
    fn list_packets(&self, visitor: &mut dyn FnMut(PacketType<Box<dyn Packet>>, usize)) {
        for (i, t) in self.types.iter().enumerate() {
            visitor(t.clone(), i);
        }
    }
}

/// SimpleUnbound-обёртка: bind() использует C::default() как контекст.
struct SimpleUnboundImpl<C: 'static + Default> {
    unbound: Box<dyn UnboundProtocol<C>>,
}

impl<C: 'static + Default> SimpleUnboundProtocol for SimpleUnboundImpl<C> {
    fn bind(&self) -> Box<dyn ProtocolInfo> {
        self.unbound.bind(&C::default())
    }
}

impl<C: 'static + Default> ProtocolInfoDetailsProvider for SimpleUnboundImpl<C> {
    fn details(&self) -> SharedProtocolInfoDetails {
        self.unbound.details()
    }
}

// ---------------------------------------------------------- статические фабрики

/// `serverboundProtocol(protocol, packetAdder)` — контекст `()`.
pub fn serverbound_protocol<F>(protocol: ConnectionProtocol, adder: F) -> Box<dyn SimpleUnboundProtocol>
where
    F: FnOnce(&mut ProtocolInfoBuilder<()>),
{
    let mut b = ProtocolInfoBuilder::new(protocol, PacketFlow::SERVERBOUND);
    adder(&mut b);
    b.build_unbound_simple()
}

/// `clientboundProtocol(protocol, packetAdder)` — контекст `()`.
pub fn clientbound_protocol<F>(protocol: ConnectionProtocol, adder: F) -> Box<dyn SimpleUnboundProtocol>
where
    F: FnOnce(&mut ProtocolInfoBuilder<()>),
{
    let mut b = ProtocolInfoBuilder::new(protocol, PacketFlow::CLIENTBOUND);
    adder(&mut b);
    b.build_unbound_simple()
}

/// `contextServerboundProtocol(protocol, packetAdder)`.
pub fn context_serverbound_protocol<C, F>(
    protocol: ConnectionProtocol,
    adder: F,
) -> Box<dyn UnboundProtocol<C>>
where
    C: 'static,
    F: FnOnce(&mut ProtocolInfoBuilder<C>),
{
    let mut b = ProtocolInfoBuilder::new(protocol, PacketFlow::SERVERBOUND);
    adder(&mut b);
    b.build_unbound()
}

/// `contextClientboundProtocol(protocol, packetAdder)`.
pub fn context_clientbound_protocol<C, F>(
    protocol: ConnectionProtocol,
    adder: F,
) -> Box<dyn UnboundProtocol<C>>
where
    C: 'static,
    F: FnOnce(&mut ProtocolInfoBuilder<C>),
{
    let mut b = ProtocolInfoBuilder::new(protocol, PacketFlow::CLIENTBOUND);
    adder(&mut b);
    b.build_unbound()
}
