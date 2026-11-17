//! Перенос `ServerboundClientInformationPacket` + `ClientboundCustomPayloadPacket`
//! + `ServerboundCustomPayloadPacket`.

use super::custom::custom_packet_payload::{custom_payload_codec, CustomPacketPayload};
use super::custom::brand_payload;
use crate::network::chat::component_json;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use crate::server::level::client_information::ClientInformation;
use std::sync::LazyLock;

// ------------------------------------------------------------ serverbound client_information

static CLIENT_INFORMATION_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::SERVERBOUND, Identifier::with_default_namespace("client_information").unwrap())
});

/// `ServerboundClientInformationPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundClientInformationPacket {
    pub information: ClientInformation,
}

impl ServerboundClientInformationPacket {
    pub fn new(information: ClientInformation) -> Self {
        Self { information }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundClientInformationPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundClientInformationPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ServerboundClientInformationPacket::new(ClientInformation::read(buf))
            },
        )
    }
}

impl Packet for ServerboundClientInformationPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        CLIENT_INFORMATION_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        self.information.write(buf);
    }
}

// ------------------------------------------------------------ serverbound custom_payload

/// `MAX_PAYLOAD_SIZE` (serverbound).
pub const SERVERBOUND_MAX_PAYLOAD_SIZE: usize = 32767;
/// `MAX_PAYLOAD_SIZE` (clientbound gameplay/config).
pub const CLIENTBOUND_MAX_PAYLOAD_SIZE: usize = 1048576;

static SERVERBOUND_CUSTOM_PAYLOAD_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::SERVERBOUND, Identifier::with_default_namespace("custom_payload").unwrap())
});

/// `ServerboundCustomPayloadPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundCustomPayloadPacket {
    pub payload: CustomPacketPayload,
}

impl ServerboundCustomPayloadPacket {
    pub fn new(payload: CustomPacketPayload) -> Self {
        Self { payload }
    }

    /// `STREAM_CODEC` — все payload'ы → DiscardedPayload
    /// (CraftBukkit: `Collections.emptyList()` known types).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundCustomPayloadPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundCustomPayloadPacket| {
                custom_payload_codec(SERVERBOUND_MAX_PAYLOAD_SIZE, Vec::new())
                    .encode(buf, &p.payload);
            },
            |buf: &mut FriendlyByteBuf| {
                ServerboundCustomPayloadPacket::new(
                    custom_payload_codec(SERVERBOUND_MAX_PAYLOAD_SIZE, Vec::new()).decode(buf),
                )
            },
        )
    }
}

impl Packet for ServerboundCustomPayloadPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        SERVERBOUND_CUSTOM_PAYLOAD_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        custom_payload_codec(SERVERBOUND_MAX_PAYLOAD_SIZE, Vec::new())
            .encode(buf, &self.payload);
    }
}

// ------------------------------------------------------------ clientbound custom_payload

static CLIENTBOUND_CUSTOM_PAYLOAD_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("custom_payload").unwrap())
});

/// `ClientboundCustomPayloadPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundCustomPayloadPacket {
    pub payload: CustomPacketPayload,
}

impl ClientboundCustomPayloadPacket {
    pub fn new(payload: CustomPacketPayload) -> Self {
        Self { payload }
    }

    /// `GAMEPLAY_STREAM_CODEC`/`CONFIG_STREAM_CODEC` — известен `minecraft:brand`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundCustomPayloadPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundCustomPayloadPacket| {
                Self::payload_codec().encode(buf, &p.payload);
            },
            |buf: &mut FriendlyByteBuf| {
                ClientboundCustomPayloadPacket::new(Self::payload_codec().decode(buf))
            },
        )
    }

    /// Кодек payload'а с известным `BrandPayload` + DiscardedPayload-fallback.
    pub fn payload_codec() -> impl StreamCodec<FriendlyByteBuf, CustomPacketPayload> {
        custom_payload_codec(
            CLIENTBOUND_MAX_PAYLOAD_SIZE,
            vec![brand_payload::type_and_codec()],
        )
    }
}

impl Packet for ClientboundCustomPayloadPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        CLIENTBOUND_CUSTOM_PAYLOAD_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        Self::payload_codec().encode(buf, &self.payload);
    }
}
