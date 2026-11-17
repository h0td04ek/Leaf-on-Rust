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
use super::clientbound_set_cursor_item_packet::ClientboundSetCursorItemPacket;
use super::clientbound_set_player_inventory_packet::ClientboundSetPlayerInventoryPacket;
use super::clientbound_set_equipment_packet::ClientboundSetEquipmentPacket;
use super::clientbound_test_instance_block_status::ClientboundTestInstanceBlockStatus;
use super::clientbound_level_particles_packet::ClientboundLevelParticlesPacket;
use super::clientbound_explode_packet::ClientboundExplodePacket;
use super::clientbound_add_entity_packet::ClientboundAddEntityPacket;
use super::clientbound_container_set_content_packet::ClientboundContainerSetContentPacket;
use super::clientbound_container_set_slot_packet::ClientboundContainerSetSlotPacket;
use super::clientbound_block_entity_data_packet::ClientboundBlockEntityDataPacket;
use super::clientbound_login_packet::ClientboundLoginPacket;

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
    fn handle_set_cursor_item(&mut self, _packet: &ClientboundSetCursorItemPacket) {}
    fn handle_set_player_inventory(&mut self, _packet: &ClientboundSetPlayerInventoryPacket) {}
    fn handle_set_equipment(&mut self, _packet: &ClientboundSetEquipmentPacket) {}
    fn handle_test_instance_block_status(&mut self, _packet: &ClientboundTestInstanceBlockStatus) {}
    fn handle_level_particles(&mut self, _packet: &ClientboundLevelParticlesPacket) {}
    fn handle_explosion(&mut self, _packet: &ClientboundExplodePacket) {}
    fn handle_add_entity(&mut self, _packet: &ClientboundAddEntityPacket) {}
    fn handle_container_content(&mut self, _packet: &ClientboundContainerSetContentPacket) {}
    fn handle_container_set_slot(&mut self, _packet: &ClientboundContainerSetSlotPacket) {}
    fn handle_block_entity_data(&mut self, _packet: &ClientboundBlockEntityDataPacket) {}
    fn handle_login(&mut self, _packet: &ClientboundLoginPacket) {}
}
