//!  — регистрация game-протокола (1.21.11).

use crate::network::connection_protocol::ConnectionProtocol;
use crate::network::protocol::common::common_packet_types as common_types;
use crate::network::protocol::cookie::cookie_packet_types;
use crate::network::protocol::ping::ping_packet_types;
use crate::network::protocol::protocol_info_builder;
use crate::network::protocol::unbound_protocol::SimpleUnboundProtocol;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;

use super::clientbound_add_entity_packet::ClientboundAddEntityPacket;
use super::clientbound_animate_packet::ClientboundAnimatePacket;
use super::clientbound_award_stats_packet::ClientboundAwardStatsPacket;
use super::clientbound_block_changed_ack_packet::ClientboundBlockChangedAckPacket;
use super::clientbound_block_destruction_packet::ClientboundBlockDestructionPacket;
use super::clientbound_block_entity_data_packet::ClientboundBlockEntityDataPacket;
use super::clientbound_block_event_packet::ClientboundBlockEventPacket;
use super::clientbound_block_update_packet::ClientboundBlockUpdatePacket;
use super::clientbound_boss_event_packet::ClientboundBossEventPacket;
use super::clientbound_change_difficulty_packet::ClientboundChangeDifficultyPacket;
use super::clientbound_chunk_batch_finished_packet::ClientboundChunkBatchFinishedPacket;
use super::clientbound_chunk_batch_start_packet::ClientboundChunkBatchStartPacket;
use super::clientbound_chunks_biomes_packet::ClientboundChunksBiomesPacket;
use super::clientbound_clear_titles_packet::ClientboundClearTitlesPacket;
use super::clientbound_command_suggestions_packet::ClientboundCommandSuggestionsPacket;
use super::clientbound_container_close_packet::ClientboundContainerClosePacket;
use super::clientbound_container_set_content_packet::ClientboundContainerSetContentPacket;
use super::clientbound_container_set_data_packet::ClientboundContainerSetDataPacket;
use super::clientbound_container_set_slot_packet::ClientboundContainerSetSlotPacket;
use super::clientbound_cooldown_packet::ClientboundCooldownPacket;
use super::clientbound_custom_chat_completions_packet::ClientboundCustomChatCompletionsPacket;
use super::clientbound_damage_event_packet::ClientboundDamageEventPacket;
use super::clientbound_debug_block_value_packet::ClientboundDebugBlockValuePacket;
use super::clientbound_debug_chunk_value_packet::ClientboundDebugChunkValuePacket;
use super::clientbound_debug_entity_value_packet::ClientboundDebugEntityValuePacket;
use super::clientbound_debug_event_packet::ClientboundDebugEventPacket;
use super::clientbound_debug_sample_packet::ClientboundDebugSamplePacket;
use super::clientbound_delete_chat_packet::ClientboundDeleteChatPacket;
use super::clientbound_disguised_chat_packet::ClientboundDisguisedChatPacket;
use super::clientbound_entity_event_packet::ClientboundEntityEventPacket;
use super::clientbound_entity_position_sync_packet::ClientboundEntityPositionSyncPacket;
use super::clientbound_explode_packet::ClientboundExplodePacket;
use super::clientbound_forget_level_chunk_packet::ClientboundForgetLevelChunkPacket;
use super::clientbound_game_event_packet::ClientboundGameEventPacket;
use super::clientbound_game_test_highlight_pos_packet::ClientboundGameTestHighlightPosPacket;
use super::clientbound_hurt_animation_packet::ClientboundHurtAnimationPacket;
use super::clientbound_initialize_border_packet::ClientboundInitializeBorderPacket;
use super::clientbound_level_event_packet::ClientboundLevelEventPacket;
use super::clientbound_level_particles_packet::ClientboundLevelParticlesPacket;
use super::clientbound_light_update_packet::ClientboundLightUpdatePacket;
use super::clientbound_login_packet::ClientboundLoginPacket;
use super::clientbound_map_item_data_packet::ClientboundMapItemDataPacket;
use super::clientbound_merchant_offers_packet::ClientboundMerchantOffersPacket;
use super::clientbound_mount_screen_open_packet::ClientboundMountScreenOpenPacket;
use super::clientbound_move_entity_packet::ClientboundMoveEntityPacket;
use super::clientbound_move_minecart_packet::ClientboundMoveMinecartPacket;
use super::clientbound_move_vehicle_packet::ClientboundMoveVehiclePacket;
use super::clientbound_open_book_packet::ClientboundOpenBookPacket;
use super::clientbound_open_screen_packet::ClientboundOpenScreenPacket;
use super::clientbound_open_sign_editor_packet::ClientboundOpenSignEditorPacket;
use super::clientbound_player_abilities_packet::ClientboundPlayerAbilitiesPacket;
use super::clientbound_player_chat_packet::ClientboundPlayerChatPacket;
use super::clientbound_player_combat_end_packet::ClientboundPlayerCombatEndPacket;
use super::clientbound_player_combat_enter_packet::ClientboundPlayerCombatEnterPacket;
use super::clientbound_player_combat_kill_packet::ClientboundPlayerCombatKillPacket;
use super::clientbound_player_info_remove_packet::ClientboundPlayerInfoRemovePacket;
use super::clientbound_player_info_update_packet::ClientboundPlayerInfoUpdatePacket;
use super::clientbound_player_look_at_packet::ClientboundPlayerLookAtPacket;
use super::clientbound_player_position_packet::ClientboundPlayerPositionPacket;
use super::clientbound_player_rotation_packet::ClientboundPlayerRotationPacket;
use super::clientbound_projectile_power_packet::ClientboundProjectilePowerPacket;
use super::clientbound_recipe_book_remove_packet::ClientboundRecipeBookRemovePacket;
use super::clientbound_recipe_book_settings_packet::ClientboundRecipeBookSettingsPacket;
use super::clientbound_remove_entities_packet::ClientboundRemoveEntitiesPacket;
use super::clientbound_remove_mob_effect_packet::ClientboundRemoveMobEffectPacket;
use super::clientbound_reset_score_packet::ClientboundResetScorePacket;
use super::clientbound_respawn_packet::ClientboundRespawnPacket;
use super::clientbound_rotate_head_packet::ClientboundRotateHeadPacket;
use super::clientbound_section_blocks_update_packet::ClientboundSectionBlocksUpdatePacket;
use super::clientbound_select_advancements_tab_packet::ClientboundSelectAdvancementsTabPacket;
use super::clientbound_server_data_packet::ClientboundServerDataPacket;
use super::clientbound_set_action_bar_text_packet::ClientboundSetActionBarTextPacket;
use super::clientbound_set_border_center_packet::ClientboundSetBorderCenterPacket;
use super::clientbound_set_border_lerp_size_packet::ClientboundSetBorderLerpSizePacket;
use super::clientbound_set_border_size_packet::ClientboundSetBorderSizePacket;
use super::clientbound_set_border_warning_delay_packet::ClientboundSetBorderWarningDelayPacket;
use super::clientbound_set_border_warning_distance_packet::ClientboundSetBorderWarningDistancePacket;
use super::clientbound_set_camera_packet::ClientboundSetCameraPacket;
use super::clientbound_set_chunk_cache_center_packet::ClientboundSetChunkCacheCenterPacket;
use super::clientbound_set_chunk_cache_radius_packet::ClientboundSetChunkCacheRadiusPacket;
use super::clientbound_set_cursor_item_packet::ClientboundSetCursorItemPacket;
use super::clientbound_set_default_spawn_position_packet::ClientboundSetDefaultSpawnPositionPacket;
use super::clientbound_set_display_objective_packet::ClientboundSetDisplayObjectivePacket;
use super::clientbound_set_entity_data_packet::ClientboundSetEntityDataPacket;
use super::clientbound_set_entity_link_packet::ClientboundSetEntityLinkPacket;
use super::clientbound_set_entity_motion_packet::ClientboundSetEntityMotionPacket;
use super::clientbound_set_equipment_packet::ClientboundSetEquipmentPacket;
use super::clientbound_set_experience_packet::ClientboundSetExperiencePacket;
use super::clientbound_set_health_packet::ClientboundSetHealthPacket;
use super::clientbound_set_held_slot_packet::ClientboundSetHeldSlotPacket;
use super::clientbound_set_objective_packet::ClientboundSetObjectivePacket;
use super::clientbound_set_passengers_packet::ClientboundSetPassengersPacket;
use super::clientbound_set_player_inventory_packet::ClientboundSetPlayerInventoryPacket;
use super::serverbound_move_player_packet::{MovePlayerPosPacket, MovePlayerPosRotPacket, MovePlayerRotPacket, MovePlayerStatusOnlyPacket};
use super::clientbound_move_entity_packet::{PosPacket, PosRotPacket, RotPacket};
use super::clientbound_set_player_team_packet::ClientboundSetPlayerTeamPacket;
use super::clientbound_set_player_team_packet::TeamPacketAction;
use super::clientbound_set_score_packet::ClientboundSetScorePacket;
use super::clientbound_set_simulation_distance_packet::ClientboundSetSimulationDistancePacket;
use super::clientbound_set_subtitle_text_packet::ClientboundSetSubtitleTextPacket;
use super::clientbound_set_time_packet::ClientboundSetTimePacket;
use super::clientbound_set_title_text_packet::ClientboundSetTitleTextPacket;
use super::clientbound_set_titles_animation_packet::ClientboundSetTitlesAnimationPacket;
use super::clientbound_sound_entity_packet::ClientboundSoundEntityPacket;
use super::clientbound_sound_packet::ClientboundSoundPacket;
use super::clientbound_start_configuration_packet::ClientboundStartConfigurationPacket;
use super::clientbound_stop_sound_packet::ClientboundStopSoundPacket;
use super::clientbound_system_chat_packet::ClientboundSystemChatPacket;
use super::clientbound_tab_list_packet::ClientboundTabListPacket;
use super::clientbound_tag_query_packet::ClientboundTagQueryPacket;
use super::clientbound_take_item_entity_packet::ClientboundTakeItemEntityPacket;
use super::clientbound_teleport_entity_packet::ClientboundTeleportEntityPacket;
use super::clientbound_ticking_state_packet::ClientboundTickingStatePacket;
use super::clientbound_ticking_step_packet::ClientboundTickingStepPacket;
use super::clientbound_tracked_waypoint_packet::ClientboundTrackedWaypointPacket;
use super::clientbound_update_attributes_packet::ClientboundUpdateAttributesPacket;
use super::clientbound_update_mob_effect_packet::ClientboundUpdateMobEffectPacket;
use super::serverbound_accept_teleportation_packet::ServerboundAcceptTeleportationPacket;
use super::serverbound_block_entity_tag_query_packet::ServerboundBlockEntityTagQueryPacket;
use super::serverbound_change_difficulty_packet::ServerboundChangeDifficultyPacket;
use super::serverbound_change_game_mode_packet::ServerboundChangeGameModePacket;
use super::serverbound_chat_ack_packet::ServerboundChatAckPacket;
use super::serverbound_chat_command_packet::ServerboundChatCommandPacket;
use super::serverbound_chat_command_signed_packet::ServerboundChatCommandSignedPacket;
use super::serverbound_chat_packet::ServerboundChatPacket;
use super::serverbound_chat_session_update_packet::ServerboundChatSessionUpdatePacket;
use super::serverbound_chunk_batch_received_packet::ServerboundChunkBatchReceivedPacket;
use super::serverbound_client_command_packet::ServerboundClientCommandPacket;
use super::serverbound_client_tick_end_packet::ServerboundClientTickEndPacket;
use super::serverbound_command_suggestion_packet::ServerboundCommandSuggestionPacket;
use super::serverbound_configuration_acknowledged_packet::ServerboundConfigurationAcknowledgedPacket;
use super::serverbound_container_button_click_packet::ServerboundContainerButtonClickPacket;
use super::serverbound_container_click_packet::ServerboundContainerClickPacket;
use super::serverbound_container_close_packet::ServerboundContainerClosePacket;
use super::serverbound_container_slot_state_changed_packet::ServerboundContainerSlotStateChangedPacket;
use super::serverbound_debug_subscription_request_packet::ServerboundDebugSubscriptionRequestPacket;
use super::serverbound_edit_book_packet::ServerboundEditBookPacket;
use super::serverbound_entity_tag_query_packet::ServerboundEntityTagQueryPacket;
use super::serverbound_interact_packet::ServerboundInteractPacket;
use super::serverbound_jigsaw_generate_packet::ServerboundJigsawGeneratePacket;
use super::serverbound_lock_difficulty_packet::ServerboundLockDifficultyPacket;
use super::serverbound_move_player_packet::ServerboundMovePlayerPacketKind;
use super::serverbound_move_vehicle_packet::ServerboundMoveVehiclePacket;
use super::serverbound_paddle_boat_packet::ServerboundPaddleBoatPacket;
use super::serverbound_pick_item_from_block_packet::ServerboundPickItemFromBlockPacket;
use super::serverbound_pick_item_from_entity_packet::ServerboundPickItemFromEntityPacket;
use super::serverbound_place_recipe_packet::ServerboundPlaceRecipePacket;
use super::serverbound_player_abilities_packet::ServerboundPlayerAbilitiesPacket;
use super::serverbound_player_action_packet::ServerboundPlayerActionPacket;
use super::serverbound_player_command_packet::ServerboundPlayerCommandPacket;
use super::serverbound_player_input_packet::ServerboundPlayerInputPacket;
use super::serverbound_player_loaded_packet::ServerboundPlayerLoadedPacket;
use super::serverbound_recipe_book_change_settings_packet::ServerboundRecipeBookChangeSettingsPacket;
use super::serverbound_recipe_book_seen_recipe_packet::ServerboundRecipeBookSeenRecipePacket;
use super::serverbound_rename_item_packet::ServerboundRenameItemPacket;
use super::serverbound_seen_advancements_packet::ServerboundSeenAdvancementsPacket;
use super::serverbound_select_bundle_item_packet::ServerboundSelectBundleItemPacket;
use super::serverbound_select_trade_packet::ServerboundSelectTradePacket;
use super::serverbound_set_beacon_packet::ServerboundSetBeaconPacket;
use super::serverbound_set_carried_item_packet::ServerboundSetCarriedItemPacket;
use super::serverbound_set_command_block_packet::ServerboundSetCommandBlockPacket;
use super::serverbound_set_command_minecart_packet::ServerboundSetCommandMinecartPacket;
use super::serverbound_set_creative_mode_slot_packet::ServerboundSetCreativeModeSlotPacket;
use super::serverbound_set_jigsaw_block_packet::ServerboundSetJigsawBlockPacket;
use super::serverbound_set_structure_block_packet::ServerboundSetStructureBlockPacket;
use super::serverbound_set_test_block_packet::ServerboundSetTestBlockPacket;
use super::serverbound_sign_update_packet::ServerboundSignUpdatePacket;
use super::serverbound_swing_packet::ServerboundSwingPacket;
use super::serverbound_teleport_to_entity_packet::ServerboundTeleportToEntityPacket;
use super::serverbound_test_instance_block_action_packet::ServerboundTestInstanceBlockActionPacket;
use super::serverbound_use_item_on_packet::ServerboundUseItemOnPacket;
use super::serverbound_use_item_packet::ServerboundUseItemPacket;

