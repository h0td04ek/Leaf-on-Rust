use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use leaf_core::network::codec::stream_codec::StreamCodec;
use leaf_core::network::friendly_byte_buf::FriendlyByteBuf;
use leaf_core::network::protocol::game::clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket;

use leaf_core::core::block_pos::BlockPos;
use leaf_core::entity::position_move_rotation::PositionMoveRotation;
use leaf_core::network::protocol::game::{
    clientbound_add_entity_packet::ClientboundAddEntityPacket,
    clientbound_container_set_content_packet::ClientboundContainerSetContentPacket,
    clientbound_explode_packet::ClientboundExplodePacket,
    clientbound_open_screen_packet::ClientboundOpenScreenPacket,
    clientbound_section_blocks_update_packet::ClientboundSectionBlocksUpdatePacket,
};
use leaf_core::resources::Identifier;
use leaf_core::util::uuid::Uuid;
use leaf_core::world::item::{component::fireworks::Fireworks, item_stack::ItemStack};
use leaf_core::world::phys::vec3::Vec3;

use crate::blocks::Blocks;
use crate::items::{item_id, named_id};
use crate::player_data::PlayerStore;
use crate::recipes::Recipes;
use crate::session::Server;
use crate::world::Generation;

pub(crate) struct TestWorld(pub(crate) PathBuf);

impl TestWorld {
    pub(crate) fn new() -> Self {
        Self(std::env::temp_dir().join(format!("leaf-native-test-{}", Uuid::random())))
    }
}

impl Drop for TestWorld {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn connect(world: &TestWorld) -> (TcpStream, thread::JoinHandle<io::Result<()>>) {
    let store = PlayerStore::open(&world.0).unwrap();
    connect_server(Arc::new(Server::new(store, 1, Generation::FLAT).unwrap()))
}

fn connect_server(server: Arc<Server>) -> (TcpStream, thread::JoinHandle<io::Result<()>>) {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(8)))
        .unwrap();
    let worker = thread::spawn(move || {
        let (stream, _) = listener.accept()?;
        server.handle(stream)
    });
    (client, worker)
}

fn send(stream: &mut TcpStream, id: i32, encode: impl FnOnce(&mut FriendlyByteBuf)) {
    let mut body = FriendlyByteBuf::new();
    body.write_var_int(id);
    encode(&mut body);
    let mut packet = FriendlyByteBuf::new();
    packet.write_var_int(body.writer_index() as i32);
    packet.write_bytes(&body.copy_bytes());
    stream.write_all(&packet.copy_bytes()).unwrap();
}

fn receive(stream: &mut TcpStream) -> (i32, FriendlyByteBuf) {
    let mut size = 0;
    for offset in 0..3 {
        let mut byte = [0];
        stream.read_exact(&mut byte).unwrap();
        size |= usize::from(byte[0] & 127) << (offset * 7);
        if byte[0] < 128 {
            let mut raw = vec![0; size];
            stream.read_exact(&mut raw).unwrap();
            let mut buf = FriendlyByteBuf::from_bytes(&raw);
            return (buf.read_var_int(), buf);
        }
    }
    panic!("invalid frame");
}

fn handshake(stream: &mut TcpStream, target: i32, version: i32) {
    send(stream, 0, |buf| {
        buf.write_var_int(version);
        buf.write_utf("127.0.0.1").unwrap();
        buf.write_short(25565);
        buf.write_var_int(target);
    });
}

#[test]
fn replies_to_status_and_ping() {
    let world = TestWorld::new();
    let (mut client, worker) = connect(&world);
    handshake(&mut client, 1, 774);
    send(&mut client, 0, |_| {});
    let (id, mut body) = receive(&mut client);
    assert_eq!(id, 0);
    let status = body.read_utf().unwrap();
    assert!(status.contains("\"protocol\":774"), "{status}");
    assert!(status.contains("Flat world preview"), "{status}");
    send(&mut client, 1, |buf| {
        buf.write_long(0x1020304050607080);
    });
    let (id, mut body) = receive(&mut client);
    assert_eq!(id, 1);
    assert_eq!(body.read_long(), 0x1020304050607080);
    worker.join().unwrap().unwrap();
}

#[test]
fn normal_world_logs_in_on_normal_difficulty_and_reloads_terrain_edits() {
    use crate::world::{NativeWorld, WorldType};
    use leaf_core::network::protocol::game::clientbound_change_difficulty_packet::ClientboundChangeDifficultyPacket;
    use leaf_core::world::difficulty::Difficulty;

    let directory = TestWorld::new();
    let store = PlayerStore::open(&directory.0).unwrap();
    let generation = store.generation(None, Some(322)).unwrap();
    assert_eq!(generation.world_type, WorldType::Normal);
    let regions = store.region_path();
    let server = Arc::new(Server::new(store, 1, generation).unwrap());
    let (mut client, worker) = connect_server(server);
    enter_play(&mut client, "h0td04ek", 2);
    let difficulty = ClientboundChangeDifficultyPacket::stream_codec()
        .decode(&mut multiplayer_packet(&mut client, 10));
    assert_eq!(difficulty.difficulty, Difficulty::Normal);
    let mut teleport = multiplayer_packet(&mut client, 70);
    let teleport_id = teleport.read_var_int();
    let position = teleport.read_vec3();
    assert!(position.y > 63.0, "{position:?}");
    send(&mut client, 0, |buf| {
        buf.write_var_int(teleport_id);
    });

    let packet = ClientboundLevelChunkWithLightPacket::stream_codec()
        .decode(&mut multiplayer_packet(&mut client, 44));
    assert_eq!(
        (packet.x, packet.z),
        (
            position.x.floor() as i32 >> 4,
            position.z.floor() as i32 >> 4
        )
    );
    assert!(packet.chunk_data.buffer.len() > 2000);
    assert!(!packet.light_data.sky_updates.is_empty());
    let pos = BlockPos::containing_vec3(&position).offset(1, 2, 0);
    command(
        &mut client,
        &format!(
            "fill {} {} {} {} {} {} stone",
            pos.x, pos.y, pos.z, pos.x, pos.y, pos.z
        ),
    );
    multiplayer_packet(&mut client, 119);
    command(&mut client, "save-all flush");
    multiplayer_packet(&mut client, 119);
    drop(client);
    assert!(worker.join().unwrap().is_err());

    let store = PlayerStore::open(&directory.0).unwrap();
    assert_eq!(store.generation(None, None).unwrap(), generation);
    assert!(store.generation(None, Some(323)).is_err());
    let world = NativeWorld::open(
        crate::registries::Registries::load().biomes,
        generation,
        regions,
    )
    .unwrap();
    assert_eq!(
        world.block(pos),
        Blocks::vanilla().named("stone").unwrap().default
    );
    let spawn = world.spawn();
    assert!(world.has_room(spawn.get_bottom_center(), 1.8, false));
    assert!(world.block(spawn.offset(0, -1, 0)) != 0);

    let legacy = TestWorld::new();
    let store = PlayerStore::open(&legacy.0).unwrap();
    std::fs::write(store.ops_path(), "h0td04ek").unwrap();
    assert!(store.generation(Some(WorldType::Normal), None).is_err());
    assert_eq!(store.generation(None, None).unwrap(), Generation::FLAT);
}

