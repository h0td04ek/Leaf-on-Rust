use std::collections::HashSet;
use std::io;
use std::net::TcpStream;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use leaf_core::authlib::game_profile::GameProfile;
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
    clientbound_entity_event_packet::ClientboundEntityEventPacket,
    clientbound_game_event_packet::{ClientboundGameEventPacket, LEVEL_CHUNKS_LOAD_START},
    clientbound_hurt_animation_packet::ClientboundHurtAnimationPacket,
    clientbound_login_packet::ClientboundLoginPacket,
    clientbound_player_combat_kill_packet::ClientboundPlayerCombatKillPacket,
    clientbound_player_info_update_packet::{
        ClientboundPlayerInfoUpdatePacket, PlayerInfoEntry, PlayerInfoUpdateAction,
    },
    clientbound_player_position_packet::ClientboundPlayerPositionPacket,
    clientbound_respawn_packet::{ClientboundRespawnPacket, KEEP_ALL_DATA},
    clientbound_set_default_spawn_position_packet::ClientboundSetDefaultSpawnPositionPacket,
    clientbound_set_entity_motion_packet::ClientboundSetEntityMotionPacket,
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

use crate::blocks::Blocks;
use crate::chunk_stream::{
    client_view_distance, ChunkStream, MAX_VIEW_DISTANCE, SIMULATION_DISTANCE,
};
use crate::commands::{Commands, PlayerCommand};
use crate::gameplay::{Gameplay, PlayContext};
use crate::items::Items;
use crate::maps::Maps;
use crate::mining::Mining;
use crate::mobs::MobTracker;
use crate::operators::{valid_name, Operators, PERMISSION_EVENT};
use crate::peers::{attack_damage, PeerTracker, Presence};
use crate::player_data::PlayerStore;
use crate::recipes::Recipes;
use crate::registries::Registries;
use crate::simulation::{Autosave, Simulation};
use crate::wire::{invalid, Cursor, PacketIds, Wire};
use crate::world::{Generation, NativeWorld, WorldType};

pub const MAX_PLAYERS: usize = 8;
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(8);
const KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(20);
const TELEPORT_ID: i32 = 1;
const PLAYER_SAVE_INTERVAL: Duration = Duration::from_secs(5);

pub struct Server {
    // Stop and flush the autosave thread before releasing the world directory lock.
    _autosave: Autosave,
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
    player_data: PlayerStore,
    maps: Maps,
    simulation: Arc<Simulation>,
    sockets: AtomicUsize,
    players: Mutex<HashSet<Uuid>>,
    online: AtomicUsize,
}

