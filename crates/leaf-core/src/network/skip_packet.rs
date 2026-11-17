//! Перенос `net.minecraft.network.SkipPacketException`,
//! `SkipPacketDecoderException`, `SkipPacketEncoderException`.
//!
//! Исключения, при которых остаток буфера пропускается без декорации
//! ошибки (`IdDispatchCodec.DontDecorateException`).

use crate::network::codec::exceptions::{DecoderException, EncoderException};
use crate::network::codec::id_dispatch_codec::DontDecorateException;

/// `SkipPacketException` — маркер «пропустить пакет».
pub trait SkipPacketException {}

/// `SkipPacketDecoderException`.
#[derive(Debug)]
pub struct SkipPacketDecoderException(pub DecoderException);

impl SkipPacketException for SkipPacketDecoderException {}
impl DontDecorateException for SkipPacketDecoderException {}

impl std::fmt::Display for SkipPacketDecoderException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for SkipPacketDecoderException {}

impl SkipPacketDecoderException {
    pub fn new(message: impl Into<String>) -> Self {
        Self(DecoderException::new(message))
    }
    pub fn with_cause(cause: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self(DecoderException::with_cause(cause))
    }
}

/// `SkipPacketEncoderException`.
#[derive(Debug)]
pub struct SkipPacketEncoderException(pub EncoderException);

impl SkipPacketException for SkipPacketEncoderException {}
impl DontDecorateException for SkipPacketEncoderException {}

impl std::fmt::Display for SkipPacketEncoderException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for SkipPacketEncoderException {}

impl SkipPacketEncoderException {
    pub fn new(message: impl Into<String>) -> Self {
        Self(EncoderException::new(message))
    }
    pub fn with_cause(cause: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self(EncoderException::with_cause(cause))
    }
}

/// Проверка panic-payload на `SkipPacketException`
/// (аналог `instanceof SkipPacketException` / `DontDecorateException`).
pub fn payload_is_skip_packet(payload: &dyn std::any::Any) -> bool {
    payload.is::<SkipPacketDecoderException>() || payload.is::<SkipPacketEncoderException>()
}