#[test]
fn native_play_handles_chunks_chat_commands_drops_and_firework_boost() {
    let world = TestWorld::new();
    let (mut client, worker) = connect(&world);
    let entity = enter_play(&mut client, "h0td04ek", 32);

    let mut rendered = Vec::new();
    let mut advertised_commands = false;
    let mut got_chunk = false;
    loop {
        let (id, mut body) = receive(&mut client);
        match id {
            16 => advertised_commands = true,
            70 => send(&mut client, 0, |buf| {
                buf.write_var_int(body.read_var_int());
            }),
            44 if !got_chunk => {
                rendered.push((0, 0));
                let packet = ClientboundLevelChunkWithLightPacket::stream_codec().decode(&mut body);
                assert_eq!((packet.x, packet.z), (0, 0));
                assert_eq!(packet.chunk_data.buffer.len(), 24 * 6);
                assert_eq!(&packet.chunk_data.buffer[..4], &[0x10, 0x00, 0x00, 0x01]);
                assert_eq!(packet.light_data.sky_updates.len(), 20);
                got_chunk = true;
            }
            44 => rendered.push((body.read_int(), body.read_int())),
            11 => {
                assert_eq!(body.read_var_int(), 9);
                break;
            }
            _ => {}
        }
    }
    assert!(got_chunk && advertised_commands);
    assert_eq!(rendered.len(), 9);

    send(&mut client, 10, |buf| {
        buf.write_float(64.0);
    });
    loop {
        let (id, mut body) = receive(&mut client);
        match id {
            44 => rendered.push((body.read_int(), body.read_int())),
            11 => {
                assert_eq!(body.read_var_int(), 64);
                break;
            }
            _ => {}
        }
    }
    assert_eq!(rendered.len(), 73);
    assert!(rendered.contains(&(4, 0)));

    // Shrinking the client view unloads sent chunks and discards stale queued chunks.
    client_information(&mut client, 13, 2);
    assert_eq!(packet(&mut client, 93).read_var_int(), 2);
    for &(x, z) in &rendered {
        if x.abs() > 2 || z.abs() > 2 {
            assert_eq!(receive(&mut client).0, 37);
        }
    }
    send(&mut client, 10, |buf| {
        buf.write_float(64.0);
    });

    send(&mut client, 43, |_| {}); // player loaded
    send(&mut client, 29, |buf| {
        buf.write_double(16.5)
            .write_double(16.0)
            .write_double(0.5)
            .write_byte(1);
    });
    let (id, mut center) = receive(&mut client);
    assert_eq!(id, 92);
    assert_eq!((center.read_var_int(), center.read_var_int()), (1, 0));
    let mut forgotten = 0;
    let mut sent = 0;
    loop {
        let (id, mut body) = receive(&mut client);
        match id {
            37 => forgotten += 1,
            44 => {
                assert_eq!(body.read_int(), 3);
                sent += 1;
            }
            11 => {
                assert_eq!(body.read_var_int(), 5);
                break;
            }
            _ => {}
        }
    }
    assert_eq!((forgotten, sent), (5, 5));

    send(&mut client, 8, |buf| {
        buf.write_utf("привет").unwrap();
        buf.write_long(0).write_long(0).write_bool(false);
        buf.write_var_int(0).write_bytes(&[0; 4]);
    });
    let mut chat = packet(&mut client, 119);
    assert_eq!(
        chat.read_nbt().unwrap().unwrap().as_string(),
        Some("<h0td04ek> привет")
    );
    assert!(!chat.read_bool());

    send(&mut client, 14, |buf| {
        buf.write_var_int(19).write_utf("/ti").unwrap();
    });
    let mut suggestions = packet(&mut client, 15);
    assert_eq!(suggestions.read_var_int(), 19);
    assert_eq!(
        (suggestions.read_var_int(), suggestions.read_var_int()),
        (1, 2)
    );
    let names = (0..suggestions.read_var_int())
        .map(|_| {
            let name = suggestions.read_utf().unwrap();
            assert!(!suggestions.read_bool());
            name
        })
        .collect::<Vec<_>>();
    assert_eq!(names, ["tick", "time"]);
    send(&mut client, 6, |buf| {
        buf.write_utf("time set night").unwrap();
    });
    let mut time = packet(&mut client, 111);
    time.read_long();
    assert!((13_000..13_004).contains(&time.read_long()));
    packet(&mut client, 119);

    send(&mut client, 6, |buf| {
        buf.write_utf("kit").unwrap();
    });
    let mut chest = packet(&mut client, 106);
    assert_eq!(
        (
            chest.read_var_int(),
            chest.read_var_int(),
            chest.read_var_int()
        ),
        (38, 1, 862)
    );
    let mut hotbar = packet(&mut client, 106);
    assert_eq!(
        (
            hotbar.read_var_int(),
            hotbar.read_var_int(),
            hotbar.read_var_int()
        ),
        (0, 64, 1242)
    );
    packet(&mut client, 119);

    // Airborne player with an equipped glider. The rocket is attached to their own entity.
    send(&mut client, 29, |buf| {
        buf.write_double(16.5)
            .write_double(20.0)
            .write_double(0.5)
            .write_byte(0);
    });
    send(&mut client, 41, |buf| {
        buf.write_var_int(entity).write_var_int(6).write_var_int(0);
    });
    let mut gliding = packet(&mut client, 97);
    assert_eq!(gliding.read_var_int(), entity);
    assert_eq!(gliding.read_bytes_vec(3), [0, 0, 128]);

    send(&mut client, 64, |buf| {
        buf.write_var_int(0)
            .write_var_int(1)
            .write_float(0.0)
            .write_float(0.0);
    });
    let rocket_id = spawned(&mut client, 53);
    let mut rocket = packet(&mut client, 97);
    assert_eq!(rocket.read_var_int(), rocket_id);
    assert_eq!(rocket.read_bytes_vec(2), [8, 7]);
    assert_eq!((rocket.read_var_int(), rocket.read_var_int()), (1, 1242));
    assert_eq!((rocket.read_var_int(), rocket.read_var_int()), (0, 0));
    assert_eq!(rocket.read_bytes_vec(2), [9, 19]);
    assert_eq!(rocket.read_var_int(), entity + 1);

    // Q reduces the held slot and spawns an item entity carrying the dropped stack.
    send(&mut client, 40, |buf| {
        buf.write_var_int(4)
            .write_long(0)
            .write_byte(0)
            .write_var_int(0);
    });
    let mut hotbar = packet(&mut client, 106);
    assert_eq!((hotbar.read_var_int(), hotbar.read_var_int()), (0, 63));
    let thrown_id = spawned(&mut client, 71);
    let mut thrown = packet(&mut client, 97);
    assert_eq!(thrown.read_var_int(), thrown_id);
    assert_eq!(thrown.read_bytes_vec(2), [8, 7]);
    assert_eq!((thrown.read_var_int(), thrown.read_var_int()), (1, 1242));

    let block = leaf_core::core::block_pos::BlockPos::new(16, 15, 0);
    send(&mut client, 29, |buf| {
        buf.write_double(16.5)
            .write_double(16.0)
            .write_double(0.5)
            .write_byte(1);
    });
    packet(&mut client, 97); // leaving the gliding pose
    send(&mut client, 40, |buf| {
        buf.write_var_int(0)
            .write_block_pos(&block)
            .write_byte(1)
            .write_var_int(2);
    });
    let mut change = packet(&mut client, 8);
    assert_eq!(change.read_block_pos(), block);
    assert_eq!(change.read_var_int(), 0);
    assert_eq!(packet(&mut client, 4).read_var_int(), 2);

    // Take a stack from the creative inventory and place a block next to the player.
    send(&mut client, 55, |buf| {
        buf.write_short(36)
            .write_var_int(16)
            .write_var_int(1)
            .write_var_int(0)
            .write_var_int(0);
    });
    let base = leaf_core::core::block_pos::BlockPos::new(17, 15, 0);
    send(&mut client, 63, |buf| {
        buf.write_var_int(0).write_block_pos(&base).write_var_int(1);
        buf.write_float(0.5).write_float(1.0).write_float(0.5);
        buf.write_bool(false).write_bool(false).write_var_int(3);
    });
    let mut placed = packet(&mut client, 8);
    assert_eq!(placed.read_block_pos(), base.offset(0, 1, 0));
    assert_eq!(placed.read_var_int(), 1);
    drop(client);
    assert!(matches!(
        worker.join().unwrap().unwrap_err().kind(),
        io::ErrorKind::UnexpectedEof
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::ConnectionReset
    ));
}

fn packet(stream: &mut TcpStream, expected: i32) -> FriendlyByteBuf {
    for _ in 0..64 {
        let (id, payload) = receive(stream);
        if id == expected {
            return payload;
        }
    }
    panic!("expected packet {expected}");
}

fn client_information(stream: &mut TcpStream, id: i32, distance: i32) {
    send(stream, id, |buf| {
        let mut information =
            leaf_core::server::level::client_information::ClientInformation::create_default();
        information.view_distance = distance as i8;
        information.write(buf);
    });
}

fn spawned(stream: &mut TcpStream, kind: i32) -> i32 {
    for _ in 0..128 {
        let mut body = multiplayer_packet(stream, 1);
        let id = body.read_var_int();
        body.read_uuid();
        if body.read_var_int() == kind {
            return id;
        }
    }
    panic!("expected entity type {kind}")
}

fn enter_play(client: &mut TcpStream, name: &str, distance: i32) -> i32 {
    handshake(client, 2, 774);
    send(client, 0, |buf| {
        buf.write_utf(name).unwrap();
        buf.write_bytes(&[0; 16]);
    });
    let (id, mut body) = receive(client);
    assert_eq!(id, 2);
    assert_eq!(body.read_uuid().version(), 3);
    assert_eq!(body.read_utf().unwrap(), name);
    assert_eq!(body.read_var_int(), 0);
    send(client, 3, |_| {});
    client_information(client, 0, distance);

    assert_eq!(receive(client).0, 12);
    let (id, mut packs) = receive(client);
    assert_eq!(id, 14);
    assert_eq!(packs.read_var_int(), 1);
    for expected in ["minecraft", "core", "1.21.11"] {
        assert_eq!(packs.read_utf().unwrap(), expected);
    }
    send(client, 7, |buf| {
        buf.write_var_int(1);
        for value in ["minecraft", "core", "1.21.11"] {
            buf.write_utf(value).unwrap();
        }
    });

    let mut saw_dimension = false;
    let mut got_tags = false;
    loop {
        let (id, mut packet) = receive(client);
        match id {
            7 => saw_dimension |= packet.read_utf().unwrap() == "minecraft:dimension_type",
            13 => got_tags = true,
            3 => break,
            _ => panic!("unexpected configuration packet ID {id}"),
        }
    }
    assert!(saw_dimension && got_tags);
    send(client, 3, |_| {});

    let (id, mut login) = receive(client);
    assert_eq!(id, 48);
    let entity = login.read_int();
    login.read_bool();
    assert_eq!(login.read_var_int(), 1);
    assert_eq!(login.read_utf().unwrap(), "minecraft:overworld");
    login.read_var_int(); // maximum players
    assert_eq!(login.read_var_int(), distance);

    entity
}

