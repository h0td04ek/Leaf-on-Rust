//! Перенос `GamePacketTypes` — сгенерировано из Java 1.21.11.

use crate::network::protocol::packet::Packet;
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;

fn id(name: &str) -> Identifier {
    Identifier::with_default_namespace(name).unwrap()
}

/// `GamePacketTypes` — `PacketType` игровой фазы.
pub struct GamePacketTypes;

impl GamePacketTypes {
    /// `CLIENTBOUND_BUNDLE`.
    pub fn clientbound_bundle<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("bundle"))
    }
    /// `CLIENTBOUND_BUNDLE_DELIMITER`.
    pub fn clientbound_bundle_delimiter<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("bundle_delimiter"))
    }
    /// `CLIENTBOUND_ADD_ENTITY`.
    pub fn clientbound_add_entity<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("add_entity"))
    }
    /// `CLIENTBOUND_ANIMATE`.
    pub fn clientbound_animate<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("animate"))
    }
    /// `CLIENTBOUND_AWARD_STATS`.
    pub fn clientbound_award_stats<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("award_stats"))
    }
    /// `CLIENTBOUND_BLOCK_CHANGED_ACK`.
    pub fn clientbound_block_changed_ack<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("block_changed_ack"))
    }
    /// `CLIENTBOUND_BLOCK_DESTRUCTION`.
    pub fn clientbound_block_destruction<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("block_destruction"))
    }
    /// `CLIENTBOUND_BLOCK_ENTITY_DATA`.
    pub fn clientbound_block_entity_data<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("block_entity_data"))
    }
    /// `CLIENTBOUND_BLOCK_EVENT`.
    pub fn clientbound_block_event<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("block_event"))
    }
    /// `CLIENTBOUND_BLOCK_UPDATE`.
    pub fn clientbound_block_update<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("block_update"))
    }
    /// `CLIENTBOUND_BOSS_EVENT`.
    pub fn clientbound_boss_event<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("boss_event"))
    }
    /// `CLIENTBOUND_CHANGE_DIFFICULTY`.
    pub fn clientbound_change_difficulty<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("change_difficulty"))
    }
    /// `CLIENTBOUND_CHUNK_BATCH_FINISHED`.
    pub fn clientbound_chunk_batch_finished<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("chunk_batch_finished"))
    }
    /// `CLIENTBOUND_CHUNK_BATCH_START`.
    pub fn clientbound_chunk_batch_start<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("chunk_batch_start"))
    }
    /// `CLIENTBOUND_CHUNKS_BIOMES`.
    pub fn clientbound_chunks_biomes<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("chunks_biomes"))
    }
    /// `CLIENTBOUND_CLEAR_TITLES`.
    pub fn clientbound_clear_titles<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("clear_titles"))
    }
    /// `CLIENTBOUND_COMMAND_SUGGESTIONS`.
    pub fn clientbound_command_suggestions<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("command_suggestions"))
    }
    /// `CLIENTBOUND_COMMANDS`.
    pub fn clientbound_commands<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("commands"))
    }
    /// `CLIENTBOUND_CONTAINER_CLOSE`.
    pub fn clientbound_container_close<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("container_close"))
    }
    /// `CLIENTBOUND_CONTAINER_SET_CONTENT`.
    pub fn clientbound_container_set_content<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("container_set_content"))
    }
    /// `CLIENTBOUND_CONTAINER_SET_DATA`.
    pub fn clientbound_container_set_data<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("container_set_data"))
    }
    /// `CLIENTBOUND_CONTAINER_SET_SLOT`.
    pub fn clientbound_container_set_slot<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("container_set_slot"))
    }
    /// `CLIENTBOUND_COOLDOWN`.
    pub fn clientbound_cooldown<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("cooldown"))
    }
    /// `CLIENTBOUND_CUSTOM_CHAT_COMPLETIONS`.
    pub fn clientbound_custom_chat_completions<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("custom_chat_completions"))
    }
    /// `CLIENTBOUND_DAMAGE_EVENT`.
    pub fn clientbound_damage_event<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("damage_event"))
    }
    /// `CLIENTBOUND_DEBUG_BLOCK_VALUE`.
    pub fn clientbound_debug_block_value<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("debug/block_value"))
    }
    /// `CLIENTBOUND_DEBUG_CHUNK_VALUE`.
    pub fn clientbound_debug_chunk_value<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("debug/chunk_value"))
    }
    /// `CLIENTBOUND_DEBUG_ENTITY_VALUE`.
    pub fn clientbound_debug_entity_value<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("debug/entity_value"))
    }
    /// `CLIENTBOUND_DEBUG_EVENT`.
    pub fn clientbound_debug_event<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("debug/event"))
    }
    /// `CLIENTBOUND_DEBUG_SAMPLE`.
    pub fn clientbound_debug_sample<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("debug_sample"))
    }
    /// `CLIENTBOUND_DELETE_CHAT`.
    pub fn clientbound_delete_chat<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("delete_chat"))
    }
    /// `CLIENTBOUND_DISGUISED_CHAT`.
    pub fn clientbound_disguised_chat<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("disguised_chat"))
    }
    /// `CLIENTBOUND_ENTITY_EVENT`.
    pub fn clientbound_entity_event<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("entity_event"))
    }
    /// `CLIENTBOUND_ENTITY_POSITION_SYNC`.
    pub fn clientbound_entity_position_sync<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("entity_position_sync"))
    }
    /// `CLIENTBOUND_EXPLODE`.
    pub fn clientbound_explode<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("explode"))
    }
    /// `CLIENTBOUND_FORGET_LEVEL_CHUNK`.
    pub fn clientbound_forget_level_chunk<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("forget_level_chunk"))
    }
    /// `CLIENTBOUND_GAME_EVENT`.
    pub fn clientbound_game_event<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("game_event"))
    }
    /// `CLIENTBOUND_GAME_TEST_HIGHLIGHT_POS`.
    pub fn clientbound_game_test_highlight_pos<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("game_test_highlight_pos"))
    }
    /// `CLIENTBOUND_MOUNT_SCREEN_OPEN`.
    pub fn clientbound_mount_screen_open<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("mount_screen_open"))
    }
    /// `CLIENTBOUND_HURT_ANIMATION`.
    pub fn clientbound_hurt_animation<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("hurt_animation"))
    }
    /// `CLIENTBOUND_INITIALIZE_BORDER`.
    pub fn clientbound_initialize_border<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("initialize_border"))
    }
    /// `CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT`.
    pub fn clientbound_level_chunk_with_light<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("level_chunk_with_light"))
    }
    /// `CLIENTBOUND_LEVEL_EVENT`.
    pub fn clientbound_level_event<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("level_event"))
    }
    /// `CLIENTBOUND_LEVEL_PARTICLES`.
    pub fn clientbound_level_particles<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("level_particles"))
    }
    /// `CLIENTBOUND_LIGHT_UPDATE`.
    pub fn clientbound_light_update<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("light_update"))
    }
    /// `CLIENTBOUND_LOGIN`.
    pub fn clientbound_login<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("login"))
    }
    /// `CLIENTBOUND_MAP_ITEM_DATA`.
    pub fn clientbound_map_item_data<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("map_item_data"))
    }
    /// `CLIENTBOUND_MERCHANT_OFFERS`.
    pub fn clientbound_merchant_offers<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("merchant_offers"))
    }
    /// `CLIENTBOUND_MOVE_ENTITY_POS`.
    pub fn clientbound_move_entity_pos<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("move_entity_pos"))
    }
    /// `CLIENTBOUND_MOVE_ENTITY_POS_ROT`.
    pub fn clientbound_move_entity_pos_rot<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("move_entity_pos_rot"))
    }
    /// `CLIENTBOUND_MOVE_MINECART_ALONG_TRACK`.
    pub fn clientbound_move_minecart_along_track<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("move_minecart_along_track"))
    }
    /// `CLIENTBOUND_MOVE_ENTITY_ROT`.
    pub fn clientbound_move_entity_rot<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("move_entity_rot"))
    }
    /// `CLIENTBOUND_MOVE_VEHICLE`.
    pub fn clientbound_move_vehicle<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("move_vehicle"))
    }
    /// `CLIENTBOUND_OPEN_BOOK`.
    pub fn clientbound_open_book<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("open_book"))
    }
    /// `CLIENTBOUND_OPEN_SCREEN`.
    pub fn clientbound_open_screen<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("open_screen"))
    }
    /// `CLIENTBOUND_OPEN_SIGN_EDITOR`.
    pub fn clientbound_open_sign_editor<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("open_sign_editor"))
    }
    /// `CLIENTBOUND_PLACE_GHOST_RECIPE`.
    pub fn clientbound_place_ghost_recipe<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("place_ghost_recipe"))
    }
    /// `CLIENTBOUND_PLAYER_ABILITIES`.
    pub fn clientbound_player_abilities<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("player_abilities"))
    }
    /// `CLIENTBOUND_PLAYER_CHAT`.
    pub fn clientbound_player_chat<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("player_chat"))
    }
    /// `CLIENTBOUND_PLAYER_COMBAT_END`.
    pub fn clientbound_player_combat_end<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("player_combat_end"))
    }
    /// `CLIENTBOUND_PLAYER_COMBAT_ENTER`.
    pub fn clientbound_player_combat_enter<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("player_combat_enter"))
    }
    /// `CLIENTBOUND_PLAYER_COMBAT_KILL`.
    pub fn clientbound_player_combat_kill<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("player_combat_kill"))
    }
    /// `CLIENTBOUND_PLAYER_INFO_REMOVE`.
    pub fn clientbound_player_info_remove<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("player_info_remove"))
    }
    /// `CLIENTBOUND_PLAYER_INFO_UPDATE`.
    pub fn clientbound_player_info_update<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("player_info_update"))
    }
    /// `CLIENTBOUND_PLAYER_LOOK_AT`.
    pub fn clientbound_player_look_at<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("player_look_at"))
    }
    /// `CLIENTBOUND_PLAYER_POSITION`.
    pub fn clientbound_player_position<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("player_position"))
    }
    /// `CLIENTBOUND_PLAYER_ROTATION`.
    pub fn clientbound_player_rotation<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("player_rotation"))
    }
    /// `CLIENTBOUND_RECIPE_BOOK_ADD`.
    pub fn clientbound_recipe_book_add<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("recipe_book_add"))
    }
    /// `CLIENTBOUND_RECIPE_BOOK_REMOVE`.
    pub fn clientbound_recipe_book_remove<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("recipe_book_remove"))
    }
    /// `CLIENTBOUND_RECIPE_BOOK_SETTINGS`.
    pub fn clientbound_recipe_book_settings<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("recipe_book_settings"))
    }
    /// `CLIENTBOUND_REMOVE_ENTITIES`.
    pub fn clientbound_remove_entities<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("remove_entities"))
    }
    /// `CLIENTBOUND_REMOVE_MOB_EFFECT`.
    pub fn clientbound_remove_mob_effect<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("remove_mob_effect"))
    }
    /// `CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION`.
    pub fn clientbound_set_default_spawn_position<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_default_spawn_position"))
    }
    /// `CLIENTBOUND_RESPAWN`.
    pub fn clientbound_respawn<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("respawn"))
    }
    /// `CLIENTBOUND_ROTATE_HEAD`.
    pub fn clientbound_rotate_head<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("rotate_head"))
    }
    /// `CLIENTBOUND_SECTION_BLOCKS_UPDATE`.
    pub fn clientbound_section_blocks_update<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("section_blocks_update"))
    }
    /// `CLIENTBOUND_SELECT_ADVANCEMENTS_TAB`.
    pub fn clientbound_select_advancements_tab<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("select_advancements_tab"))
    }
    /// `CLIENTBOUND_SERVER_DATA`.
    pub fn clientbound_server_data<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("server_data"))
    }
    /// `CLIENTBOUND_SET_ACTION_BAR_TEXT`.
    pub fn clientbound_set_action_bar_text<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_action_bar_text"))
    }
    /// `CLIENTBOUND_SET_BORDER_CENTER`.
    pub fn clientbound_set_border_center<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_border_center"))
    }
    /// `CLIENTBOUND_SET_BORDER_LERP_SIZE`.
    pub fn clientbound_set_border_lerp_size<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_border_lerp_size"))
    }
    /// `CLIENTBOUND_SET_BORDER_SIZE`.
    pub fn clientbound_set_border_size<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_border_size"))
    }
    /// `CLIENTBOUND_SET_BORDER_WARNING_DELAY`.
    pub fn clientbound_set_border_warning_delay<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_border_warning_delay"))
    }
    /// `CLIENTBOUND_SET_BORDER_WARNING_DISTANCE`.
    pub fn clientbound_set_border_warning_distance<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_border_warning_distance"))
    }
    /// `CLIENTBOUND_SET_CAMERA`.
    pub fn clientbound_set_camera<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_camera"))
    }
    /// `CLIENTBOUND_SET_CHUNK_CACHE_CENTER`.
    pub fn clientbound_set_chunk_cache_center<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_chunk_cache_center"))
    }
    /// `CLIENTBOUND_SET_CHUNK_CACHE_RADIUS`.
    pub fn clientbound_set_chunk_cache_radius<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_chunk_cache_radius"))
    }
    /// `CLIENTBOUND_SET_DISPLAY_OBJECTIVE`.
    pub fn clientbound_set_display_objective<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_display_objective"))
    }
    /// `CLIENTBOUND_SET_ENTITY_DATA`.
    pub fn clientbound_set_entity_data<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_entity_data"))
    }
    /// `CLIENTBOUND_SET_ENTITY_LINK`.
    pub fn clientbound_set_entity_link<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_entity_link"))
    }
    /// `CLIENTBOUND_SET_ENTITY_MOTION`.
    pub fn clientbound_set_entity_motion<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_entity_motion"))
    }
    /// `CLIENTBOUND_SET_EQUIPMENT`.
    pub fn clientbound_set_equipment<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_equipment"))
    }
    /// `CLIENTBOUND_SET_EXPERIENCE`.
    pub fn clientbound_set_experience<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_experience"))
    }
    /// `CLIENTBOUND_SET_HEALTH`.
    pub fn clientbound_set_health<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_health"))
    }
    /// `CLIENTBOUND_SET_HELD_SLOT`.
    pub fn clientbound_set_held_slot<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_held_slot"))
    }
    /// `CLIENTBOUND_SET_OBJECTIVE`.
    pub fn clientbound_set_objective<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_objective"))
    }
    /// `CLIENTBOUND_SET_PASSENGERS`.
    pub fn clientbound_set_passengers<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_passengers"))
    }
    /// `CLIENTBOUND_SET_PLAYER_TEAM`.
    pub fn clientbound_set_player_team<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_player_team"))
    }
    /// `CLIENTBOUND_SET_SCORE`.
    pub fn clientbound_set_score<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_score"))
    }
    /// `CLIENTBOUND_SET_SIMULATION_DISTANCE`.
    pub fn clientbound_set_simulation_distance<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_simulation_distance"))
    }
    /// `CLIENTBOUND_SET_SUBTITLE_TEXT`.
    pub fn clientbound_set_subtitle_text<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_subtitle_text"))
    }
    /// `CLIENTBOUND_SET_TIME`.
    pub fn clientbound_set_time<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_time"))
    }
    /// `CLIENTBOUND_SET_TITLE_TEXT`.
    pub fn clientbound_set_title_text<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_title_text"))
    }
    /// `CLIENTBOUND_SET_TITLES_ANIMATION`.
    pub fn clientbound_set_titles_animation<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_titles_animation"))
    }
    /// `CLIENTBOUND_SOUND_ENTITY`.
    pub fn clientbound_sound_entity<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("sound_entity"))
    }
    /// `CLIENTBOUND_SOUND`.
    pub fn clientbound_sound<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("sound"))
    }
    /// `CLIENTBOUND_START_CONFIGURATION`.
    pub fn clientbound_start_configuration<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("start_configuration"))
    }
    /// `CLIENTBOUND_STOP_SOUND`.
    pub fn clientbound_stop_sound<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("stop_sound"))
    }
    /// `CLIENTBOUND_SYSTEM_CHAT`.
    pub fn clientbound_system_chat<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("system_chat"))
    }
    /// `CLIENTBOUND_TAB_LIST`.
    pub fn clientbound_tab_list<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("tab_list"))
    }
    /// `CLIENTBOUND_TAG_QUERY`.
    pub fn clientbound_tag_query<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("tag_query"))
    }
    /// `CLIENTBOUND_TAKE_ITEM_ENTITY`.
    pub fn clientbound_take_item_entity<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("take_item_entity"))
    }
    /// `CLIENTBOUND_TELEPORT_ENTITY`.
    pub fn clientbound_teleport_entity<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("teleport_entity"))
    }
    /// `CLIENTBOUND_TEST_INSTANCE_BLOCK_STATUS`.
    pub fn clientbound_test_instance_block_status<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("test_instance_block_status"))
    }
    /// `CLIENTBOUND_UPDATE_ADVANCEMENTS`.
    pub fn clientbound_update_advancements<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("update_advancements"))
    }
    /// `CLIENTBOUND_UPDATE_ATTRIBUTES`.
    pub fn clientbound_update_attributes<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("update_attributes"))
    }
    /// `CLIENTBOUND_UPDATE_MOB_EFFECT`.
    pub fn clientbound_update_mob_effect<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("update_mob_effect"))
    }
    /// `CLIENTBOUND_UPDATE_RECIPES`.
    pub fn clientbound_update_recipes<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("update_recipes"))
    }
    /// `CLIENTBOUND_PROJECTILE_POWER`.
    pub fn clientbound_projectile_power<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("projectile_power"))
    }
    /// `CLIENTBOUND_WAYPOINT`.
    pub fn clientbound_waypoint<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("waypoint"))
    }
    /// `SERVERBOUND_ACCEPT_TELEPORTATION`.
    pub fn serverbound_accept_teleportation<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("accept_teleportation"))
    }
    /// `SERVERBOUND_BLOCK_ENTITY_TAG_QUERY`.
    pub fn serverbound_block_entity_tag_query<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("block_entity_tag_query"))
    }
    /// `SERVERBOUND_BUNDLE_ITEM_SELECTED`.
    pub fn serverbound_bundle_item_selected<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("bundle_item_selected"))
    }
    /// `SERVERBOUND_CHANGE_DIFFICULTY`.
    pub fn serverbound_change_difficulty<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("change_difficulty"))
    }
    /// `SERVERBOUND_CHANGE_GAME_MODE`.
    pub fn serverbound_change_game_mode<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("change_game_mode"))
    }
    /// `SERVERBOUND_CHAT_ACK`.
    pub fn serverbound_chat_ack<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("chat_ack"))
    }
    /// `SERVERBOUND_CHAT_COMMAND`.
    pub fn serverbound_chat_command<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("chat_command"))
    }
    /// `SERVERBOUND_CHAT_COMMAND_SIGNED`.
    pub fn serverbound_chat_command_signed<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("chat_command_signed"))
    }
    /// `SERVERBOUND_CHAT`.
    pub fn serverbound_chat<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("chat"))
    }
    /// `SERVERBOUND_CHAT_SESSION_UPDATE`.
    pub fn serverbound_chat_session_update<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("chat_session_update"))
    }
    /// `SERVERBOUND_CHUNK_BATCH_RECEIVED`.
    pub fn serverbound_chunk_batch_received<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("chunk_batch_received"))
    }
    /// `SERVERBOUND_CLIENT_COMMAND`.
    pub fn serverbound_client_command<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("client_command"))
    }
    /// `SERVERBOUND_CLIENT_TICK_END`.
    pub fn serverbound_client_tick_end<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("client_tick_end"))
    }
    /// `SERVERBOUND_COMMAND_SUGGESTION`.
    pub fn serverbound_command_suggestion<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("command_suggestion"))
    }
    /// `SERVERBOUND_CONTAINER_BUTTON_CLICK`.
    pub fn serverbound_container_button_click<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("container_button_click"))
    }
    /// `SERVERBOUND_CONTAINER_CLICK`.
    pub fn serverbound_container_click<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("container_click"))
    }
    /// `SERVERBOUND_CONTAINER_CLOSE`.
    pub fn serverbound_container_close<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("container_close"))
    }
    /// `SERVERBOUND_CONFIGURATION_ACKNOWLEDGED`.
    pub fn serverbound_configuration_acknowledged<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("configuration_acknowledged"))
    }
    /// `SERVERBOUND_CONTAINER_SLOT_STATE_CHANGED`.
    pub fn serverbound_container_slot_state_changed<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("container_slot_state_changed"))
    }
    /// `SERVERBOUND_DEBUG_SUBSCRIPTION_REQUEST`.
    pub fn serverbound_debug_subscription_request<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("debug_subscription_request"))
    }
    /// `SERVERBOUND_RECIPE_BOOK_CHANGE_SETTINGS`.
    pub fn serverbound_recipe_book_change_settings<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("recipe_book_change_settings"))
    }
    /// `SERVERBOUND_EDIT_BOOK`.
    pub fn serverbound_edit_book<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("edit_book"))
    }
    /// `SERVERBOUND_ENTITY_TAG_QUERY`.
    pub fn serverbound_entity_tag_query<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("entity_tag_query"))
    }
    /// `SERVERBOUND_INTERACT`.
    pub fn serverbound_interact<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("interact"))
    }
    /// `SERVERBOUND_JIGSAW_GENERATE`.
    pub fn serverbound_jigsaw_generate<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("jigsaw_generate"))
    }
    /// `SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION`.
    pub fn serverbound_test_instance_block_action<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("test_instance_block_action"))
    }
    /// `SERVERBOUND_LOCK_DIFFICULTY`.
    pub fn serverbound_lock_difficulty<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("lock_difficulty"))
    }
    /// `SERVERBOUND_MOVE_PLAYER_POS`.
    pub fn serverbound_move_player_pos<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("move_player_pos"))
    }
    /// `SERVERBOUND_MOVE_PLAYER_POS_ROT`.
    pub fn serverbound_move_player_pos_rot<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("move_player_pos_rot"))
    }
    /// `SERVERBOUND_MOVE_PLAYER_ROT`.
    pub fn serverbound_move_player_rot<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("move_player_rot"))
    }
    /// `SERVERBOUND_MOVE_PLAYER_STATUS_ONLY`.
    pub fn serverbound_move_player_status_only<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("move_player_status_only"))
    }
    /// `SERVERBOUND_MOVE_VEHICLE`.
    pub fn serverbound_move_vehicle<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("move_vehicle"))
    }
    /// `SERVERBOUND_PADDLE_BOAT`.
    pub fn serverbound_paddle_boat<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("paddle_boat"))
    }
    /// `SERVERBOUND_PICK_ITEM_FROM_BLOCK`.
    pub fn serverbound_pick_item_from_block<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("pick_item_from_block"))
    }
    /// `SERVERBOUND_PICK_ITEM_FROM_ENTITY`.
    pub fn serverbound_pick_item_from_entity<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("pick_item_from_entity"))
    }
    /// `SERVERBOUND_PLACE_RECIPE`.
    pub fn serverbound_place_recipe<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("place_recipe"))
    }
    /// `SERVERBOUND_PLAYER_ABILITIES`.
    pub fn serverbound_player_abilities<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("player_abilities"))
    }
    /// `SERVERBOUND_PLAYER_ACTION`.
    pub fn serverbound_player_action<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("player_action"))
    }
    /// `SERVERBOUND_PLAYER_COMMAND`.
    pub fn serverbound_player_command<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("player_command"))
    }
    /// `SERVERBOUND_PLAYER_INPUT`.
    pub fn serverbound_player_input<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("player_input"))
    }
    /// `SERVERBOUND_PLAYER_LOADED`.
    pub fn serverbound_player_loaded<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("player_loaded"))
    }
    /// `SERVERBOUND_RECIPE_BOOK_SEEN_RECIPE`.
    pub fn serverbound_recipe_book_seen_recipe<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("recipe_book_seen_recipe"))
    }
    /// `SERVERBOUND_RENAME_ITEM`.
    pub fn serverbound_rename_item<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("rename_item"))
    }
    /// `SERVERBOUND_SEEN_ADVANCEMENTS`.
    pub fn serverbound_seen_advancements<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("seen_advancements"))
    }
    /// `SERVERBOUND_SELECT_TRADE`.
    pub fn serverbound_select_trade<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("select_trade"))
    }
    /// `SERVERBOUND_SET_BEACON`.
    pub fn serverbound_set_beacon<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("set_beacon"))
    }
    /// `SERVERBOUND_SET_CARRIED_ITEM`.
    pub fn serverbound_set_carried_item<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("set_carried_item"))
    }
    /// `SERVERBOUND_SET_COMMAND_BLOCK`.
    pub fn serverbound_set_command_block<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("set_command_block"))
    }
    /// `SERVERBOUND_SET_COMMAND_MINECART`.
    pub fn serverbound_set_command_minecart<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("set_command_minecart"))
    }
    /// `SERVERBOUND_SET_CREATIVE_MODE_SLOT`.
    pub fn serverbound_set_creative_mode_slot<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("set_creative_mode_slot"))
    }
    /// `SERVERBOUND_SET_JIGSAW_BLOCK`.
    pub fn serverbound_set_jigsaw_block<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("set_jigsaw_block"))
    }
    /// `SERVERBOUND_SET_STRUCTURE_BLOCK`.
    pub fn serverbound_set_structure_block<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("set_structure_block"))
    }
    /// `SERVERBOUND_SET_TEST_BLOCK`.
    pub fn serverbound_set_test_block<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("set_test_block"))
    }
    /// `SERVERBOUND_SIGN_UPDATE`.
    pub fn serverbound_sign_update<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("sign_update"))
    }
    /// `SERVERBOUND_SWING`.
    pub fn serverbound_swing<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("swing"))
    }
    /// `SERVERBOUND_TELEPORT_TO_ENTITY`.
    pub fn serverbound_teleport_to_entity<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("teleport_to_entity"))
    }
    /// `SERVERBOUND_USE_ITEM_ON`.
    pub fn serverbound_use_item_on<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("use_item_on"))
    }
    /// `SERVERBOUND_USE_ITEM`.
    pub fn serverbound_use_item<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::SERVERBOUND, id("use_item"))
    }
    /// `CLIENTBOUND_RESET_SCORE`.
    pub fn clientbound_reset_score<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("reset_score"))
    }
    /// `CLIENTBOUND_TICKING_STATE`.
    pub fn clientbound_ticking_state<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("ticking_state"))
    }
    /// `CLIENTBOUND_TICKING_STEP`.
    pub fn clientbound_ticking_step<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("ticking_step"))
    }
    /// `CLIENTBOUND_SET_CURSOR_ITEM`.
    pub fn clientbound_set_cursor_item<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_cursor_item"))
    }
    /// `CLIENTBOUND_SET_PLAYER_INVENTORY`.
    pub fn clientbound_set_player_inventory<P: Packet>() -> PacketType<P> {
        PacketType::new(PacketFlow::CLIENTBOUND, id("set_player_inventory"))
    }
}
