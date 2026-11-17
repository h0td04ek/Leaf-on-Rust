//! Перенос `net.minecraft.network.protocol.common.custom.BrandPayload`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::common::custom::custom_packet_payload::{
    custom_payload_codec, CustomPacketPayload, PayloadCodec, PayloadDecoder,
};
use crate::resources::identifier::Identifier;
use std::sync::Arc;

/// `BrandPayload` — `minecraft:brand`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrandPayload {
    pub brand: String,
}

impl BrandPayload {
    /// `TYPE` — `minecraft:brand`.
    pub fn payload_type(&self) -> Identifier {
        brand_type()
    }
}

fn brand_type() -> Identifier {
    Identifier::with_default_namespace("brand").unwrap()
}

fn brand_encode() -> PayloadCodec {
    Arc::new(|buf: &mut FriendlyByteBuf, value: &CustomPacketPayload| {
        if let CustomPacketPayload::Brand(b) = value {
            buf.write_utf(&b.brand).expect("brand too long");
        }
    })
}

fn brand_decode() -> PayloadDecoder {
    Arc::new(|buf: &mut FriendlyByteBuf| {
        CustomPacketPayload::Brand(BrandPayload {
            brand: buf.read_utf().expect("invalid brand"),
        })
    })
}

/// `BrandPayload.STREAM_CODEC` как member-type-and-codec для
/// `custom_payload_codec` (encode/decode на уровне CustomPacketPayload).
pub fn type_and_codec() -> (Identifier, PayloadCodec, PayloadDecoder) {
    (brand_type(), brand_encode(), brand_decode())
}

#[allow(dead_code)]
fn _codec_type_check(_: &dyn StreamCodec<FriendlyByteBuf, CustomPacketPayload>) {}