impl Server {
    pub fn new(
        player_data: PlayerStore,
        workers: usize,
        generation: Generation,
    ) -> io::Result<Self> {
        Items::vanilla();
        Blocks::vanilla();
        Mining::vanilla();
        Recipes::vanilla();
        let registries = Registries::load();
        let operators = Operators::open(player_data.ops_path())?;
        let maps = Maps::open(player_data.maps_path())?;
        let simulation = Simulation::start(
            NativeWorld::open(registries.biomes, generation, player_data.region_path())?,
            Commands::new(operators, generation.seed),
            workers,
        )?;
        Ok(Self {
            _autosave: simulation.autosave()?,
            handshake: PacketIds::from_protocol(handshake_protocols::serverbound()),
            status_in: PacketIds::from_protocol(status_protocols::serverbound()),
            status_out: PacketIds::from_protocol(status_protocols::clientbound()),
            login_in: PacketIds::from_protocol(login_protocols::serverbound()),
            login_out: PacketIds::from_protocol(login_protocols::clientbound()),
            configuration_in: PacketIds::from_protocol(configuration_protocols::serverbound()),
            configuration_out: PacketIds::from_protocol(configuration_protocols::clientbound()),
            play_in: PacketIds::from_protocol(game_protocols::serverbound_template().bind()),
            play_out: PacketIds::from_protocol(game_protocols::clientbound_template().bind()),
            maps,
            simulation,
            registries,
            player_data,
            sockets: AtomicUsize::new(0),
            players: Mutex::new(HashSet::new()),
            online: AtomicUsize::new(0),
        })
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
            description: Component::literal(
                match self.simulation.terrain.generation().world_type {
                    WorldType::Flat => "Leaf native | Flat world preview",
                    WorldType::Normal => "Leaf native | Overworld",
                },
            ),
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
        if !valid_name(&name) {
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
        let world = &self.simulation.terrain;
        let mode = if world.generation().world_type == WorldType::Normal {
            GameType::Survival
        } else {
            GameType::Creative
        };
        let player = match self
            .player_data
            .load(uuid, world.spawn().get_bottom_center(), mode)
        {
            Ok(player) => player,
            Err(error) => {
                eprintln!("Could not load player {uuid}: {error}");
                return self.login_disconnect(wire, "Failed to read saved player data.");
            }
        };
        let profile = GameProfile::new(uuid, &name);
        wire.send(
            &self.login_out,
            &ClientboundLoginFinishedPacket::new(profile.clone()),
        )?;
        wire.expect(&self.login_in, "login_acknowledged")?
            .finish()?;
        let Some(view_distance) = self.configure(wire)? else {
            return Ok(());
        };

        let entity = self.simulation.next_entity.fetch_add(1, Ordering::Relaxed);
        let subscription = self.simulation.audience.subscribe(uuid);
        let mut game = Gameplay::new(entity, player, self.simulation.commands.game_time());
        self.spawn(wire, profile.clone(), entity, view_distance, &game)?;
        slot.joined = true;
        self.online.fetch_add(1, Ordering::Relaxed);

        let presence = self.simulation.peers.join(
            entity,
            profile.clone(),
            game.view(),
            game.player.health,
            game.player.environment,
        );
        let mut saved = Vec::new();
        let result = self.play(
            wire,
            &profile,
            &mut game,
            view_distance,
            &subscription,
            &presence,
            &mut saved,
        );
        (game.player.health, game.player.environment) = presence.leave();
        game.finish_session();
        if let Err(error) =
            self.player_data
                .save(uuid, &game.player, std::iter::empty(), &mut saved)
        {
            eprintln!("Could not save player {uuid} on disconnect: {error}");
        }
        result
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
        game: &Gameplay,
    ) -> io::Result<()> {
        let ids = &self.play_out;
        let dimension = LevelKey::overworld();
        let mode = game.player.mode;
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
                self.spawn_info(mode),
                false,
            ),
        )?;
        let snapshot = self.simulation.commands.world.lock().unwrap().snapshot();
        snapshot.send(wire, ids)?;
        wire.send(ids, &game.abilities())?;
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
                self.simulation.terrain.spawn(),
                0.0,
                0.0,
            )),
        )?;
        wire.send(
            ids,
            &ClientboundSetHealthPacket::new(game.player.health, 20, 5.0),
        )?;
        game.sync_inventory(wire, ids)?;
        wire.send(ids, &Recipes::vanilla().book)?;
        self.teleport(wire, game, TELEPORT_ID)?;
        if game.player.health == 0.0 {
            wire.send(
                ids,
                &ClientboundPlayerCombatKillPacket::new(entity, Component::literal("You died")),
            )?;
        }
        wire.send(
            ids,
            &ClientboundGameEventPacket::new(LEVEL_CHUNKS_LOAD_START, 0.0),
        )
    }

    fn spawn_info(&self, mode: GameType) -> CommonPlayerSpawnInfo {
        CommonPlayerSpawnInfo {
            dimension_type: Holder::Reference(self.registries.overworld_id),
            dimension: LevelKey::overworld(),
            seed: self.simulation.terrain.generation().biome_seed(),
            game_type: mode,
            previous_game_type: None,
            is_debug: false,
            is_flat: self.simulation.terrain.generation().world_type == WorldType::Flat,
            last_death_location: None,
            portal_cooldown: 0,
            sea_level: self.simulation.terrain.sea_level(),
        }
    }

    fn teleport(&self, wire: &mut Wire, game: &Gameplay, id: i32) -> io::Result<()> {
        wire.send(
            &self.play_out,
            &ClientboundPlayerPositionPacket::of(
                id,
                PositionMoveRotation::new(
                    game.player.position,
                    Vec3::ZERO,
                    game.player.yaw,
                    game.player.pitch,
                ),
                Default::default(),
            ),
        )
    }

    fn play(
        &self,
        wire: &mut Wire,
        profile: &GameProfile,
        game: &mut Gameplay,
        view_distance: i32,
        subscription: &crate::audience::Subscription,
        presence: &Presence<'_>,
        saved: &mut Vec<u8>,
    ) -> io::Result<()> {
        let name = &profile.name;
        let uuid = profile.id;
        let context = PlayContext {
            world: &self.simulation.terrain,
            drops: &self.simulation.drops,
            maps: &self.maps,
            mobs: &self.simulation.mobs,
            name,
            player: uuid,
            ids: &self.play_out,
            audience: &self.simulation.audience,
            commands: &self.simulation.commands,
            entity_ids: &self.simulation.next_entity,
        };
        let mut chunks =
            ChunkStream::start(game.player.chunk(), view_distance, wire, &self.play_out)?;
        wire.play_timeout()?;
        game.restore_crafting(wire, &context)?;
        let mut last_ping = Instant::now();
        let mut outstanding = None;
        let mut ping_id = 0i64;
        let mut confirmed = false;
        let mut teleport_id = TELEPORT_ID;
        let mut tracker = PeerTracker::default();
        let mut mob_tracker = MobTracker::default();
        let mut next_peer_update = Instant::now();
        let mut player_loaded = false;
        let mut permission = None;
        let mut next_save = Instant::now() + PLAYER_SAVE_INTERVAL;
        loop {
            let now = Instant::now();
            if let Some(mode) = presence.take_mode() {
                game.close_crafting(wire, &context, true)?;
                game.set_mode(mode, wire, &self.play_out)?;
                let mut entry = PlayerInfoEntry::new_minimal(uuid, true);
                entry.game_mode = mode;
                wire.send(
                    &self.play_out,
                    &ClientboundPlayerInfoUpdatePacket::new(
                        [PlayerInfoUpdateAction::UpdateGameMode]
                            .into_iter()
                            .collect(),
                        vec![entry],
                    ),
                )?;
                presence.publish(game.view());
                self.simulation.audience.chat(
                    Component::literal(format!("Game mode set to {}", mode.get_name())),
                    Some(uuid),
                );
            }
            let (hit, environment) = presence.take_updates();
            game.sync_environment(environment, wire, &self.play_out)?;
            if let Some(hit) = hit {
                game.player.health = hit.health;
                if hit.health == 0.0 {
                    game.close_crafting(wire, &context, true)?;
                }
                wire.send(
                    &self.play_out,
                    &ClientboundSetHealthPacket::new(hit.health, 20, 5.0),
                )?;
                wire.send(
                    &self.play_out,
                    &ClientboundHurtAnimationPacket::new(presence.id, 0.0),
                )?;
                if hit.health == 0.0 {
                    let death = Component::literal(hit.death_message);
                    wire.send(
                        &self.play_out,
                        &ClientboundPlayerCombatKillPacket::new(presence.id, death.clone()),
                    )?;
                    self.simulation.audience.chat(death, None);
                } else if let Some(velocity) = hit.velocity {
                    wire.send(
                        &self.play_out,
                        &ClientboundSetEntityMotionPacket::new(presence.id, velocity),
                    )?;
                }
            }
            if now >= next_save {
                if let Err(error) =
                    self.player_data
                        .save(uuid, &game.player, game.crafting.recovery_items(), saved)
                {
                    eprintln!("Could not autosave player {uuid}: {error}");
                }
                next_save = now + PLAYER_SAVE_INTERVAL;
            }
            let updated_permission = self.simulation.commands.for_player(name, |world| {
                let level = world.permission();
                (permission != Some(level)).then(|| {
                    (
                        level,
                        ClientboundCommandsPacket::from_dispatcher(
                            &self.simulation.commands.dispatcher,
                            world,
                        ),
                    )
                })
            });
            if let Some((level, tree)) = updated_permission {
                wire.send(&self.play_out, &tree)?;
                wire.send(
                    &self.play_out,
                    &ClientboundEntityEventPacket::new(
                        presence.id,
                        (PERMISSION_EVENT + level) as i8,
                    ),
                )?;
                permission = Some(level);
            }
            let update_peers = now >= next_peer_update;
            subscription.flush(wire, &self.play_out, chunks.loaded())?;
            game.check_crafting(wire, &context)?;
            game.tick(wire, &context, chunks.loaded(), subscription)?;
            chunks.send_next(&self.simulation.terrain, wire, &self.play_out)?;
            if update_peers {
                presence.publish(game.view());
                tracker.sync(presence, chunks.loaded(), wire, &self.play_out)?;
                mob_tracker.sync(
                    game.player.position,
                    chunks.loaded(),
                    &context,
                    wire,
                    subscription,
                )?;
                next_peer_update = now + Duration::from_millis(50);
            }
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
                    confirmed = body.varint()? == teleport_id;
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
                        self.simulation
                            .audience
                            .chat(Component::literal(format!("<{name}> {message}")), None);
                    }
                }
                "chat_command" | "chat_command_signed" => {
                    let command = body.string(256)?;
                    if game.player.health > 0.0 {
                        match self.simulation.commands.execute(
                            command.trim(),
                            &self.simulation.audience,
                            uuid,
                            name,
                        ) {
                            Some(PlayerCommand::Kit) => {
                                game.kit(wire, &context)?;
                                self.simulation.audience.chat(
                                    Component::literal("Flight kit: elytra and firework rockets"),
                                    Some(uuid),
                                );
                            }
                            Some(PlayerCommand::SaveWorld) => {
                                let feedback = match self.simulation.save_world() {
                                    Ok(count) => {
                                        format!("Saved the world ({count} changed chunk(s))")
                                    }
                                    Err(error) => {
                                        eprintln!("leaf-native: saving block changes: {error}");
                                        "Failed to save the world. Check the server log.".into()
                                    }
                                };
                                context
                                    .audience
                                    .chat(Component::literal(feedback), Some(uuid));
                            }
                            Some(PlayerCommand::SetMode(mode, target)) => {
                                let player = target.as_deref().unwrap_or(name);
                                if !self.simulation.peers.set_mode(player, mode) {
                                    self.simulation.audience.chat(
                                        Component::literal(format!(
                                            "Player {player} is not online"
                                        )),
                                        Some(uuid),
                                    );
                                } else if player != name {
                                    self.simulation.audience.chat(
                                        Component::literal(format!(
                                            "Set {player}'s game mode to {}",
                                            mode.get_name()
                                        )),
                                        Some(uuid),
                                    );
                                }
                            }
                            Some(PlayerCommand::Fill {
                                from,
                                to,
                                state,
                                mode,
                                limit,
                            }) => {
                                let player = &game.player;
                                let resolve =
                                    |pos: leaf_core::command::arguments::vanilla::Coordinates| {
                                        pos.block_pos(
                                            player.position,
                                            player.position,
                                            player.yaw,
                                            player.pitch,
                                        )
                                    };
                                let feedback = match context.world.fill(
                                    resolve(from),
                                    resolve(to),
                                    state,
                                    mode,
                                    limit,
                                    context.audience,
                                ) {
                                    Ok(result) => {
                                        let drops =
                                            context.commands.world.lock().unwrap().drops_enabled();
                                        if drops {
                                            for (pos, item) in result.destroyed {
                                                game.drop_block(pos, item, &context);
                                            }
                                        }
                                        format!("Filled {} block(s)", result.changed)
                                    }
                                    Err(error) => error,
                                };
                                context
                                    .audience
                                    .chat(Component::literal(feedback), Some(uuid));
                            }
                            Some(PlayerCommand::Summon {
                                kind,
                                position,
                                count,
                            }) => {
                                let player = &game.player;
                                let origin = position.map_or(player.position, |pos| {
                                    pos.resolve(
                                        player.position,
                                        player.position,
                                        player.yaw,
                                        player.pitch,
                                    )
                                });
                                let feedback = match self
                                    .simulation
                                    .mobs
                                    .summon(kind, origin, count, &context)
                                {
                                    Ok(()) if count == 1 => {
                                        format!("Summoned minecraft:{}", kind.name)
                                    }
                                    Ok(()) => format!(
                                        "Queued {count} minecraft:{} around the summon position",
                                        kind.name
                                    ),
                                    Err(error) => error.to_owned(),
                                };
                                self.simulation
                                    .audience
                                    .chat(Component::literal(feedback), Some(uuid));
                            }
                            None => {}
                        }
                        presence.publish(game.view());
                    }
                }
                "command_suggestion" => {
                    let transaction = body.varint()?;
                    let command = body.string(2048)?;
                    body.finish()?;
                    let request = ServerboundCommandSuggestionPacket::new(transaction, command);
                    let reply = self.simulation.commands.for_player(name, |world| {
                        request.reply(&self.simulation.commands.dispatcher, world)
                    });
                    wire.send(&self.play_out, &reply)?;
                }
                "client_command" => {
                    let action = body.varint()?;
                    body.finish()?;
                    if action == 0 && presence.health() == 0.0 {
                        game.reset_after_death(self.simulation.terrain.spawn().get_bottom_center());
                        presence.respawn(game.view());
                        confirmed = false;
                        teleport_id = teleport_id.wrapping_add(1);
                        let ids = &self.play_out;
                        wire.send(
                            ids,
                            &ClientboundRespawnPacket::new(
                                self.spawn_info(game.player.mode),
                                KEEP_ALL_DATA,
                            ),
                        )?;
                        wire.send(ids, &game.abilities())?;
                        wire.send(
                            ids,
                            &ClientboundSetHealthPacket::new(game.player.health, 20, 5.0),
                        )?;
                        game.sync_inventory(wire, ids)?;
                        self.teleport(wire, game, teleport_id)?;
                        wire.send(
                            ids,
                            &ClientboundGameEventPacket::new(LEVEL_CHUNKS_LOAD_START, 0.0),
                        )?;
                        permission = None;
                        tracker.reset_entities(wire, ids)?;
                        mob_tracker.reset(wire, ids)?;
                        chunks.restart(game.player.chunk(), wire, ids)?;
                    }
                }
                "interact" if confirmed && game.player.health > 0.0 => {
                    let target = body.varint()?;
                    if body.varint()? == 1 {
                        body.fixed::<1>()?;
                        body.finish()?;
                        if tracker.sees(target) {
                            presence.attack(
                                target,
                                &self.simulation.terrain,
                                self.simulation.commands.game_time(),
                            );
                        } else if mob_tracker.sees(target) {
                            let damage =
                                attack_damage(&game.player.inventory[game.player.selection]);
                            self.simulation.mobs.attack(
                                target,
                                game.player.position,
                                game.player.mode,
                                damage,
                                &context,
                            );
                        }
                    }
                }
                "swing" if confirmed && game.player.health > 0.0 => {
                    let hand = body.count(1)?;
                    body.finish()?;
                    presence.swing(if hand == 0 { 0 } else { 3 });
                }
                "player_loaded" if !player_loaded => {
                    body.finish()?;
                    player_loaded = true;
                    println!("{name} loaded the world");
                }
                kind @ ("move_player_pos"
                | "move_player_pos_rot"
                | "move_player_rot"
                | "move_player_status_only")
                    if confirmed && game.player.health > 0.0 =>
                {
                    game.movement(kind, body, wire, &self.play_out)?;
                    presence.publish(game.view());
                    let next = game.player.chunk();
                    chunks.move_to(next, wire, &self.play_out)?;
                }
                kind if confirmed && game.player.health > 0.0 => {
                    game.handle(kind, body, wire, &context)?;
                    presence.publish(game.view());
                }
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
