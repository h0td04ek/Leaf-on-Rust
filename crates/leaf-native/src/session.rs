use std::collections::HashSet;
use std::io;
use std::net::TcpStream;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use leaf_core::authlib::game_profile::GameProfile;
use leaf_core::core::block_pos::BlockPos;
use leaf_core::core::holder::Holder;
use leaf_core::core::shared_constants::CURRENT_PROTOCOL_VERSION;
use leaf_core::entity::position_move_rotation::PositionMoveRotation;
use leaf_core::network::chat::component::Component;
use leaf_core::network::protocol::common::{
    clientbound_disconnect_packet::ClientboundDisconnectPacket,
    clientbound_keep_alive_packet::ClientboundKeepAlivePacket,
};
use leaf_core::network::protocol::configuration::{
    clientbound_finish_configuration_packet::CLIENTBOUND_FINISH_CONFIGURATION_PACKET,
    clientbound_update_enabled_features_packet::ClientboundUpdateEnabledFeaturesPacket,
    configuration_protocols, ClientboundSelectKnownPacks,
};
use leaf_core::network::protocol::game::{
    clientbound_commands_packet::ClientboundCommandsPacket,
    clientbound_game_event_packet::{ClientboundGameEventPacket, LEVEL_CHUNKS_LOAD_START},
    clientbound_login_packet::ClientboundLoginPacket,
    clientbound_player_abilities_packet::ClientboundPlayerAbilitiesPacket,
    clientbound_player_info_update_packet::{ClientboundPlayerInfoUpdatePacket, PlayerInfoEntry},
    clientbound_player_position_packet::ClientboundPlayerPositionPacket,
    clientbound_set_default_spawn_position_packet::ClientboundSetDefaultSpawnPositionPacket,
    clientbound_set_health_packet::ClientboundSetHealthPacket,
    common_player_spawn_info::CommonPlayerSpawnInfo,
    game_protocols,
    serverbound_command_suggestion_packet::ServerboundCommandSuggestionPacket,
};
use leaf_core::network::protocol::handshake::handshake_protocols;
use leaf_core::network::protocol::login::{
    clientbound_login_disconnect_packet::ClientboundLoginDisconnectPacket,
    clientbound_login_finished_packet::ClientboundLoginFinishedPacket, login_protocols,
};
use leaf_core::network::protocol::ping::clientbound_pong_response_packet::ClientboundPongResponsePacket;
use leaf_core::network::protocol::status::{
    clientbound_status_response_packet::ClientboundStatusResponsePacket,
    server_status::{Players, ServerStatus, Version},
    status_protocols,
};
use leaf_core::resources::Identifier;
use leaf_core::server::packs::repository::KnownPack;
use leaf_core::util::uuid::Uuid;
use leaf_core::world::level::{game_type::GameType, LevelKey, RespawnData};
use leaf_core::world::phys::vec3::Vec3;

use crate::audience::Audience;
use crate::chunk_stream::{
    client_view_distance, ChunkStream, MAX_VIEW_DISTANCE, SIMULATION_DISTANCE,
};
use crate::commands::Commands;
use crate::gameplay::{Gameplay, PlayContext};
use crate::items::Items;
use crate::registries::Registries;
use crate::wire::{invalid, Cursor, PacketIds, Wire};
use crate::world::{FlatWorld, SPAWN_Y};

pub const MAX_PLAYERS: usize = 8;
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(8);
const KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(20);
const TELEPORT_ID: i32 = 1;

pub struct Server {
    handshake: PacketIds,
    status_in: PacketIds,
    status_out: PacketIds,
    login_in: PacketIds,
    login_out: PacketIds,
    configuration_in: PacketIds,
    configuration_out: PacketIds,
    play_in: PacketIds,
    play_out: PacketIds,
    registries: Registries,
    commands: Commands,
    audience: Audience,
    flat_world: FlatWorld,
    sockets: AtomicUsize,
    players: Mutex<HashSet<Uuid>>,
    online: AtomicUsize,
    next_entity: AtomicI32,
}

impl Server {
    pub fn new() -> Self {
        Items::vanilla();
        let registries = Registries::load();
        Self {
            handshake: PacketIds::from_protocol(handshake_protocols::serverbound()),
            status_in: PacketIds::from_protocol(status_protocols::serverbound()),
            status_out: PacketIds::from_protocol(status_protocols::clientbound()),
            login_in: PacketIds::from_protocol(login_protocols::serverbound()),
            login_out: PacketIds::from_protocol(login_protocols::clientbound()),
            configuration_in: PacketIds::from_protocol(configuration_protocols::serverbound()),
            configuration_out: PacketIds::from_protocol(configuration_protocols::clientbound()),
            play_in: PacketIds::from_protocol(game_protocols::serverbound_template().bind()),
            play_out: PacketIds::from_protocol(game_protocols::clientbound_template().bind()),
            flat_world: FlatWorld::new(registries.plains_id),
            registries,
            commands: Commands::new(),
            audience: Audience::default(),
            sockets: AtomicUsize::new(0),
            players: Mutex::new(HashSet::new()),
            online: AtomicUsize::new(0),
            next_entity: AtomicI32::new(1),
        }
    }

