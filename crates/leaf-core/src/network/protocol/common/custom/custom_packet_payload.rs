//! Перенос `net.minecraft.network.protocol.common.custom.CustomPacketPayload`.
//!
//! В Java — интерфейс с реестром `TypeAndCodec`; здесь — enum-модель
//! (Brand/Discarded) + кодек с fallback на DiscardedPayload
//! (как CraftBukkit: все неизвестные payload'ы отбрасываются).
//! Leaf/Leaves protocol-core payload'ы — при порте плагин-ядра.

use super::brand_payload::BrandPayload;
use super::discarded_payload::DiscardedPayload;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use std::collections::HashMap;
use std::sync::Arc;

/// `CustomPacketPayload` — полезная нагрузка custom_payload-пакета.
#[derive(Clone, Debug, PartialEq)]
pub enum CustomPacketPayload {
    /// `BrandPayload` (`minecraft:brand`).
    Brand(BrandPayload),
    /// `DiscardedPayload` — неизвестный/отброшенный payload.
    Discarded(DiscardedPayload),
}

impl CustomPacketPayload {
    /// `type()`.
    pub fn payload_type(&self) -> Identifier {
        match self {
            CustomPacketPayload::Brand(b) => b.payload_type(),
            CustomPacketPayload::Discarded(d) => d.payload_type(),
        }
    }
}

/// Кодек известного типа: `StreamCodec<B, CustomPacketPayload>`
/// для конкретного payload-варианта.
pub type PayloadCodec = Arc<dyn Fn(&mut FriendlyByteBuf, &CustomPacketPayload) + Send + Sync>;
pub type PayloadDecoder = Arc<dyn Fn(&mut FriendlyByteBuf) -> CustomPacketPayload + Send + Sync>;

/// `CustomPacketPayload.codec(fallbackProvider, typeAndCodecs)` —
/// реестр известных типов + fallback (DiscardedPayload).
///
/// `max_size` — лимит байт payload (`MAX_PAYLOAD_SIZE`).
pub fn custom_payload_codec(
    max_size: usize,
    known: Vec<(Identifier, PayloadCodec, PayloadDecoder)>,
) -> impl StreamCodec<FriendlyByteBuf, CustomPacketPayload> {
    let known: std::sync::Arc<HashMap<Identifier, (PayloadCodec, PayloadDecoder)>> =
        std::sync::Arc::new(known.into_iter().map(|(id, e, d)| (id, (e, d))).collect());
    crate::network::codec::stream_codec::FnCodec::new(
        {
            let known = known.clone();
            move |buf: &mut FriendlyByteBuf, value: &CustomPacketPayload| {
            let id = value.payload_type();
            buf.write_identifier(&id).expect("payload id");
            if let Some((enc, _)) = known.get(&id) {
                enc(buf, value);
            } else {
                // fallback — DiscardedPayload пишет сырые байты
                match value {
                    CustomPacketPayload::Discarded(d) => { buf.write_bytes(&d.data); }
                    _ => {}
                }
            }
            }
        },
        move |buf: &mut FriendlyByteBuf| {
            let id = buf.read_identifier().expect("payload id");
            if let Some((_, dec)) = known.get(&id) {
                return dec(buf);
            }
            // fallbackProvider.create(id) → DiscardedPayload.codec(id, maxSize)
            let data = read_payload_bytes(buf, max_size);
            CustomPacketPayload::Discarded(DiscardedPayload::new(id, data))
        },
    )
}

/// Чтение payload-байт: остаток буфера в пределах max_size
/// (DiscardedPayload.codec читает `readableBytes`).
pub fn read_payload_bytes(buf: &mut FriendlyByteBuf, max_size: usize) -> Vec<u8> {
    let size = buf.readable_bytes();
    if size > max_size {
        panic!("Payload may not be larger than {} bytes", max_size);
    }
    buf.readable_bytes_vec()
}