/// .
pub fn serverbound_template() -> Box<dyn SimpleUnboundProtocol> {
    protocol_info_builder::serverbound_protocol(ConnectionProtocol::Play, |b| {
        b.add_packet(
            GamePacketTypes::serverbound_accept_teleportation::<ServerboundAcceptTeleportationPacket>(),
            ServerboundAcceptTeleportationPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_block_entity_tag_query::<ServerboundBlockEntityTagQueryPacket>(),
            ServerboundBlockEntityTagQueryPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_bundle_item_selected::<ServerboundSelectBundleItemPacket>(),
            ServerboundSelectBundleItemPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_change_difficulty::<ServerboundChangeDifficultyPacket>(),
            ServerboundChangeDifficultyPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_change_game_mode::<ServerboundChangeGameModePacket>(),
            ServerboundChangeGameModePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_chat_ack::<ServerboundChatAckPacket>(),
            ServerboundChatAckPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_chat_command::<ServerboundChatCommandPacket>(),
            ServerboundChatCommandPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_chat_command_signed::<ServerboundChatCommandSignedPacket>(),
            ServerboundChatCommandSignedPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_chat::<ServerboundChatPacket>(),
            ServerboundChatPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_chat_session_update::<ServerboundChatSessionUpdatePacket>(),
            ServerboundChatSessionUpdatePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_chunk_batch_received::<ServerboundChunkBatchReceivedPacket>(),
            ServerboundChunkBatchReceivedPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_client_command::<ServerboundClientCommandPacket>(),
            ServerboundClientCommandPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_client_tick_end::<ServerboundClientTickEndPacket>(),
            ServerboundClientTickEndPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_command_suggestion::<ServerboundCommandSuggestionPacket>(),
            ServerboundCommandSuggestionPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_configuration_acknowledged::<ServerboundConfigurationAcknowledgedPacket>(),
            ServerboundConfigurationAcknowledgedPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_container_button_click::<ServerboundContainerButtonClickPacket>(),
            ServerboundContainerButtonClickPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_container_click::<ServerboundContainerClickPacket>(),
            ServerboundContainerClickPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_container_close::<ServerboundContainerClosePacket>(),
            ServerboundContainerClosePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_container_slot_state_changed::<ServerboundContainerSlotStateChangedPacket>(),
            ServerboundContainerSlotStateChangedPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_debug_subscription_request::<ServerboundDebugSubscriptionRequestPacket>(),
            ServerboundDebugSubscriptionRequestPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_edit_book::<ServerboundEditBookPacket>(),
            ServerboundEditBookPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_entity_tag_query::<ServerboundEntityTagQueryPacket>(),
            ServerboundEntityTagQueryPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_interact::<ServerboundInteractPacket>(),
            ServerboundInteractPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_jigsaw_generate::<ServerboundJigsawGeneratePacket>(),
            ServerboundJigsawGeneratePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_lock_difficulty::<ServerboundLockDifficultyPacket>(),
            ServerboundLockDifficultyPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_move_player_pos::<MovePlayerPosPacket>(),
            MovePlayerPosPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_move_player_pos_rot::<MovePlayerPosRotPacket>(),
            MovePlayerPosRotPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_move_player_rot::<MovePlayerRotPacket>(),
            MovePlayerRotPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_move_player_status_only::<MovePlayerStatusOnlyPacket>(),
            MovePlayerStatusOnlyPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_move_vehicle::<ServerboundMoveVehiclePacket>(),
            ServerboundMoveVehiclePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_paddle_boat::<ServerboundPaddleBoatPacket>(),
            ServerboundPaddleBoatPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_pick_item_from_block::<ServerboundPickItemFromBlockPacket>(),
            ServerboundPickItemFromBlockPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_pick_item_from_entity::<ServerboundPickItemFromEntityPacket>(),
            ServerboundPickItemFromEntityPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_place_recipe::<ServerboundPlaceRecipePacket>(),
            ServerboundPlaceRecipePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_player_abilities::<ServerboundPlayerAbilitiesPacket>(),
            ServerboundPlayerAbilitiesPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_player_action::<ServerboundPlayerActionPacket>(),
            ServerboundPlayerActionPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_player_command::<ServerboundPlayerCommandPacket>(),
            ServerboundPlayerCommandPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_player_input::<ServerboundPlayerInputPacket>(),
            ServerboundPlayerInputPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_player_loaded::<ServerboundPlayerLoadedPacket>(),
            ServerboundPlayerLoadedPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_recipe_book_change_settings::<ServerboundRecipeBookChangeSettingsPacket>(),
            ServerboundRecipeBookChangeSettingsPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_recipe_book_seen_recipe::<ServerboundRecipeBookSeenRecipePacket>(),
            ServerboundRecipeBookSeenRecipePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_rename_item::<ServerboundRenameItemPacket>(),
            ServerboundRenameItemPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_seen_advancements::<ServerboundSeenAdvancementsPacket>(),
            ServerboundSeenAdvancementsPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_select_trade::<ServerboundSelectTradePacket>(),
            ServerboundSelectTradePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_set_beacon::<ServerboundSetBeaconPacket>(),
            ServerboundSetBeaconPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_set_carried_item::<ServerboundSetCarriedItemPacket>(),
            ServerboundSetCarriedItemPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_set_command_block::<ServerboundSetCommandBlockPacket>(),
            ServerboundSetCommandBlockPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_set_command_minecart::<ServerboundSetCommandMinecartPacket>(),
            ServerboundSetCommandMinecartPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_set_creative_mode_slot::<ServerboundSetCreativeModeSlotPacket>(),
            ServerboundSetCreativeModeSlotPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_set_jigsaw_block::<ServerboundSetJigsawBlockPacket>(),
            ServerboundSetJigsawBlockPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_set_structure_block::<ServerboundSetStructureBlockPacket>(),
            ServerboundSetStructureBlockPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_set_test_block::<ServerboundSetTestBlockPacket>(),
            ServerboundSetTestBlockPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_sign_update::<ServerboundSignUpdatePacket>(),
            ServerboundSignUpdatePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_swing::<ServerboundSwingPacket>(),
            ServerboundSwingPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_teleport_to_entity::<ServerboundTeleportToEntityPacket>(),
            ServerboundTeleportToEntityPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_test_instance_block_action::<ServerboundTestInstanceBlockActionPacket>(),
            ServerboundTestInstanceBlockActionPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_use_item_on::<ServerboundUseItemOnPacket>(),
            ServerboundUseItemOnPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::serverbound_use_item::<ServerboundUseItemPacket>(),
            ServerboundUseItemPacket::stream_codec(),
        );
    })
}

