//! Перенос `net.minecraft.network.PacketBundlePacker` /
//! `PacketBundleUnpacker` — логика bundle-трансформации пакетов
//! (Netty-хендлеры заменены конечными автоматами над потоком пакетов).

use crate::network::protocol::bundler_info::{BundlerInfo, PacketBundler};
use crate::network::protocol::packet::Packet;
use std::sync::Arc;

/// Ошибка bundle-декодирования (`DecoderException` в Java).
#[derive(Debug)]
pub struct BundlePacketError(pub String);
impl std::fmt::Display for BundlePacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for BundlePacketError {}

/// `verifyNonTerminalPacket`.
fn verify_non_terminal(packet: &dyn Packet) -> Result<(), BundlePacketError> {
    if packet.is_terminal() {
        return Err(BundlePacketError("Terminal message received in bundle".into()));
    }
    Ok(())
}

/// `PacketBundlePacker` — входящая сторона: склеивает
/// delimiter + packets + delimiter в bundle-пакет.
pub struct PacketBundlePacker {
    bundler_info: Arc<dyn BundlerInfo>,
    current_bundler: Option<Box<dyn PacketBundler>>,
}

impl PacketBundlePacker {
    /// `new PacketBundlePacker(bundlerInfo)`.
    pub fn new(bundler_info: Arc<dyn BundlerInfo>) -> Self {
        Self { bundler_info, current_bundler: None }
    }

    /// `decode(packet)` → Vec исходящих пакетов.
    /// Пустой Vec = пакет поглощён bundler'ом (delimiter/часть bundle).
    /// Caller проверяет `is_terminal` на возвращённых пакетах —
    /// аналог `pipeline.remove` в Java.
    pub fn decode(
        &mut self,
        packet: Box<dyn Packet>,
    ) -> Result<Vec<Box<dyn Packet>>, BundlePacketError> {
        if let Some(bundler) = self.current_bundler.as_mut() {
            verify_non_terminal(packet.as_ref())?;
            if let Some(bundle) = bundler.add_packet(packet) {
                self.current_bundler = None;
                return Ok(vec![bundle]);
            }
            return Ok(Vec::new());
        }
        if let Some(bundler) = self
            .bundler_info
            .start_packet_bundling(packet.packet_type())
        {
            verify_non_terminal(packet.as_ref())?;
            self.current_bundler = Some(bundler);
            return Ok(Vec::new());
        }
        Ok(vec![packet])
    }
}

/// `PacketBundleUnpacker` — исходящая сторона: раскрывает bundle
/// в delimiter + sub-packets + delimiter.
pub struct PacketBundleUnpacker {
    bundler_info: Arc<dyn BundlerInfo>,
}

impl PacketBundleUnpacker {
    /// `new PacketBundleUnpacker(bundlerInfo)`.
    pub fn new(bundler_info: Arc<dyn BundlerInfo>) -> Self {
        Self { bundler_info }
    }

    /// `encode(packet)` → Vec пакетов для отправки.
    /// Caller проверяет `is_terminal` на возвращённых пакетах.
    pub fn encode(&self, packet: Box<dyn Packet>) -> Vec<Box<dyn Packet>> {
        let mut out = Vec::new();
        self.bundler_info.unbundle_packet(packet, &mut out);
        out
    }
}
