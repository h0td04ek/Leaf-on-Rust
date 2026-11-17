//! Перенос `ClientGamePacketListener` (+ `ServerPacketListener`-аналоги).
// Методы добавляются по мере порта пакетов — сигнатуры как в Java
// (default no-op, чтобы не ломать существующие impl'ы).

use crate::network::connection_protocol::ConnectionProtocol;
use crate::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::network::protocol::ping::client_pong_packet_listener::ClientPongPacketListener;
use super::clientbound_change_difficulty_packet::ClientboundChangeDifficultyPacket;
use super::clientbound_chunk_batch_finished_packet::ClientboundChunkBatchFinishedPacket;
use super::clientbound_chunk_batch_start_packet::ClientboundChunkBatchStartPacket;
use super::clientbound_clear_titles_packet::ClientboundClearTitlesPacket;
use super::clientbound_container_close_packet::ClientboundContainerClosePacket;
use super::clientbound_cooldown_packet::ClientboundCooldownPacket;
use super::clientbound_debug_sample_packet::ClientboundDebugSamplePacket;
use super::clientbound_forget_level_chunk_packet::ClientboundForgetLevelChunkPacket;

/// `ClientGamePacketListener`.
pub trait ClientGamePacketListener:
    ClientCommonPacketListener + ClientPongPacketListener
{
    /// `protocol()` — `ConnectionProtocol.PLAY`.
    fn protocol(&self) -> ConnectionProtocol {
        ConnectionProtocol::Play
    }

    fn handle_change_difficulty(&mut self, _packet: &ClientboundChangeDifficultyPacket) {}
    fn handle_chunk_batch_start(&mut self, _packet: &ClientboundChunkBatchStartPacket) {}
    fn handle_chunk_batch_finished(&mut self, _packet: &ClientboundChunkBatchFinishedPacket) {}
    fn handle_titles_clear(&mut self, _packet: &ClientboundClearTitlesPacket) {}
    fn handle_container_close(&mut self, _packet: &ClientboundContainerClosePacket) {}
    fn handle_item_cooldown(&mut self, _packet: &ClientboundCooldownPacket) {}
    fn handle_debug_sample(&mut self, _packet: &ClientboundDebugSamplePacket) {}
    fn handle_forget_level_chunk(&mut self, _packet: &ClientboundForgetLevelChunkPacket) {}
}
