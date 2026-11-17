//! Перенос `net.minecraft.network.PacketDecoder` /
//! `PacketEncoder` — логика кодека протокола над буфером
//! (Netty-хендлеры заменены функциями над `FriendlyByteBuf`).

use crate::network::codec::exceptions::DecoderException;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::Packet;
use crate::network::protocol::protocol_info::ProtocolInfo;
use crate::network::skip_packet::payload_is_skip_packet;
use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// `PacketEncoder.MAX_PACKET_SIZE` (8 MiB до сжатия).
pub const MAX_PACKET_SIZE: usize = 8388608;
/// `PacketEncoder.MAX_FINAL_PACKET_SIZE` (3-byte varint).
pub const MAX_FINAL_PACKET_SIZE: usize = (1 << 21) - 1;

/// Результат декодирования пакета.
pub struct DecodedPacket {
    pub packet: Box<dyn Packet>,
    /// Сколько байт было прочитано до пакета.
    pub bytes_read: usize,
}

/// Ошибка декодирования.
pub enum DecodeFailure {
    /// `SkipPacketException` — остаток буфера пропущен.
    Skip,
    /// Прочая ошибка (DecoderException/IOException).
    Error(String),
}

/// `PacketDecoder.decode`.
/// Читает один пакет из `buf`; при SkipPacket-ошибке дочитывает остаток.
pub fn decode_packet(
    protocol_info: &dyn ProtocolInfo,
    buf: &mut FriendlyByteBuf,
) -> Result<DecodedPacket, DecodeFailure> {
    let bytes_before = buf.readable_bytes();
    let result = catch_unwind(AssertUnwindSafe(|| protocol_info.codec().decode(buf)));
    let packet = match result {
        Ok(p) => p,
        Err(payload) => {
            if payload_is_skip_packet(payload.as_ref()) {
                // in.skipBytes(in.readableBytes())
                let w = buf.writer_index();
                buf.set_index(w, w);
                return Err(DecodeFailure::Skip);
            }
            return Err(DecodeFailure::Error(payload_to_message(payload.as_ref())));
        }
    };
    if buf.readable_bytes() > 0 {
        return Err(DecodeFailure::Error(format!(
            "Packet {}/{} was larger than I expected, found {} bytes extra whilst reading packet {}",
            protocol_info.id().id(),
            packet.packet_type(),
            buf.readable_bytes(),
            packet.packet_type()
        )));
    }
    Ok(DecodedPacket {
        packet,
        bytes_read: bytes_before - buf.readable_bytes(),
    })
}

fn payload_to_message(payload: &(dyn Any + Send)) -> String {
    if let Some(e) = payload.downcast_ref::<DecoderException>() {
        e.message().to_string()
    } else if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown codec error".to_string()
    }
}

/// Результат кодирования пакета.
pub struct EncodedPacket {
    pub bytes: Vec<u8>,
    /// `packet.isTerminal()` — для protocol swap.
    pub terminal: bool,
}

/// Ошибка кодирования.
pub enum EncodeFailure {
    /// `SkipPacketEncoderException` — пакет пропущен (isSkippable).
    Skip,
    /// `PacketTooLargeException` (Paper) — превышен лимит размера.
    TooLarge { packet_type: String, packet_length: usize },
    /// Прочая ошибка (EncoderException).
    Error(String),
}

/// `PacketTooLargeException` (Paper) — как самостоятельная ошибка.
#[derive(Debug)]
pub struct PacketTooLargeException {
    pub packet_type: String,
    pub packet_length: usize,
}
impl std::fmt::Display for PacketTooLargeException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PacketTooLarge - {} is {}. Max is {}",
            self.packet_type, self.packet_length, MAX_PACKET_SIZE
        )
    }
}
impl std::error::Error for PacketTooLargeException {}

/// `PacketEncoder.encode` — кодирует owned-пакет в байты.
/// `packet.isSkippable()` → Skip; превышение лимита → TooLarge.
pub fn encode_packet(
    protocol_info: &dyn ProtocolInfo,
    packet: Box<dyn Packet>,
) -> Result<EncodedPacket, EncodeFailure> {
    let mut buf = FriendlyByteBuf::new();
    let packet_type = packet.packet_type().to_string();
    let is_terminal = packet.is_terminal();
    let has_fallback = packet.has_large_packet_fallback();
    let result = catch_unwind(AssertUnwindSafe(|| {
        protocol_info.codec().encode(&mut buf, &packet)
    }));
    match result {
        Ok(()) => {}
        Err(payload) => {
            if payload_is_skip_packet(payload.as_ref()) {
                return Err(EncodeFailure::Skip);
            }
            return Err(EncodeFailure::Error(payload_to_message(payload.as_ref())));
        }
    }
    let packet_length = buf.readable_bytes();
    if packet_length > MAX_PACKET_SIZE
        || (packet_length > MAX_FINAL_PACKET_SIZE && has_fallback)
    {
        return Err(EncodeFailure::TooLarge { packet_type, packet_length });
    }
    Ok(EncodedPacket { bytes: buf.readable_bytes_vec(), terminal: is_terminal })
}