#[test]
fn restores_player_inventory_and_exact_location_after_server_restart() {
    let world = TestWorld::new();
    let (mut client, worker) = connect(&world);
    enter_play(&mut client, "Persistent", 2);
    let mut spawn = packet(&mut client, 70);
    send(&mut client, 0, |buf| {
        buf.write_var_int(spawn.read_var_int());
    });

    send(&mut client, 55, |buf| {
        let mut properties = FriendlyByteBuf::new();
        properties
            .write_var_int(3)
            .write_var_int(1)
            .write_var_int(4);
        properties.write_var_int(1).write_int(0xff0000);
        properties.write_var_int(1).write_int(0x0000ff);
        properties.write_bool(true).write_bool(false);
        buf.write_short(39)
            .write_var_int(12)
            .write_var_int(1242)
            .write_var_int(1)
            .write_var_int(0);
        buf.write_var_int(67)
            .write_var_int(properties.writer_index() as i32);
        buf.write_bytes(&properties.copy_bytes());
    });
    for (slot, count, item) in [(6, 1, 862), (45, 17, 1)] {
        send(&mut client, 55, |buf| {
            buf.write_short(slot)
                .write_var_int(count)
                .write_var_int(item);
            buf.write_var_int(0).write_var_int(0);
        });
    }
    send(&mut client, 52, |buf| {
        buf.write_short(3);
    });

    let first = Vec3::new(-513.25, 74.5, 1025.75);
    send_location(&mut client, first, 112.5, -12.25);
    let mut origin = packet(&mut client, 92);
    let center = (origin.read_var_int(), origin.read_var_int());
    if center == (0, 0) {
        origin = packet(&mut client, 92);
        assert_eq!((origin.read_var_int(), origin.read_var_int()), (-33, 64));
    } else {
        assert_eq!(center, (-33, 64));
    }
    drop(client);
    assert!(worker.join().unwrap().is_err());

    let (mut client, worker) = connect(&world);
    enter_play(&mut client, "Persistent", 2);
    let mut slots = Vec::new();
    let mut selected = None;
    let teleport = loop {
        let (id, mut body) = receive(&mut client);
        match id {
            103 => selected = Some(body.read_var_int()),
            106 => slots.push((
                body.read_var_int(),
                ItemStack::optional_stream_codec().decode(&mut body),
            )),
            70 => break body,
            _ => {}
        }
    };
    assert_eq!(selected, Some(3));
    assert_eq!(
        slots
            .iter()
            .map(|(slot, item)| (*slot, item.count))
            .collect::<Vec<_>>(),
        [(3, 12), (38, 1), (40, 17)]
    );
    let rockets = slots[0]
        .1
        .get_component::<Fireworks>(&Identifier::minecraft("fireworks"))
        .unwrap();
    assert_eq!(rockets.flight_duration, 3);
    assert_eq!(rockets.explosions[0].colors, [0xff0000]);
    assert_eq!(rockets.explosions[0].fade_colors, [0x0000ff]);
    assert!(rockets.explosions[0].has_trail);
    let mut teleport = teleport;
    let teleport_id = teleport.read_var_int();
    let restored = PositionMoveRotation::read(&mut teleport);
    assert_eq!(restored.position, first);
    assert_eq!((restored.y_rot, restored.x_rot), (112.5, -12.25));

    let mut center = packet(&mut client, 92);
    assert_eq!((center.read_var_int(), center.read_var_int()), (-33, 64));
    let mut first_chunk = packet(&mut client, 44);
    assert_eq!((first_chunk.read_int(), first_chunk.read_int()), (-33, 64));

    send(&mut client, 0, |buf| {
        buf.write_var_int(teleport_id);
    });
    send(&mut client, 55, |buf| {
        buf.write_short(45).write_var_int(0);
    });
    let next = Vec3::new(54.5, 80.25, -101.5);
    send_location(&mut client, next, 10.0, 30.0);
    let mut center = packet(&mut client, 92);
    assert_eq!((center.read_var_int(), center.read_var_int()), (3, -7));
    drop(client);
    assert!(worker.join().unwrap().is_err());

    let store = PlayerStore::open(&world.0).unwrap();
    let data = store
        .load(
            Uuid::create_offline_player_uuid("Persistent"),
            Vec3::ZERO,
            leaf_core::world::level::game_type::GameType::Creative,
        )
        .unwrap();
    assert_eq!(data.position, next);
    assert_eq!((data.yaw, data.pitch), (10.0, 30.0));
    assert!(data.inventory[40].is_empty());
    assert_eq!(data.inventory[3].count, 12);
}

fn send_location(client: &mut TcpStream, position: Vec3, yaw: f32, pitch: f32) {
    send(client, 30, |buf| {
        buf.write_vec3(&position)
            .write_float(yaw)
            .write_float(pitch)
            .write_byte(0);
    });
}

#[test]
fn players_see_each_other_and_can_fight_respawn_and_leave() {
    let world = TestWorld::new();
    let server =
        Arc::new(Server::new(PlayerStore::open(&world.0).unwrap(), 1, Generation::FLAT).unwrap());
    let (mut alice, alice_worker) = connect_server(Arc::clone(&server));
    let alice_id = enter_play(&mut alice, "h0td04ek", 2);
    acknowledge_teleport(&mut alice);

    let (mut bob, bob_worker) = connect_server(Arc::clone(&server));
    let bob_id = enter_play(&mut bob, "Bob", 2);
    acknowledge_teleport(&mut bob);
    assert_eq!(peer_entity(&mut alice, "Bob"), bob_id);
    assert_eq!(peer_entity(&mut bob, "h0td04ek"), alice_id);

    // A regular player cannot run /gamemode before being made an operator.
    command(&mut bob, "gamemode survival");
    let denial = multiplayer_packet(&mut bob, 119)
        .read_nbt()
        .unwrap()
        .unwrap();
    assert!(denial.as_string().unwrap().contains("Unknown"));

    // Keep both players close but at different positions.
    send_location(&mut bob, Vec3::new(0.5, 16.0, 2.5), 180.0, 0.0);
    let mut movement = multiplayer_packet(&mut alice, 35);
    assert_eq!(movement.read_var_int(), bob_id);
    assert_eq!(
        PositionMoveRotation::read(&mut movement).position,
        Vec3::new(0.5, 16.0, 2.5)
    );

    command(&mut alice, "op Bob");
    multiplayer_packet(&mut alice, 119);
    let mut permission = multiplayer_packet(&mut bob, 34);
    assert_eq!(permission.read_int(), bob_id);
    assert_eq!(permission.read_unsigned_byte(), 28);

    // The defender opts into PvP. Creative is the normal starting mode.
    send(&mut bob, 6, |buf| {
        buf.write_utf("gamemode survival").unwrap();
    });
    let mut mode = multiplayer_packet(&mut bob, 38);
    assert_eq!(mode.read_unsigned_byte(), 3); // CHANGE_GAME_MODE
    assert_eq!(mode.read_float(), 0.0); // SURVIVAL

    // Alice equips the sword and Bob receives the equipment update.
    send(&mut alice, 55, |buf| {
        buf.write_short(36)
            .write_var_int(1)
            .write_var_int(941)
            .write_var_int(0)
            .write_var_int(0);
    });
    let mut saw_sword = false;
    for _ in 0..3 {
        use leaf_core::network::protocol::game::clientbound_set_equipment_packet::ClientboundSetEquipmentPacket;
        let mut data = multiplayer_packet(&mut bob, 100);
        let equipment = ClientboundSetEquipmentPacket::stream_codec().decode(&mut data);
        if equipment.entity == alice_id
            && equipment.slots.iter().any(|(_, item)| {
                item.item_identifier() == Some(Identifier::minecraft("netherite_sword"))
            })
        {
            saw_sword = true;
            break;
        }
    }
    assert!(saw_sword, "the receiver should see the equipped sword");

    for expected in [12.0, 4.0, 0.0] {
        send(&mut alice, 25, |buf| {
            buf.write_var_int(bob_id).write_var_int(1).write_bool(false);
        });
        assert_eq!(multiplayer_packet(&mut bob, 102).read_float(), expected);
        if expected > 0.0 {
            assert_eq!(multiplayer_packet(&mut bob, 99).read_var_int(), bob_id);
            thread::sleep(Duration::from_millis(525));
        }
    }
    assert_eq!(multiplayer_packet(&mut bob, 66).read_var_int(), bob_id);

    send(&mut bob, 11, |buf| {
        buf.write_var_int(0);
    });
    multiplayer_packet(&mut bob, 80);
    assert_eq!(multiplayer_packet(&mut bob, 102).read_float(), 20.0);
    let mut teleport = multiplayer_packet(&mut bob, 70);
    let teleport_id = teleport.read_var_int();
    assert_eq!(
        PositionMoveRotation::read(&mut teleport).position,
        Vec3::new(0.5, 16.0, 0.5)
    );
    send(&mut bob, 0, |buf| {
        buf.write_var_int(teleport_id);
    });

    drop(bob);
    assert!(bob_worker.join().unwrap().is_err());
    let mut removed = multiplayer_packet(&mut alice, 67);
    assert_eq!(removed.read_var_int(), 1);
    assert_eq!(removed.read_uuid(), Uuid::create_offline_player_uuid("Bob"));
    drop(alice);
    assert!(alice_worker.join().unwrap().is_err());
}