    pub fn open_session(self: &Arc<Self>) -> Option<SocketSlot> {
        // Leave capacity for handshakes and status requests when the world is full.
        if self.sockets.fetch_add(1, Ordering::AcqRel) >= MAX_PLAYERS + 8 {
            self.sockets.fetch_sub(1, Ordering::AcqRel);
            return None;
        }
        Some(SocketSlot(self.clone()))
    }

    pub fn handle(&self, stream: TcpStream) -> io::Result<()> {
        let mut wire = Wire::new(stream)?;
        let mut handshake = wire.expect(&self.handshake, "intention")?;
        let version = handshake.varint()?;
        let _host = handshake.string(255)?;
        let _port = handshake.fixed::<2>()?;
        let intent = handshake.varint()?;
        handshake.finish()?;
        match intent {
            1 => self.status(&mut wire),
            2 => self.login(&mut wire, version),
            _ => Err(invalid("unsupported handshake intention")),
        }
    }

    fn status(&self, wire: &mut Wire) -> io::Result<()> {
        wire.expect(&self.status_in, "status_request")?.finish()?;
        let status = ServerStatus {
            version: Some(Version::current()),
            description: Component::literal("Leaf native | Flat world preview"),
            players: Some(Players::new(
                MAX_PLAYERS as i32,
                self.online.load(Ordering::Relaxed) as i32,
                vec![],
            )),
            ..ServerStatus::empty()
        };
        wire.send(
            &self.status_out,
            &ClientboundStatusResponsePacket::new(status),
        )?;
        let mut ping = wire.expect(&self.status_in, "ping_request")?;
        let payload = i64::from_be_bytes(ping.fixed()?);
        ping.finish()?;
        wire.send(
            &self.status_out,
            &ClientboundPongResponsePacket::new(payload),
        )
    }

    fn login(&self, wire: &mut Wire, version: i32) -> io::Result<()> {
        if version != CURRENT_PROTOCOL_VERSION {
            return self.login_disconnect(wire, "This server requires Minecraft Java 1.21.11.");
        }
        let mut hello = wire.expect(&self.login_in, "hello")?;
        let name = hello.string(16)?;
        let _client_uuid = hello.fixed::<16>()?;
        hello.finish()?;
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return self.login_disconnect(wire, "Invalid player name.");
        }
        let uuid = Uuid::create_offline_player_uuid(&name);
        let mut slot = {
            let mut players = self.players.lock().unwrap();
            if players.len() >= MAX_PLAYERS || !players.insert(uuid) {
                drop(players);
                return self.login_disconnect(
                    wire,
                    "The preview is full or this name is already connected.",
                );
            }
            PlayerSlot {
                server: self,
                uuid,
                joined: false,
            }
        };
        wire.send(
            &self.login_out,
            &ClientboundLoginFinishedPacket::new(GameProfile::new(uuid, &name)),
        )?;
        wire.expect(&self.login_in, "login_acknowledged")?
            .finish()?;
        let Some(view_distance) = self.configure(wire)? else {
            return Ok(());
        };

