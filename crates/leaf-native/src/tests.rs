use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use leaf_core::network::codec::stream_codec::StreamCodec;
use leaf_core::network::friendly_byte_buf::FriendlyByteBuf;
use leaf_core::network::protocol::game::clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket;

use crate::session::Server;

fn connect() -> (TcpStream, thread::JoinHandle<io::Result<()>>) {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    client
        .set_read_timeout(Some(Duration::from_secs(8)))
        .unwrap();
    let worker = thread::spawn(move || {
        let (stream, _) = listener.accept()?;
        Server::new().handle(stream)
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
    let (mut client, worker) = connect();
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
fn native_play_handles_chunks_chat_commands_drops_and_firework_boost() {
    let (mut client, worker) = connect();
    handshake(&mut client, 2, 774);
    send(&mut client, 0, |buf| {
        buf.write_utf("NativeSmoke").unwrap();
        buf.write_bytes(&[0; 16]);
    });
    let (id, mut body) = receive(&mut client);
    assert_eq!(id, 2);
    assert_eq!(body.read_uuid().version(), 3);
    assert_eq!(body.read_utf().unwrap(), "NativeSmoke");
    assert_eq!(body.read_var_int(), 0);
    send(&mut client, 3, |_| {});
    client_information(&mut client, 0, 32);

    assert_eq!(receive(&mut client).0, 12);
    let (id, mut packs) = receive(&mut client);
    assert_eq!(id, 14);
    assert_eq!(packs.read_var_int(), 1);
    for expected in ["minecraft", "core", "1.21.11"] {
        assert_eq!(packs.read_utf().unwrap(), expected);
    }
    send(&mut client, 7, |buf| {
        buf.write_var_int(1);
        for value in ["minecraft", "core", "1.21.11"] {
            buf.write_utf(value).unwrap();
        }
    });

    let mut saw_dimension = false;
    let mut got_tags = false;
    loop {
        let (id, mut packet) = receive(&mut client);
        match id {
            7 => saw_dimension |= packet.read_utf().unwrap() == "minecraft:dimension_type",
            13 => got_tags = true,
            3 => break,
            _ => panic!("unexpected configuration packet ID {id}"),
        }
    }
    assert!(saw_dimension && got_tags);
    send(&mut client, 3, |_| {});

    let (id, mut login) = receive(&mut client);
    assert_eq!(id, 48);
    let entity = login.read_int();
    login.read_bool();
    assert_eq!(login.read_var_int(), 1);
    assert_eq!(login.read_utf().unwrap(), "minecraft:overworld");
    login.read_var_int(); // maximum players
    assert_eq!(login.read_var_int(), 32);

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
        Some("<NativeSmoke> привет")
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
    assert_eq!(suggestions.read_var_int(), 1);
    assert_eq!(suggestions.read_utf().unwrap(), "time");
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
    let mined_id = spawned(&mut client, 71);
    let mut mined = packet(&mut client, 97);
    assert_eq!(mined.read_var_int(), mined_id);
    assert_eq!(mined.read_bytes_vec(2), [8, 7]);
    assert_eq!((mined.read_var_int(), mined.read_var_int()), (1, 1));
    let mut change = packet(&mut client, 8);
    assert_eq!(change.read_block_pos(), block);
    assert_eq!(change.read_var_int(), 0);

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
    assert_eq!(
        worker.join().unwrap().unwrap_err().kind(),
        io::ErrorKind::UnexpectedEof
    );
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
    let mut body = packet(stream, 1);
    let id = body.read_var_int();
    body.read_uuid();
    assert_eq!(body.read_var_int(), kind);
    id
}