fn acknowledge_teleport(client: &mut TcpStream) {
    let teleport_id = multiplayer_packet(client, 70).read_var_int();
    send(client, 0, |buf| {
        buf.write_var_int(teleport_id);
    });
}

fn peer_entity(client: &mut TcpStream, name: &str) -> i32 {
    let mut entity = multiplayer_packet(client, 1);
    let id = entity.read_var_int();
    assert_eq!(entity.read_uuid(), Uuid::create_offline_player_uuid(name));
    assert_eq!(entity.read_var_int(), 155);
    id
}

fn multiplayer_packet(stream: &mut TcpStream, expected: i32) -> FriendlyByteBuf {
    for _ in 0..128 {
        let (id, payload) = receive(stream);
        if id == 11 {
            send(stream, 10, |buf| {
                buf.write_float(64.0);
            });
        }
        if id == expected {
            return payload;
        }
    }
    panic!("expected multiplayer packet {expected}");
}

fn command(client: &mut TcpStream, command: &str) {
    send(client, 6, |buf| {
        buf.write_utf(command).unwrap();
    });
}

fn mine(client: &mut TcpStream, pos: BlockPos, action: i32, sequence: i32) {
    send(client, 40, |buf| {
        buf.write_var_int(action)
            .write_block_pos(&pos)
            .write_byte(1)
            .write_var_int(sequence);
    });
}

fn block_update(client: &mut TcpStream, block: BlockPos) -> i32 {
    let mut update = multiplayer_packet(client, 8);
    assert_eq!(update.read_block_pos(), block);
    update.read_var_int()
}

fn set_creative(client: &mut TcpStream, slot: i32, item: &str, count: i32) {
    send(client, 55, |buf| {
        buf.write_short(slot).write_var_int(count);
        if count > 0 {
            buf.write_var_int(named_id(item))
                .write_var_int(0)
                .write_var_int(0);
        }
    });
}

fn use_block(client: &mut TcpStream, block: BlockPos, sequence: i32) {
    send(client, 63, |buf| {
        buf.write_var_int(0)
            .write_block_pos(&block)
            .write_var_int(1)
            .write_float(0.5)
            .write_float(1.0)
            .write_float(0.5)
            .write_bool(false)
            .write_bool(false)
            .write_var_int(sequence);
    });
}

#[test]
fn mines_grass_and_stone_and_picks_up_the_vanilla_loot() {
    use leaf_core::network::protocol::game::clientbound_set_entity_data_packet::ClientboundSetEntityDataPacket;
    let world = TestWorld::new();
    let (mut client, worker) = connect(&world);
    let player = enter_play(&mut client, "h0td04ek", 2);
    acknowledge_teleport(&mut client);
    command(&mut client, "gamerule spawn_mobs false");
    multiplayer_packet(&mut client, 119);
    for command_line in [
        "fill 1 16 0 1 16 0 grass_block",
        "fill 0 16 1 0 16 1 stone",
        "fill 1 16 1 1 16 1 short_grass",
    ] {
        command(&mut client, command_line);
        multiplayer_packet(&mut client, 119);
    }
    set_creative(&mut client, 36, "diamond_shovel", 1);
    set_creative(&mut client, 37, "iron_pickaxe", 1);
    command(&mut client, "gamemode survival");
    multiplayer_packet(&mut client, 38);

    for (slot, pos, loot) in [
        (0, BlockPos::new(1, 16, 0), "dirt"),
        (1, BlockPos::new(0, 16, 1), "cobblestone"),
    ] {
        send(&mut client, 52, |buf| {
            buf.write_short(slot);
        });
        mine(&mut client, pos, 0, slot * 2 + 1);
        mine(&mut client, pos, 2, slot * 2 + 2);
        assert_eq!(block_update(&mut client, pos), 0);
        let started = std::time::Instant::now();
        let item = spawned(&mut client, crate::items::ITEM_ENTITY);
        let data = ClientboundSetEntityDataPacket::stream_codec()
            .decode(&mut multiplayer_packet(&mut client, 97));
        assert_eq!(data.id, item);
        let stack = data.packed_items[0]
            .value
            .downcast_ref::<ItemStack>()
            .unwrap();
        assert_eq!(item_id(stack), Some(named_id(loot)));
        let mut pickup = multiplayer_packet(&mut client, 122);
        assert_eq!(pickup.read_var_int(), item);
        assert_eq!(pickup.read_var_int(), player);
        assert_eq!(pickup.read_var_int(), 1);
        assert!(started.elapsed() < Duration::from_millis(1700));
    }
    let pos = BlockPos::new(1, 16, 1);
    mine(&mut client, pos, 0, 5);
    assert_eq!(block_update(&mut client, pos), 0);

    drop(client);
    assert!(worker.join().unwrap().is_err());
}

#[test]
fn switching_to_spectator_updates_the_own_profile_and_flight_abilities() {
    use leaf_core::network::protocol::game::{
        clientbound_player_abilities_packet::ClientboundPlayerAbilitiesPacket,
        clientbound_player_info_update_packet::{
            ClientboundPlayerInfoUpdatePacket, PlayerInfoUpdateAction,
        },
    };
    use leaf_core::world::level::game_type::GameType;
    let world = TestWorld::new();
    let (mut client, worker) = connect(&world);
    enter_play(&mut client, "h0td04ek", 2);
    acknowledge_teleport(&mut client);
    command(&mut client, "gamemode spectator");
    multiplayer_packet(&mut client, 38);
    let abilities = ClientboundPlayerAbilitiesPacket::stream_codec()
        .decode(&mut multiplayer_packet(&mut client, 62));
    assert!(abilities.is_flying && abilities.can_fly);
    assert_eq!(abilities.flying_speed, 0.1);
    let mode = ClientboundPlayerInfoUpdatePacket::stream_codec()
        .decode(&mut multiplayer_packet(&mut client, 68));
    assert!(mode
        .actions
        .contains(&PlayerInfoUpdateAction::UpdateGameMode));
    assert_eq!(
        mode.entries[0].profile_id,
        Uuid::create_offline_player_uuid("h0td04ek")
    );
    assert_eq!(mode.entries[0].game_mode, GameType::Spectator);
    drop(client);
    assert!(worker.join().unwrap().is_err());
}