        let entity = self.next_entity.fetch_add(1, Ordering::Relaxed);
        let subscription = self.audience.subscribe(uuid);
        self.spawn(wire, GameProfile::new(uuid, &name), entity, view_distance)?;
        slot.joined = true;
        self.online.fetch_add(1, Ordering::Relaxed);
        self.play(wire, &name, uuid, entity, view_distance, &subscription)
    }

    fn login_disconnect(&self, wire: &mut Wire, reason: &str) -> io::Result<()> {
        wire.send(
            &self.login_out,
            &ClientboundLoginDisconnectPacket::new(Component::literal(reason)),
        )
    }

    fn configure(&self, wire: &mut Wire) -> io::Result<Option<i32>> {
        let mut view_distance = MAX_VIEW_DISTANCE;
        let ids = &self.configuration_out;
        let core = KnownPack::vanilla("core");
        wire.send(
            ids,
            &ClientboundUpdateEnabledFeaturesPacket::new(
                [Identifier::minecraft("vanilla")].into_iter().collect(),
            ),
        )?;
        wire.send(ids, &ClientboundSelectKnownPacks::new(vec![core.clone()]))?;
        let mut choice =
            self.configuration_packet(wire, "select_known_packs", &mut view_distance)?;
        let supported = choice.varint()? == 1
            && choice.string(64)? == core.namespace
            && choice.string(64)? == core.id
            && choice.string(64)? == core.version;
        if !supported {
            wire.send(
                ids,
                &ClientboundDisconnectPacket::new(Component::literal(
                    "This preview requires the built-in Minecraft 1.21.11 core data pack.",
                )),
            )?;
            return Ok(None);
        }
        choice.finish()?;
        for registry in &self.registries.packets {
            wire.send(ids, registry)?;
        }
        wire.send(ids, &self.registries.tags)?;
        wire.send(ids, &CLIENTBOUND_FINISH_CONFIGURATION_PACKET)?;
        self.configuration_packet(wire, "finish_configuration", &mut view_distance)?
            .finish()?;
        Ok(Some(view_distance))
    }

    fn configuration_packet(
        &self,
        wire: &mut Wire,
        expected: &str,
        view_distance: &mut i32,
    ) -> io::Result<Cursor> {
        for _ in 0..32 {
            let (id, body) = wire.receive()?;
            match self.configuration_in.name(id)? {
                name if name == expected => return Ok(body),
                "client_information" => *view_distance = client_view_distance(body)?,
                "custom_payload" => {}
                name => return Err(invalid(format!("expected {expected}, received {name}"))),
            }
        }
        Err(invalid("too many packets during configuration"))
    }

    fn spawn(
        &self,
        wire: &mut Wire,
        profile: GameProfile,
        entity: i32,
        view_distance: i32,
    ) -> io::Result<()> {
        let ids = &self.play_out;
        let dimension = LevelKey::overworld();
        let mode = GameType::Creative;
        wire.send(
            ids,
            &ClientboundLoginPacket::new(
                entity,
                false,
                [dimension.clone()].into_iter().collect(),
                MAX_PLAYERS as i32,
                view_distance,
                SIMULATION_DISTANCE,
                false,
                true,
                false,
                CommonPlayerSpawnInfo {
                    dimension_type: Holder::Reference(self.registries.overworld_id),
                    dimension,
                    seed: 0,
                    game_type: mode,
                    previous_game_type: None,
                    is_debug: false,
                    is_flat: true,
                    last_death_location: None,
                    portal_cooldown: 0,
                    sea_level: SPAWN_Y - 1,
                },
                false,
            ),
        )?;
        {
            let world = self.commands.world.lock().unwrap();
            world.snapshot().send(wire, ids)?;
            wire.send(
                ids,
                &ClientboundCommandsPacket::from_dispatcher(&self.commands.dispatcher, &world),
            )?;
        }
        wire.send(
            ids,
            &ClientboundPlayerAbilitiesPacket {
                invulnerable: true,
                is_flying: false,
                can_fly: true,
                instabuild: true,
                flying_speed: 0.05,
                walking_speed: 0.1,
            },
        )?;
        let mut entry = PlayerInfoEntry::new_minimal(profile.id, true);
        entry.profile = Some(profile);
        entry.game_mode = mode;
        wire.send(
            ids,
            &ClientboundPlayerInfoUpdatePacket::create_player_initializing(vec![entry]),
        )?;
        wire.send(
            ids,
            &ClientboundSetDefaultSpawnPositionPacket::new(RespawnData::of(
                LevelKey::overworld(),
                BlockPos::new(0, SPAWN_Y, 0),
                0.0,
                0.0,
            )),
        )?;
        wire.send(ids, &ClientboundSetHealthPacket::new(20.0, 20, 5.0))?;
        wire.send(
            ids,
            &ClientboundPlayerPositionPacket::of(
                TELEPORT_ID,
                PositionMoveRotation::new(
                    Vec3::new(0.5, f64::from(SPAWN_Y), 0.5),
                    Vec3::new(0.0, 0.0, 0.0),
                    0.0,
                    0.0,
                ),
                Default::default(),
            ),
        )?;
        wire.send(
            ids,
            &ClientboundGameEventPacket::new(LEVEL_CHUNKS_LOAD_START, 0.0),
        )
    }

    fn play(
        &self,
        wire: &mut Wire,
        name: &str,
        uuid: Uuid,
        entity: i32,
        view_distance: i32,
        subscription: &crate::audience::Subscription,
    ) -> io::Result<()> {
        let context = PlayContext {
            world: &self.flat_world,
            ids: &self.play_out,
            audience: &self.audience,
            commands: &self.commands,
            entity_ids: &self.next_entity,
        };
        let mut game = Gameplay::new(entity);
        let mut chunks = ChunkStream::start(view_distance, wire, &self.play_out)?;
        wire.play_timeout()?;
        let mut last_ping = Instant::now();
        let mut outstanding = None;
        let mut ping_id = 0i64;
        let mut confirmed = false;
        let mut player_loaded = false;
        let mut next_world_refresh = Instant::now();
        loop {
            if Instant::now() >= next_world_refresh {
                self.commands.world.lock().unwrap().refresh(&self.audience);
                next_world_refresh = Instant::now() + Duration::from_secs(1);
            }
            subscription.flush(wire, &self.play_out, chunks.loaded())?;
            game.tick(wire, &context)?;
            chunks.send_next(&self.flat_world, wire, &self.play_out)?;
            let elapsed = last_ping.elapsed();
            if outstanding.is_some() && elapsed >= KEEPALIVE_TIMEOUT {
                return Err(io::Error::other("keepalive timed out"));
            }
            if outstanding.is_none() && elapsed >= KEEPALIVE_INTERVAL {
                ping_id += 1;
                wire.send(&self.play_out, &ClientboundKeepAlivePacket::new(ping_id))?;
                outstanding = Some(ping_id);
                last_ping = Instant::now();
            }
            if !wire.ready()? {
                continue;
            }

            let (id, mut body) = wire.receive()?;
            match self.play_in.name(id)? {
                "keep_alive" => {
                    let received = i64::from_be_bytes(body.fixed()?);
                    body.finish()?;
                    if outstanding != Some(received) {
                        return Err(invalid("unexpected keepalive reply"));
                    }
                    outstanding = None;
                }
                "chunk_batch_received" => {
                    let desired = f32::from_be_bytes(body.fixed()?);
                    body.finish()?;
                    chunks.acknowledge(desired);
                }
                "client_information" => {
                    chunks.resize(client_view_distance(body)?, wire, &self.play_out)?;
                }
                "accept_teleportation" => {
                    confirmed = body.varint()? == TELEPORT_ID;
                    body.finish()?;
                }
                "chat" => {
                    let message = body.string(256)?;
                    let _timestamp_and_salt = body.fixed::<16>()?;
                    if body.fixed::<1>()?[0] != 0 {
                        body.fixed::<256>()?;
                    }
                    body.varint()?;
                    body.fixed::<4>()?;
                    body.finish()?;
                    let message = message.trim();
                    if !message.is_empty() && !message.chars().any(char::is_control) {
                        self.audience
                            .chat(Component::literal(format!("<{name}> {message}")), None);
                    }
                }
                "chat_command" | "chat_command_signed" => {
                    let command = body.string(256)?;
                    if self.commands.execute(command.trim(), &self.audience, uuid) {
                        game.kit(wire, &context)?;
                        self.audience.chat(
                            Component::literal("Flight kit: elytra and firework rockets"),
                            Some(uuid),
                        );
                    }
                }
                "command_suggestion" => {
                    let transaction = body.varint()?;
                    let command = body.string(2048)?;
                    body.finish()?;
                    let request = ServerboundCommandSuggestionPacket::new(transaction, command);
                    let reply = request.reply(
                        &self.commands.dispatcher,
                        &self.commands.world.lock().unwrap(),
                    );
                    wire.send(&self.play_out, &reply)?;
                }
                "player_loaded" if !player_loaded => {
                    body.finish()?;
                    player_loaded = true;
                    println!("{name} loaded the flat world");
                }
                kind @ ("move_player_pos"
                | "move_player_pos_rot"
                | "move_player_rot"
                | "move_player_status_only")
                    if confirmed =>
                {
                    game.movement(kind, body, wire, &self.play_out)?;
                    let next = (
                        game.position.x.floor() as i32 >> 4,
                        game.position.z.floor() as i32 >> 4,
                    );
                    chunks.move_to(next, wire, &self.play_out)?;
                }
                kind if confirmed => game.handle(kind, body, wire, &context)?,
                _ => {}
            }
        }
    }
}

pub struct SocketSlot(Arc<Server>);

impl SocketSlot {
    pub fn server(&self) -> &Server {
        &self.0
    }
}

impl Drop for SocketSlot {
    fn drop(&mut self) {
        self.0.sockets.fetch_sub(1, Ordering::AcqRel);
    }
}

struct PlayerSlot<'a> {
    server: &'a Server,
    uuid: Uuid,
    joined: bool,
}

impl Drop for PlayerSlot<'_> {
    fn drop(&mut self) {
        self.server.players.lock().unwrap().remove(&self.uuid);
        if self.joined {
            self.server.online.fetch_sub(1, Ordering::Relaxed);
        }
    }
}