/// .
pub fn clientbound_template() -> Box<dyn SimpleUnboundProtocol> {
    protocol_info_builder::clientbound_protocol(ConnectionProtocol::Play, |b| {
        b.add_packet(
            GamePacketTypes::clientbound_add_entity::<ClientboundAddEntityPacket>(),
            ClientboundAddEntityPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_animate::<ClientboundAnimatePacket>(),
            ClientboundAnimatePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_award_stats::<ClientboundAwardStatsPacket>(),
            ClientboundAwardStatsPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_block_changed_ack::<ClientboundBlockChangedAckPacket>(),
            ClientboundBlockChangedAckPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_block_destruction::<ClientboundBlockDestructionPacket>(),
            ClientboundBlockDestructionPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_block_entity_data::<ClientboundBlockEntityDataPacket>(),
            ClientboundBlockEntityDataPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_block_event::<ClientboundBlockEventPacket>(),
            ClientboundBlockEventPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_block_update::<ClientboundBlockUpdatePacket>(),
            ClientboundBlockUpdatePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_boss_event::<ClientboundBossEventPacket>(),
            ClientboundBossEventPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_change_difficulty::<ClientboundChangeDifficultyPacket>(),
            ClientboundChangeDifficultyPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_chunk_batch_finished::<ClientboundChunkBatchFinishedPacket>(),
            ClientboundChunkBatchFinishedPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_chunk_batch_start::<ClientboundChunkBatchStartPacket>(),
            ClientboundChunkBatchStartPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_chunks_biomes::<ClientboundChunksBiomesPacket>(),
            ClientboundChunksBiomesPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_clear_titles::<ClientboundClearTitlesPacket>(),
            ClientboundClearTitlesPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_command_suggestions::<ClientboundCommandSuggestionsPacket>(),
            ClientboundCommandSuggestionsPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_container_close::<ClientboundContainerClosePacket>(),
            ClientboundContainerClosePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_container_set_content::<ClientboundContainerSetContentPacket>(),
            ClientboundContainerSetContentPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_container_set_data::<ClientboundContainerSetDataPacket>(),
            ClientboundContainerSetDataPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_container_set_slot::<ClientboundContainerSetSlotPacket>(),
            ClientboundContainerSetSlotPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_cooldown::<ClientboundCooldownPacket>(),
            ClientboundCooldownPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_custom_chat_completions::<ClientboundCustomChatCompletionsPacket>(),
            ClientboundCustomChatCompletionsPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_damage_event::<ClientboundDamageEventPacket>(),
            ClientboundDamageEventPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_debug_block_value::<ClientboundDebugBlockValuePacket>(),
            ClientboundDebugBlockValuePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_debug_chunk_value::<ClientboundDebugChunkValuePacket>(),
            ClientboundDebugChunkValuePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_debug_entity_value::<ClientboundDebugEntityValuePacket>(),
            ClientboundDebugEntityValuePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_debug_event::<ClientboundDebugEventPacket>(),
            ClientboundDebugEventPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_debug_sample::<ClientboundDebugSamplePacket>(),
            ClientboundDebugSamplePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_delete_chat::<ClientboundDeleteChatPacket>(),
            ClientboundDeleteChatPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_disguised_chat::<ClientboundDisguisedChatPacket>(),
            ClientboundDisguisedChatPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_entity_event::<ClientboundEntityEventPacket>(),
            ClientboundEntityEventPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_entity_position_sync::<ClientboundEntityPositionSyncPacket>(),
            ClientboundEntityPositionSyncPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_explode::<ClientboundExplodePacket>(),
            ClientboundExplodePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_forget_level_chunk::<ClientboundForgetLevelChunkPacket>(),
            ClientboundForgetLevelChunkPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_game_event::<ClientboundGameEventPacket>(),
            ClientboundGameEventPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_game_test_highlight_pos::<ClientboundGameTestHighlightPosPacket>(),
            ClientboundGameTestHighlightPosPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_mount_screen_open::<ClientboundMountScreenOpenPacket>(),
            ClientboundMountScreenOpenPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_hurt_animation::<ClientboundHurtAnimationPacket>(),
            ClientboundHurtAnimationPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_initialize_border::<ClientboundInitializeBorderPacket>(),
            ClientboundInitializeBorderPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_level_event::<ClientboundLevelEventPacket>(),
            ClientboundLevelEventPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_level_particles::<ClientboundLevelParticlesPacket>(),
            ClientboundLevelParticlesPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_light_update::<ClientboundLightUpdatePacket>(),
            ClientboundLightUpdatePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_login::<ClientboundLoginPacket>(),
            ClientboundLoginPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_map_item_data::<ClientboundMapItemDataPacket>(),
            ClientboundMapItemDataPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_merchant_offers::<ClientboundMerchantOffersPacket>(),
            ClientboundMerchantOffersPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_move_entity_pos::<PosPacket>(),
            PosPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_move_entity_pos_rot::<PosRotPacket>(),
            PosRotPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_move_minecart_along_track::<ClientboundMoveMinecartPacket>(),
            ClientboundMoveMinecartPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_move_entity_rot::<RotPacket>(),
            RotPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_move_vehicle::<ClientboundMoveVehiclePacket>(),
            ClientboundMoveVehiclePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_open_book::<ClientboundOpenBookPacket>(),
            ClientboundOpenBookPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_open_screen::<ClientboundOpenScreenPacket>(),
            ClientboundOpenScreenPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_open_sign_editor::<ClientboundOpenSignEditorPacket>(),
            ClientboundOpenSignEditorPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_player_abilities::<ClientboundPlayerAbilitiesPacket>(),
            ClientboundPlayerAbilitiesPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_player_chat::<ClientboundPlayerChatPacket>(),
            ClientboundPlayerChatPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_player_combat_end::<ClientboundPlayerCombatEndPacket>(),
            ClientboundPlayerCombatEndPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_player_combat_enter::<ClientboundPlayerCombatEnterPacket>(),
            ClientboundPlayerCombatEnterPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_player_combat_kill::<ClientboundPlayerCombatKillPacket>(),
            ClientboundPlayerCombatKillPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_player_info_remove::<ClientboundPlayerInfoRemovePacket>(),
            ClientboundPlayerInfoRemovePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_player_info_update::<ClientboundPlayerInfoUpdatePacket>(),
            ClientboundPlayerInfoUpdatePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_player_look_at::<ClientboundPlayerLookAtPacket>(),
            ClientboundPlayerLookAtPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_player_position::<ClientboundPlayerPositionPacket>(),
            ClientboundPlayerPositionPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_player_rotation::<ClientboundPlayerRotationPacket>(),
            ClientboundPlayerRotationPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_recipe_book_remove::<ClientboundRecipeBookRemovePacket>(),
            ClientboundRecipeBookRemovePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_recipe_book_settings::<ClientboundRecipeBookSettingsPacket>(),
            ClientboundRecipeBookSettingsPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_remove_entities::<ClientboundRemoveEntitiesPacket>(),
            ClientboundRemoveEntitiesPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_remove_mob_effect::<ClientboundRemoveMobEffectPacket>(),
            ClientboundRemoveMobEffectPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_reset_score::<ClientboundResetScorePacket>(),
            ClientboundResetScorePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_respawn::<ClientboundRespawnPacket>(),
            ClientboundRespawnPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_rotate_head::<ClientboundRotateHeadPacket>(),
            ClientboundRotateHeadPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_section_blocks_update::<ClientboundSectionBlocksUpdatePacket>(),
            ClientboundSectionBlocksUpdatePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_select_advancements_tab::<ClientboundSelectAdvancementsTabPacket>(),
            ClientboundSelectAdvancementsTabPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_server_data::<ClientboundServerDataPacket>(),
            ClientboundServerDataPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_action_bar_text::<ClientboundSetActionBarTextPacket>(),
            ClientboundSetActionBarTextPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_border_center::<ClientboundSetBorderCenterPacket>(),
            ClientboundSetBorderCenterPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_border_lerp_size::<ClientboundSetBorderLerpSizePacket>(),
            ClientboundSetBorderLerpSizePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_border_size::<ClientboundSetBorderSizePacket>(),
            ClientboundSetBorderSizePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_border_warning_delay::<ClientboundSetBorderWarningDelayPacket>(),
            ClientboundSetBorderWarningDelayPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_border_warning_distance::<ClientboundSetBorderWarningDistancePacket>(),
            ClientboundSetBorderWarningDistancePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_camera::<ClientboundSetCameraPacket>(),
            ClientboundSetCameraPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_chunk_cache_center::<ClientboundSetChunkCacheCenterPacket>(),
            ClientboundSetChunkCacheCenterPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_chunk_cache_radius::<ClientboundSetChunkCacheRadiusPacket>(),
            ClientboundSetChunkCacheRadiusPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_cursor_item::<ClientboundSetCursorItemPacket>(),
            ClientboundSetCursorItemPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_default_spawn_position::<ClientboundSetDefaultSpawnPositionPacket>(),
            ClientboundSetDefaultSpawnPositionPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_display_objective::<ClientboundSetDisplayObjectivePacket>(),
            ClientboundSetDisplayObjectivePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_entity_data::<ClientboundSetEntityDataPacket>(),
            ClientboundSetEntityDataPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_entity_link::<ClientboundSetEntityLinkPacket>(),
            ClientboundSetEntityLinkPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_entity_motion::<ClientboundSetEntityMotionPacket>(),
            ClientboundSetEntityMotionPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_equipment::<ClientboundSetEquipmentPacket>(),
            ClientboundSetEquipmentPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_experience::<ClientboundSetExperiencePacket>(),
            ClientboundSetExperiencePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_health::<ClientboundSetHealthPacket>(),
            ClientboundSetHealthPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_held_slot::<ClientboundSetHeldSlotPacket>(),
            ClientboundSetHeldSlotPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_objective::<ClientboundSetObjectivePacket>(),
            ClientboundSetObjectivePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_passengers::<ClientboundSetPassengersPacket>(),
            ClientboundSetPassengersPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_player_inventory::<ClientboundSetPlayerInventoryPacket>(),
            ClientboundSetPlayerInventoryPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_player_team::<ClientboundSetPlayerTeamPacket>(),
            ClientboundSetPlayerTeamPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_score::<ClientboundSetScorePacket>(),
            ClientboundSetScorePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_simulation_distance::<ClientboundSetSimulationDistancePacket>(),
            ClientboundSetSimulationDistancePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_subtitle_text::<ClientboundSetSubtitleTextPacket>(),
            ClientboundSetSubtitleTextPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_time::<ClientboundSetTimePacket>(),
            ClientboundSetTimePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_title_text::<ClientboundSetTitleTextPacket>(),
            ClientboundSetTitleTextPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_set_titles_animation::<ClientboundSetTitlesAnimationPacket>(),
            ClientboundSetTitlesAnimationPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_sound_entity::<ClientboundSoundEntityPacket>(),
            ClientboundSoundEntityPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_sound::<ClientboundSoundPacket>(),
            ClientboundSoundPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_start_configuration::<ClientboundStartConfigurationPacket>(),
            ClientboundStartConfigurationPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_stop_sound::<ClientboundStopSoundPacket>(),
            ClientboundStopSoundPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_system_chat::<ClientboundSystemChatPacket>(),
            ClientboundSystemChatPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_tab_list::<ClientboundTabListPacket>(),
            ClientboundTabListPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_tag_query::<ClientboundTagQueryPacket>(),
            ClientboundTagQueryPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_take_item_entity::<ClientboundTakeItemEntityPacket>(),
            ClientboundTakeItemEntityPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_teleport_entity::<ClientboundTeleportEntityPacket>(),
            ClientboundTeleportEntityPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_ticking_state::<ClientboundTickingStatePacket>(),
            ClientboundTickingStatePacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_ticking_step::<ClientboundTickingStepPacket>(),
            ClientboundTickingStepPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_update_attributes::<ClientboundUpdateAttributesPacket>(),
            ClientboundUpdateAttributesPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_update_mob_effect::<ClientboundUpdateMobEffectPacket>(),
            ClientboundUpdateMobEffectPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_projectile_power::<ClientboundProjectilePowerPacket>(),
            ClientboundProjectilePowerPacket::stream_codec(),
        );
        b.add_packet(
            GamePacketTypes::clientbound_waypoint::<ClientboundTrackedWaypointPacket>(),
            ClientboundTrackedWaypointPacket::stream_codec(),
        );
    })
}