#[test]
fn players_share_block_edits_and_survival_drops() {
    let world = TestWorld::new();
    let server =
        Arc::new(Server::new(PlayerStore::open(&world.0).unwrap(), 1, Generation::FLAT).unwrap());
    let (mut owner, owner_worker) = connect_server(Arc::clone(&server));
    let owner_id = enter_play(&mut owner, "h0td04ek", 2);
    acknowledge_teleport(&mut owner);
    let (mut guest, guest_worker) = connect_server(Arc::clone(&server));
    enter_play(&mut guest, "Witness", 2);
    acknowledge_teleport(&mut guest);
    multiplayer_packet(&mut guest, 11);
    multiplayer_packet(&mut owner, 11);
    assert_eq!(peer_entity(&mut guest, "h0td04ek"), owner_id);
    peer_entity(&mut owner, "Witness");

    let base = BlockPos::new(1, 15, 0);
    let lower = base.offset(0, 1, 0);
    let upper = lower.offset(0, 1, 0);
    set_creative(&mut owner, 36, "oak_door", 4);
    use_block(&mut owner, base, 1);
    for stream in [&mut owner, &mut guest] {
        let state = block_update(stream, lower);
        assert_eq!(
            Blocks::vanilla().get(state).property(state, "half"),
            Some("lower")
        );
        let state = block_update(stream, upper);
        assert_eq!(
            Blocks::vanilla().get(state).property(state, "half"),
            Some("upper")
        );
    }
    use_block(&mut owner, lower, 2);
    for stream in [&mut owner, &mut guest] {
        for pos in [upper, lower] {
            let state = block_update(stream, pos);
            assert_eq!(
                Blocks::vanilla().get(state).property(state, "open"),
                Some("true")
            );
        }
    }
    mine(&mut owner, upper, 0, 3);
    for stream in [&mut owner, &mut guest] {
        assert_eq!(block_update(stream, lower), 0);
        assert_eq!(block_update(stream, upper), 0);
    }
    for stream in [&mut owner, &mut guest] {
        // A chat marker confirms the scene continues to tick for both clients.
        send(stream, 8, |buf| {
            buf.write_utf("creative break").unwrap();
            buf.write_long(0)
                .write_long(0)
                .write_bool(false)
                .write_var_int(0)
                .write_bytes(&[0; 4]);
        });
        loop {
            let (id, _) = receive(stream);
            assert_ne!(id, 1, "creative break spawned an item");
            if id == 119 {
                break;
            }
        }
    }

    command(&mut owner, "gamemode survival");
    let mut event = multiplayer_packet(&mut owner, 38);
    assert_eq!(event.read_unsigned_byte(), 3);
    assert_eq!(event.read_float(), 0.0);
    let anvil_base = BlockPos::new(2, 15, 0);
    // The operator stays in survival. Use inventory received while in creative.
    use_block(&mut owner, base, 4);
    block_update(&mut owner, lower);
    block_update(&mut owner, upper);
    block_update(&mut guest, lower);
    block_update(&mut guest, upper);
    mine(&mut owner, lower, 0, 5);
    mine(&mut owner, lower, 2, 6);
    for stream in [&mut owner, &mut guest] {
        assert_eq!(block_update(stream, upper), 0);
        assert_eq!(block_update(stream, lower), 0);
    }
    let drop_id = spawned(&mut owner, 71);
    assert_eq!(spawned(&mut guest, 71), drop_id);
    let mut dropped = multiplayer_packet(&mut guest, 97);
    assert_eq!(dropped.read_var_int(), drop_id);
    assert_eq!(dropped.read_bytes_vec(2), [8, 7]);
    assert_eq!(
        (dropped.read_var_int(), dropped.read_var_int()),
        (1, named_id("oak_door"))
    );

    command(&mut owner, "gamemode creative");
    multiplayer_packet(&mut owner, 38);
    set_creative(&mut owner, 36, "anvil", 1);
    use_block(&mut owner, anvil_base, 7);
    for stream in [&mut owner, &mut guest] {
        let state = block_update(stream, anvil_base.offset(0, 1, 0));
        assert_eq!(Blocks::vanilla().get(state).name, "anvil");
    }
    drop(owner);
    drop(guest);
    assert!(owner_worker.join().unwrap().is_err());
    assert!(guest_worker.join().unwrap().is_err());
}

fn click(
    client: &mut TcpStream,
    menu: i32,
    slot: i32,
    button: i32,
    kind: i32,
) -> ClientboundContainerSetContentPacket {
    send(client, 17, |buf| {
        buf.write_var_int(menu)
            .write_var_int(0)
            .write_short(slot)
            .write_byte(button)
            .write_var_int(kind)
            .write_var_int(0)
            .write_bool(false);
    });
    contents(client)
}

fn contents(client: &mut TcpStream) -> ClientboundContainerSetContentPacket {
    ClientboundContainerSetContentPacket::stream_codec().decode(&mut multiplayer_packet(client, 18))
}

#[test]
fn crafts_in_both_menus_and_restores_leftovers() {
    let world = TestWorld::new();
    let (mut client, worker) = connect(&world);
    enter_play(&mut client, "h0td04ek", 2);
    acknowledge_teleport(&mut client);
    set_creative(&mut client, 36, "oak_planks", 12);
    command(&mut client, "gamemode survival");
    multiplayer_packet(&mut client, 38);
    click(&mut client, 0, 36, 0, 0);
    for i in 1..=4 {
        let result = click(&mut client, 0, i, 1, 0);
        assert_eq!(result.carried_item.count, 12 - i);
        if i == 4 {
            assert_eq!(item_id(&result.items[0]), Some(named_id("crafting_table")));
        }
    }
    let shift = click(&mut client, 0, 0, 0, 1);
    assert!(shift.items[1..5].iter().all(ItemStack::is_empty));
    assert_eq!(item_id(&shift.items[36]), Some(named_id("crafting_table")));
    send(&mut client, 18, |buf| {
        buf.write_var_int(0);
    });
    let closed = contents(&mut client);
    assert_eq!(closed.items[37].count, 8);
    assert!(closed.carried_item.is_empty());

    let base = BlockPos::new(1, 15, 0);
    use_block(&mut client, base, 1);
    assert_eq!(
        Blocks::vanilla()
            .get(block_update(&mut client, base.offset(0, 1, 0)))
            .name,
        "crafting_table"
    );
    use_block(&mut client, base.offset(0, 1, 0), 2);
    let open = ClientboundOpenScreenPacket::stream_codec()
        .decode(&mut multiplayer_packet(&mut client, 57));
    assert_eq!(open.menu_type, 12);
    let menu = open.container_id;
    assert_eq!(contents(&mut client).container_id, menu);

    let book = &Recipes::vanilla().book;
    let recipe = book
        .entries
        .iter()
        .find(|entry| {
            use leaf_core::world::item::crafting::display::slot_display::SlotDisplay;
            matches!(entry.contents.display.result(), SlotDisplay::ItemStack(item)
            if item_id(item) == Some(named_id("oak_door")))
        })
        .unwrap()
        .contents
        .id
        .index;
    send(&mut client, 38, |buf| {
        buf.write_var_int(menu)
            .write_var_int(recipe)
            .write_bool(false);
    });
    let filled = contents(&mut client);
    assert_eq!(
        (item_id(&filled.items[0]), filled.items[0].count),
        (Some(named_id("oak_door")), 3)
    );
    let crafted = click(&mut client, menu, 0, 0, 0);
    assert_eq!(
        (item_id(&crafted.carried_item), crafted.carried_item.count),
        (Some(named_id("oak_door")), 3)
    );
    assert!(crafted.items[1..10].iter().all(ItemStack::is_empty));
    click(&mut client, menu, 2, 0, 0); // leave the three doors in the grid
    drop(client);
    assert!(worker.join().unwrap().is_err());

    let (mut client, worker) = connect(&world);
    enter_play(&mut client, "h0td04ek", 2);
    let mut items = vec![];
    loop {
        let (id, mut body) = receive(&mut client);
        if id == 106 {
            body.read_var_int();
            items.push(ItemStack::optional_stream_codec().decode(&mut body));
        }
        if id == 70 {
            break;
        }
    }
    assert!(items
        .iter()
        .any(|item| item_id(item) == Some(named_id("oak_door")) && item.count == 3));
    assert!(items
        .iter()
        .any(|item| item_id(item) == Some(named_id("oak_planks")) && item.count == 2));
    drop(client);
    assert!(worker.join().unwrap().is_err());
}

#[test]
fn expands_native_maps_and_saves_firework_components() {
    use crate::items::{stack, Items};
    use crate::maps::{map_id, set_map_id, Maps};
    use leaf_core::world::item::component::firework_explosion::FireworkExplosion;
    use leaf_core::world::level::saveddata::maps::MapId;

    Items::vanilla();
    let world = TestWorld::new();
    let store = PlayerStore::open(&world.0).unwrap();
    let maps = Maps::open(store.maps_path()).unwrap();
    let mut map = stack(named_id("filled_map"), 1);
    let initial = maps.create(-201, 12).unwrap();
    assert_eq!(initial, MapId(0));
    set_map_id(&mut map, initial);
    let mut grid: [ItemStack; 9] = std::array::from_fn(|_| stack(named_id("paper"), 1));
    grid[4] = map.clone();
    let recipe = Recipes::vanilla().find(&grid, 3, &maps).unwrap();
    assert!(recipe.expand_map);
    assert_eq!(item_id(&recipe.output), Some(named_id("filled_map")));

    let mut latest = initial;
    for _ in 0..4 {
        latest = maps.expand(latest).unwrap();
    }
    assert!(!maps.can_expand(latest));
    set_map_id(&mut grid[4], latest);
    assert!(Recipes::vanilla().find(&grid, 3, &maps).is_none());
    let saved = grid[4].save(leaf_core::nbt::CompoundTag::new()).unwrap();
    assert_eq!(
        map_id(&ItemStack::from_nbt(saved.as_compound().unwrap()).unwrap()),
        Some(latest)
    );

    let inputs = [
        stack(named_id("gunpowder"), 1),
        stack(named_id("red_dye"), 1),
        stack(named_id("diamond"), 1),
        ItemStack::empty(),
    ];
    let star = Recipes::vanilla().find(&inputs, 2, &maps).unwrap().output;
    let effect = star
        .get_component::<FireworkExplosion>(&Identifier::minecraft("firework_explosion"))
        .unwrap();
    assert!(effect.has_trail);
    assert_eq!(effect.colors, [11743532]);
    let rockets = [
        star,
        stack(named_id("gunpowder"), 1),
        stack(named_id("paper"), 1),
        stack(named_id("gunpowder"), 1),
    ];
    let output = Recipes::vanilla().find(&rockets, 2, &maps).unwrap().output;
    let encoded = output.save(leaf_core::nbt::CompoundTag::new()).unwrap();
    let restored = ItemStack::from_nbt(encoded.as_compound().unwrap()).unwrap();
    let fireworks = restored
        .get_component::<Fireworks>(&Identifier::minecraft("fireworks"))
        .unwrap();
    assert_eq!(fireworks.flight_duration, 2);
    assert_eq!(fireworks.explosions, [effect]);
}

