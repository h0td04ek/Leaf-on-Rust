//! `ClientboundLoginPacket` — вход в игру (game state).

use crate::world::level::LevelKey;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::common_player_spawn_info::CommonPlayerSpawnInfo;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::resources::resource_key::ResourceKey;
use std::collections::HashSet;

/// `ClientboundLoginPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundLoginPacket {
    /// `playerId`.
    pub player_id: i32,
    /// `hardcore`.
    pub hardcore: bool,
    /// `levels` — `Set<ResourceKey<LevelKey>>`.
    pub levels: HashSet<ResourceKey<LevelKey>>,
    /// `maxPlayers`.
    pub max_players: i32,
    /// `chunkRadius`.
    pub chunk_radius: i32,
    /// `simulationDistance`.
    pub simulation_distance: i32,
    /// `reducedDebugInfo`.
    pub reduced_debug_info: bool,
    /// `showDeathScreen`.
    pub show_death_screen: bool,
    /// `doLimitedCrafting`.
    pub do_limited_crafting: bool,
    /// `commonPlayerSpawnInfo`.
    pub common_player_spawn_info: CommonPlayerSpawnInfo,
    /// `enforcesSecureChat`.
    pub enforces_secure_chat: bool,
}

impl ClientboundLoginPacket {
    /// `new`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        player_id: i32,
        hardcore: bool,
        levels: HashSet<ResourceKey<LevelKey>>,
        max_players: i32,
        chunk_radius: i32,
        simulation_distance: i32,
        reduced_debug_info: bool,
        show_death_screen: bool,
        do_limited_crafting: bool,
        common_player_spawn_info: CommonPlayerSpawnInfo,
        enforces_secure_chat: bool,
    ) -> Self {
        Self {
            player_id, hardcore, levels, max_players, chunk_radius,
            simulation_distance, reduced_debug_info, show_death_screen,
            do_limited_crafting, common_player_spawn_info, enforces_secure_chat,
        }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundLoginPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundLoginPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let player_id = buf.read_int();
                let hardcore = buf.read_bool();
                let levels_len = buf.read_var_int();
                let mut levels = HashSet::with_capacity(levels_len.max(0) as usize);
                for _ in 0..levels_len {
                    let key = buf.read_resource_key(crate::resources::identifier::Identifier::minecraft("dimension"));
                    levels.insert(key);
                }
                let max_players = buf.read_var_int();
                let chunk_radius = buf.read_var_int();
                let simulation_distance = buf.read_var_int();
                let reduced_debug_info = buf.read_bool();
                let show_death_screen = buf.read_bool();
                let do_limited_crafting = buf.read_bool();
                let common_player_spawn_info = CommonPlayerSpawnInfo::read(buf);
                let enforces_secure_chat = buf.read_bool();
                ClientboundLoginPacket::new(
                    player_id, hardcore, levels, max_players, chunk_radius,
                    simulation_distance, reduced_debug_info, show_death_screen,
                    do_limited_crafting, common_player_spawn_info, enforces_secure_chat,
                )
            },
        )
    }
}

impl Packet for ClientboundLoginPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_login::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_int(self.player_id);
        buf.write_bool(self.hardcore);
        buf.write_var_int(self.levels.len() as i32);
        for key in &self.levels {
            let _ = buf.write_resource_key(key);
        }
        buf.write_var_int(self.max_players);
        buf.write_var_int(self.chunk_radius);
        buf.write_var_int(self.simulation_distance);
        buf.write_bool(self.reduced_debug_info);
        buf.write_bool(self.show_death_screen);
        buf.write_bool(self.do_limited_crafting);
        self.common_player_spawn_info.write(buf);
        buf.write_bool(self.enforces_secure_chat);
    }
}