#[test]
fn summons_mobs_after_checking_permissions_and_spawns_from_eggs() {
    let world = TestWorld::new();
    let server =
        Arc::new(Server::new(PlayerStore::open(&world.0).unwrap(), 1, Generation::FLAT).unwrap());
    let (mut owner, owner_worker) = connect_server(Arc::clone(&server));
    let owner_id = enter_play(&mut owner, "h0td04ek", 4);
    acknowledge_teleport(&mut owner);
    let (mut guest, guest_worker) = connect_server(Arc::clone(&server));
    enter_play(&mut guest, "Witness", 4);
    acknowledge_teleport(&mut guest);
    let pig = crate::mob_types::named("pig").unwrap().id;
    let chicken = crate::mob_types::named("chicken").unwrap().id;

    assert_eq!(peer_entity(&mut guest, "h0td04ek"), owner_id);
    peer_entity(&mut owner, "Witness");
    command(&mut owner, "gamerule spawn_mobs false");
    multiplayer_packet(&mut owner, 119);
    command(&mut guest, "summon pig");
    loop {
        let reply = multiplayer_packet(&mut guest, 119)
            .read_nbt()
            .unwrap()
            .unwrap();
        if reply
            .as_string()
            .is_some_and(|text| text.contains("Unknown"))
        {
            break;
        }
    }

    command(&mut owner, "summon minecraft:pig ~2 ~ ~ 1");
    assert_eq!(spawned(&mut owner, pig), spawned(&mut guest, pig));
    set_creative(&mut owner, 36, "chicken_spawn_egg", 1);
    use_block(&mut owner, BlockPos::new(1, 15, 1), 1);
    assert_eq!(spawned(&mut owner, chicken), spawned(&mut guest, chicken));

    command(&mut owner, "gamerule spawn_mobs true");
    let natural = loop {
        let entity = ClientboundAddEntityPacket::stream_codec()
            .decode(&mut multiplayer_packet(&mut guest, 1));
        if entity.id != owner_id {
            break entity;
        }
    };
    assert!(
        Vec3::new(natural.x, natural.y, natural.z).distance_to(&Vec3::new(0.5, 16.0, 0.5)) >= 24.0
    );
    drop(owner);
    drop(guest);
    assert!(owner_worker.join().unwrap().is_err());
    assert!(guest_worker.join().unwrap().is_err());
}

#[test]
fn fills_a_thousand_tnt_across_chunk_boundaries_for_both_clients() {
    let world = TestWorld::new();
    let server =
        Arc::new(Server::new(PlayerStore::open(&world.0).unwrap(), 1, Generation::FLAT).unwrap());
    let (mut owner, _) = connect_server(Arc::clone(&server));
    let owner_id = enter_play(&mut owner, "h0td04ek", 2);
    acknowledge_teleport(&mut owner);
    let (mut guest, _) = connect_server(server);
    enter_play(&mut guest, "Witness", 2);
    acknowledge_teleport(&mut guest);
    assert_eq!(peer_entity(&mut guest, "h0td04ek"), owner_id);
    peer_entity(&mut owner, "Witness");

    let fill = "fill ~21 ~9 ~21 ~12 ~ ~12 minecraft:tnt";
    command(&mut guest, fill);
    assert!(multiplayer_packet(&mut guest, 119)
        .copy_bytes()
        .windows(b"Unknown or incomplete command".len())
        .any(|window| window == b"Unknown or incomplete command"));

    command(&mut owner, "gamerule max_block_modifications 999");
    multiplayer_packet(&mut owner, 119);
    command(&mut owner, fill);
    assert!(multiplayer_packet(&mut owner, 119)
        .copy_bytes()
        .windows(b"maximum 999".len())
        .any(|window| window == b"maximum 999"));
    command(&mut owner, "gamerule max_block_modifications 32768");
    multiplayer_packet(&mut owner, 119);

    command(&mut owner, fill);
    let tnt = Blocks::vanilla().named("tnt").unwrap().default;
    let expected = (12..=21)
        .flat_map(|x| (16..=25).flat_map(move |y| (12..=21).map(move |z| BlockPos::new(x, y, z))))
        .collect::<std::collections::HashSet<_>>();
    for client in [&mut owner, &mut guest] {
        let mut changed = std::collections::HashSet::new();
        for _ in 0..4 {
            let section = ClientboundSectionBlocksUpdatePacket::stream_codec()
                .decode(&mut multiplayer_packet(client, 82));
            section.run_updates(|pos, state| {
                assert_eq!(state, tnt);
                assert!(changed.insert(pos));
            });
        }
        assert_eq!(changed, expected);
    }
    assert!(multiplayer_packet(&mut owner, 119)
        .copy_bytes()
        .windows(b"Filled 1000 block(s)".len())
        .any(|window| window == b"Filled 1000 block(s)"));

    // More sections than the subscriber queue can hold as individual messages.
    command(&mut owner, "fill 0 17 0 4096 17 0 tnt");
    assert!(multiplayer_packet(&mut owner, 119)
        .copy_bytes()
        .windows(b"Filled 4097 block(s)".len())
        .any(|window| window == b"Filled 4097 block(s)"));
}

#[test]
fn saves_block_changes_into_anvil_and_loads_them_after_restart() {
    use leaf_core::world::level::chunk::storage::region_file_storage::RegionFileStorage;
    use leaf_core::world::level::chunk_pos::ChunkPos;

    let directory = TestWorld::new();
    let (mut client, worker) = connect(&directory);
    enter_play(&mut client, "h0td04ek", 2);
    acknowledge_teleport(&mut client);
    command(&mut client, "fill 1 16 0 1 16 0 tnt");
    multiplayer_packet(&mut client, 119);
    command(&mut client, "fill -513 16 0 -512 16 0 tnt");
    multiplayer_packet(&mut client, 119);
    command(&mut client, "save-all flush");
    assert!(multiplayer_packet(&mut client, 119)
        .copy_bytes()
        .windows(b"Saved the world".len())
        .any(|window| window == b"Saved the world"));
    let storage = RegionFileStorage::new(directory.0.join("native-region"));
    assert_eq!(
        storage.chunk_positions().unwrap(),
        [
            ChunkPos::new(-33, 0),
            ChunkPos::new(-32, 0),
            ChunkPos::new(0, 0)
        ]
    );
    drop(client);
    assert!(worker.join().unwrap().is_err());

    let (mut client, worker) = connect(&directory);
    enter_play(&mut client, "h0td04ek", 2);
    acknowledge_teleport(&mut client);
    set_creative(&mut client, 36, "flint_and_steel", 1);
    use_block(&mut client, BlockPos::new(1, 16, 0), 0);
    spawned(&mut client, crate::mob_types::named("tnt").unwrap().id);
    drop(client);
    assert!(worker.join().unwrap().is_err());
}

#[test]
fn buckets_exchange_items_and_sync_flowing_water_to_both_clients() {
    let directory = TestWorld::new();
    let server = Arc::new(
        Server::new(
            PlayerStore::open(&directory.0).unwrap(),
            1,
            Generation::FLAT,
        )
        .unwrap(),
    );
    let (mut owner, owner_worker) = connect_server(Arc::clone(&server));
    let owner_id = enter_play(&mut owner, "h0td04ek", 2);
    acknowledge_teleport(&mut owner);
    let (mut guest, guest_worker) = connect_server(server);
    enter_play(&mut guest, "Witness", 2);
    acknowledge_teleport(&mut guest);
    assert_eq!(peer_entity(&mut guest, "h0td04ek"), owner_id);
    peer_entity(&mut owner, "Witness");

    set_creative(&mut owner, 36, "water_bucket", 1);
    command(&mut owner, "gamemode survival");
    let mut event = multiplayer_packet(&mut owner, 38);
    assert_eq!(event.read_unsigned_byte(), 3);
    assert_eq!(event.read_float(), 0.0);

    let source = BlockPos::new(1, 16, 0);
    let side = source.offset(1, 0, 0);
    let water = Blocks::vanilla().named("water").unwrap();
    let use_bucket = |client: &mut TcpStream, sequence| {
        send(client, 64, |buf| {
            buf.write_var_int(0)
                .write_var_int(sequence)
                .write_float(-90.0)
                .write_float(60.0);
        })
    };
    let assert_item = |client: &mut TcpStream, item: &str| {
        let mut data = multiplayer_packet(client, 106);
        assert_eq!(data.read_var_int(), 0);
        assert_eq!(data.read_var_int(), 1);
        assert_eq!(data.read_var_int(), named_id(item));
    };

    use_bucket(&mut owner, 1);
    assert_item(&mut owner, "bucket");
    for client in [&mut owner, &mut guest] {
        assert_eq!(block_update(client, source), water.default);
        let mut flowed = false;
        for _ in 0..8 {
            let update = ClientboundSectionBlocksUpdatePacket::stream_codec()
                .decode(&mut multiplayer_packet(client, 82));
            update.run_updates(|pos, state| {
                flowed |= pos == side && water.property(state, "level") == Some("1");
            });
            if flowed {
                break;
            }
        }
        assert!(flowed);
    }

    use_bucket(&mut owner, 2);
    assert_item(&mut owner, "water_bucket");
    for client in [&mut owner, &mut guest] {
        assert_eq!(block_update(client, source), 0);
    }
    drop(owner);
    drop(guest);
    assert!(owner_worker.join().unwrap().is_err());
    assert!(guest_worker.join().unwrap().is_err());
}

#[test]
fn buckets_waterlog_the_clicked_slab_and_keep_its_orientation() {
    let directory = TestWorld::new();
    let (mut client, worker) = connect(&directory);
    enter_play(&mut client, "h0td04ek", 2);
    acknowledge_teleport(&mut client);
    multiplayer_packet(&mut client, 11);

    let pos = BlockPos::new(1, 16, 0);
    command(&mut client, "fill 1 16 0 1 16 0 oak_slab");
    multiplayer_packet(&mut client, 119);
    set_creative(&mut client, 36, "water_bucket", 1);
    command(&mut client, "gamemode survival");
    let mut event = multiplayer_packet(&mut client, 38);
    assert_eq!((event.read_unsigned_byte(), event.read_float()), (3, 0.0));

    let slab = Blocks::vanilla().named("oak_slab").unwrap();
    let wet = slab.state(slab.default, &[("waterlogged", "true")]);
    for (sequence, expected, held) in [(1, wet, "bucket"), (2, slab.default, "water_bucket")] {
        send(&mut client, 64, |buf| {
            buf.write_var_int(0)
                .write_var_int(sequence)
                .write_float(-90.0)
                .write_float(60.0);
        });
        let mut inventory = multiplayer_packet(&mut client, 106);
        assert_eq!(inventory.read_var_int(), 0);
        assert_eq!(inventory.read_var_int(), 1);
        assert_eq!(inventory.read_var_int(), named_id(held));
        assert_eq!(block_update(&mut client, pos), expected);
    }
    drop(client);
    assert!(worker.join().unwrap().is_err());
}

#[test]
fn players_drown_burn_and_are_extinguished_by_water() {
    use leaf_core::network::protocol::game::clientbound_set_entity_data_packet::ClientboundSetEntityDataPacket;

    let world = TestWorld::new();
    let (mut client, worker) = connect(&world);
    let player = enter_play(&mut client, "h0td04ek", 2);
    assert_eq!(multiplayer_packet(&mut client, 102).read_float(), 20.0);
    acknowledge_teleport(&mut client);
    command(&mut client, "gamerule spawn_mobs false");
    multiplayer_packet(&mut client, 119);
    command(&mut client, "tick freeze");
    multiplayer_packet(&mut client, 119);
    command(&mut client, "fill -1 16 -1 1 17 1 glass");
    multiplayer_packet(&mut client, 119);
    command(&mut client, "fill 0 16 0 0 17 0 water");
    multiplayer_packet(&mut client, 119);
    send_location(&mut client, Vec3::new(0.5, 16.0, 0.5), 0.0, 0.0);

    let mut state = (300, false, 20.0);
    let mut sprint = |client: &mut TcpStream, ticks: i64| {
        command(client, &format!("tick sprint {ticks}t"));
        let summary = format!("World sprint ran {ticks} tick(s)");
        let mut finished = false;
        for _ in 0..512 {
            let (id, mut body) = receive(client);
            match id {
                11 => send(client, 10, |buf| {
                    buf.write_float(64.0);
                }),
                97 => {
                    let packet = ClientboundSetEntityDataPacket::stream_codec().decode(&mut body);
                    if packet.id == player {
                        for value in packet.packed_items {
                            match value.id {
                                0 => state.1 = *value.value.downcast_ref::<i8>().unwrap() & 1 != 0,
                                1 => state.0 = *value.value.downcast_ref::<i32>().unwrap(),
                                _ => {}
                            }
                        }
                    }
                }
                102 => state.2 = body.read_float(),
                119 => {
                    let tag = body.read_nbt().unwrap().unwrap();
                    if let Some(message) = tag.as_string() {
                        if message.starts_with(&summary) {
                            finished = true;
                            // The reply follows any player updates from the completed sprint.
                            command(client, "tick freeze");
                        } else if finished && message == "The world is frozen" {
                            return state;
                        }
                    }
                }
                _ => {}
            }
        }
        panic!("no completion message after sprinting {ticks} ticks");
    };

    assert_eq!(sprint(&mut client, 320), (300, false, 20.0));
    command(&mut client, "gamemode survival");
    multiplayer_packet(&mut client, 38);
    assert_eq!(sprint(&mut client, 300), (0, false, 20.0));
    assert_eq!(sprint(&mut client, 19), (-19, false, 20.0));
    assert_eq!(sprint(&mut client, 1), (0, false, 18.0));

    send_location(&mut client, Vec3::new(8.5, 16.0, 0.5), 0.0, 0.0);
    assert_eq!(sprint(&mut client, 75), (300, false, 18.0));
    command(&mut client, "fill 4 16 0 4 16 0 lava");
    multiplayer_packet(&mut client, 119);
    // The edge of the player touches the lava while their position is in the next block.
    send_location(&mut client, Vec3::new(3.8, 16.0, 0.5), 0.0, 0.0);
    assert_eq!(sprint(&mut client, 1), (300, true, 14.0));
    send_location(&mut client, Vec3::new(8.5, 16.0, 0.5), 0.0, 0.0);
    assert_eq!(sprint(&mut client, 21), (300, true, 13.0));

    send_location(&mut client, Vec3::new(0.5, 17.5, 0.5), 0.0, 0.0);
    assert_eq!(sprint(&mut client, 21), (300, false, 13.0));
    command(&mut client, "gamerule fire_damage false");
    multiplayer_packet(&mut client, 119);
    send_location(&mut client, Vec3::new(4.5, 16.0, 0.5), 0.0, 0.0);
    assert_eq!(sprint(&mut client, 20), (300, true, 13.0));
    command(&mut client, "gamemode spectator");
    multiplayer_packet(&mut client, 38);
    assert_eq!(sprint(&mut client, 1), (300, false, 13.0));
    command(&mut client, "gamerule drowning_damage false");
    multiplayer_packet(&mut client, 119);
    command(&mut client, "gamemode adventure");
    multiplayer_packet(&mut client, 38);
    send_location(&mut client, Vec3::new(0.5, 16.0, 0.5), 0.0, 0.0);
    assert_eq!(sprint(&mut client, 320), (0, false, 13.0));

    drop(client);
    assert!(worker.join().unwrap().is_err());
}

#[test]
fn tnt_detonates_for_both_clients_and_primes_an_adjacent_block() {
    let world = TestWorld::new();
    let server = Arc::new(
        Server::new(
            PlayerStore::open(&world.0).unwrap(),
            crate::workers::available_threads(),
            Generation::FLAT,
        )
        .unwrap(),
    );
    let (mut owner, owner_worker) = connect_server(Arc::clone(&server));
    let owner_id = enter_play(&mut owner, "h0td04ek", 2);
    acknowledge_teleport(&mut owner);
    let (mut guest, guest_worker) = connect_server(server);
    enter_play(&mut guest, "Witness", 2);
    acknowledge_teleport(&mut guest);
    assert_eq!(peer_entity(&mut guest, "h0td04ek"), owner_id);
    peer_entity(&mut owner, "Witness");
    command(&mut owner, "gamerule spawn_mobs false");
    multiplayer_packet(&mut owner, 119);

    let a = BlockPos::new(1, 16, 0);
    let b = BlockPos::new(2, 16, 0);
    let tnt = crate::mob_types::named("tnt").unwrap().id;
    set_creative(&mut owner, 36, "tnt", 2);
    for (number, target) in [a, b].into_iter().enumerate() {
        use_block(&mut owner, target.offset(0, -1, 0), number as i32);
        for stream in [&mut owner, &mut guest] {
            let state = block_update(stream, target);
            assert_eq!(Blocks::vanilla().get(state).name, "tnt");
        }
    }
    set_creative(&mut owner, 36, "flint_and_steel", 1);
    use_block(&mut owner, a, 2);
    for stream in [&mut owner, &mut guest] {
        assert_eq!(block_update(stream, a), 0);
    }
    let first = spawned(&mut owner, tnt);
    assert_eq!(spawned(&mut guest, tnt), first);

    let first_chain = observe_explosion_and_chain(&mut owner, tnt, b);
    assert_ne!(first_chain, first);
    assert_eq!(observe_explosion_and_chain(&mut guest, tnt, b), first_chain);
    let next =
        ClientboundExplodePacket::stream_codec().decode(&mut multiplayer_packet(&mut guest, 36));
    assert!(next.center.distance_to(&Vec3::new(2.5, 16.0, 0.5)) < 1.0);
    drop(owner);
    drop(guest);
    assert!(owner_worker.join().unwrap().is_err());
    assert!(guest_worker.join().unwrap().is_err());
}

fn observe_explosion_and_chain(client: &mut TcpStream, tnt: i32, block: BlockPos) -> i32 {
    let mut chain = None;
    let mut exploded = false;
    let mut cleared = false;
    for _ in 0..400 {
        let (id, mut body) = receive(client);
        match id {
            1 => {
                let entity = body.read_var_int();
                body.read_uuid();
                if body.read_var_int() == tnt && chain.is_none() {
                    chain = Some(entity);
                }
            }
            36 => {
                let event = ClientboundExplodePacket::stream_codec().decode(&mut body);
                assert_eq!(event.radius, 4.0);
                assert!(event.block_count > 0);
                assert!(event.center.distance_to(&Vec3::new(1.5, 16.0, 0.5)) < 1.0);
                exploded = true;
            }
            82 => {
                let packet = ClientboundSectionBlocksUpdatePacket::stream_codec().decode(&mut body);
                packet.run_updates(|pos, state| cleared |= pos == block && state == 0);
            }
            _ => {}
        }
        if exploded && cleared {
            if let Some(chain) = chain {
                return chain;
            }
        }
    }
    panic!("expected the first explosion, block update and chained TNT");
}

#[test]
fn sends_mobs_in_all_loaded_chunks_and_clears_the_edge_on_view_shrink() {
    use leaf_core::network::protocol::game::clientbound_remove_entities_packet::ClientboundRemoveEntitiesPacket;
    use std::collections::HashSet;

    let world = TestWorld::new();
    let (mut client, worker) = connect(&world);
    enter_play(&mut client, "h0td04ek", 32);
    acknowledge_teleport(&mut client);
    command(&mut client, "gamerule spawn_mobs false");
    command(&mut client, "summon minecraft:pig ~ ~ ~ 220");
    command(&mut client, "summon minecraft:pig 512.5 16 0.5");
    command(&mut client, "summon minecraft:pig 528.5 16 0.5");

    let pig_type = crate::mob_types::named("pig").unwrap().id;
    let mut loaded = HashSet::new();
    let mut inner = HashSet::new();
    let mut outer_id = None;
    for _ in 0..15_000 {
        let (id, mut body) = receive(&mut client);
        match id {
            44 => {
                loaded.insert((body.read_int(), body.read_int()));
            }
            11 => send(&mut client, 10, |buf| {
                buf.write_float(64.0);
            }),
            43 => send(&mut client, 27, |buf| {
                buf.write_long(body.read_long());
            }),
            1 => {
                let entity = ClientboundAddEntityPacket::stream_codec().decode(&mut body);
                if matches!(entity.type_, leaf_core::core::holder::Holder::Reference(kind) if kind == pig_type)
                {
                    let chunk = (
                        (entity.x.floor() as i32) >> 4,
                        (entity.z.floor() as i32) >> 4,
                    );
                    assert!(
                        loaded.contains(&chunk),
                        "spawned entity in a chunk still waiting to load"
                    );
                    assert_ne!(chunk, (33, 0), "sent a pig outside the view radius");
                    if chunk == (32, 0) {
                        outer_id = Some(entity.id);
                    } else {
                        assert!(inner.insert(entity.id), "resent an already visible pig");
                    }
                }
            }
            _ => {}
        }
        if outer_id.is_some() && inner.len() == 220 {
            break;
        }
    }
    let outer_id = outer_id.expect("the pig in chunk 32 must reach the client");
    assert_eq!(inner.len(), 220);
    assert!(!loaded.contains(&(33, 0)));

    client_information(&mut client, 13, 2);
    let mut removed = false;
    for _ in 0..15_000 {
        let (id, mut body) = receive(&mut client);
        match id {
            43 => send(&mut client, 27, |buf| {
                buf.write_long(body.read_long());
            }),
            75 => {
                let packet = ClientboundRemoveEntitiesPacket::stream_codec().decode(&mut body);
                assert!(packet.entity_ids.iter().all(|id| !inner.contains(id)));
                if packet.entity_ids.contains(&outer_id) {
                    removed = true;
                    break;
                }
            }
            _ => {}
        }
    }
    assert!(removed, "unloading chunk 32 must remove its pig");
    drop(client);
    assert!(worker.join().unwrap().is_err());
}

#[test]
fn tick_commands_change_the_clock_and_pause_the_tnt_fuse() {
    let world = TestWorld::new();
    let (mut client, worker) = connect(&world);
    enter_play(&mut client, "h0td04ek", 2);
    acknowledge_teleport(&mut client);

    command(&mut client, "gamerule spawn_mobs false");
    multiplayer_packet(&mut client, 119);
    command(&mut client, "tick rate 1000");
    let mut rate = multiplayer_packet(&mut client, 125);
    if rate.read_float() == 20.0 {
        rate = multiplayer_packet(&mut client, 125);
        assert_eq!(rate.read_float(), 1000.0);
    }
    assert!(!rate.read_bool());
    multiplayer_packet(&mut client, 119);

    command(&mut client, "tick freeze");
    let mut frozen = multiplayer_packet(&mut client, 125);
    assert_eq!(frozen.read_float(), 1000.0);
    assert!(frozen.read_bool());
    let frozen_tick = multiplayer_packet(&mut client, 111).read_long();
    multiplayer_packet(&mut client, 119);

    command(&mut client, "summon minecraft:tnt ~2 ~ ~");
    let tnt_type = crate::mob_types::named("tnt").unwrap().id;
    let tnt = spawned(&mut client, tnt_type);
    let mut data = multiplayer_packet(&mut client, 97);
    assert_eq!(data.read_var_int(), tnt);
    assert_eq!(
        (
            data.read_unsigned_byte(),
            data.read_var_int(),
            data.read_var_int()
        ),
        (8, 1, 80)
    );

    command(&mut client, "tick sprint 79t");
    let mut resumed = multiplayer_packet(&mut client, 125);
    assert_eq!(resumed.read_float(), 1000.0);
    assert!(!resumed.read_bool());
    let mut finished = false;
    let mut stepped = false;
    for _ in 0..400 {
        let (id, mut body) = receive(&mut client);
        assert_ne!(id, 36, "TNT exploded before 80 world ticks had elapsed");
        if id == 125 {
            assert_eq!(body.read_float(), 1000.0);
            finished = body.read_bool();
        }
        if id == 111 && finished {
            assert_eq!(body.read_long(), frozen_tick + 79);
            stepped = true;
            break;
        }
    }
    assert!(stepped);

    command(&mut client, "tick step");
    assert_eq!(multiplayer_packet(&mut client, 126).read_var_int(), 1);
    let explosion =
        ClientboundExplodePacket::stream_codec().decode(&mut multiplayer_packet(&mut client, 36));
    assert_eq!(explosion.radius, 4.0);
    assert_eq!(
        multiplayer_packet(&mut client, 111).read_long(),
        frozen_tick + 80
    );

    command(&mut client, "tick unfreeze");
    let mut running = multiplayer_packet(&mut client, 125);
    assert_eq!(running.read_float(), 1000.0);
    assert!(!running.read_bool());
    command(&mut client, "tick query");
    let status = loop {
        let reply = multiplayer_packet(&mut client, 119)
            .read_nbt()
            .unwrap()
            .unwrap();
        if let Some(tag) = reply.as_compound() {
            break tag.get_string("translate").unwrap().to_owned();
        }
    };
    assert!(matches!(
        status.as_str(),
        "commands.tick.status.running" | "commands.tick.status.lagging"
    ));
    let rate = multiplayer_packet(&mut client, 119)
        .read_nbt()
        .unwrap()
        .unwrap();
    let rate = rate.as_compound().unwrap();
    assert_eq!(
        rate.get_string("translate"),
        Some("commands.tick.query.rate.running")
    );
    let args = rate.get_list("with").unwrap();
    assert_eq!(args[0].as_string(), Some("1000.0"));
    assert_eq!(args[2].as_string(), Some("1.0"));
    let percentiles = multiplayer_packet(&mut client, 119)
        .read_nbt()
        .unwrap()
        .unwrap();
    let percentiles = percentiles.as_compound().unwrap();
    assert_eq!(
        percentiles.get_string("translate"),
        Some("commands.tick.query.percentiles")
    );
    let samples = percentiles.get_list("with").unwrap()[3]
        .as_string()
        .unwrap()
        .parse::<usize>()
        .unwrap();
    assert!((80..=100).contains(&samples));
    drop(client);
    assert!(worker.join().unwrap().is_err());
}
