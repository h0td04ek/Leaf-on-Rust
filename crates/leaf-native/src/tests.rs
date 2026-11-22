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
fn negotiates_vanilla_registries_and_streams_spawn_and_neighbor_chunks() {
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

    assert_eq!(receive(&mut client).0, 48); // play login
    let mut got_chunk = false;
    loop {
        let (id, mut body) = receive(&mut client);
        match id {
            70 => send(&mut client, 0, |buf| {
                buf.write_var_int(body.read_var_int());
            }),
            44 if !got_chunk => {
                let packet = ClientboundLevelChunkWithLightPacket::stream_codec().decode(&mut body);
                assert_eq!((packet.x, packet.z), (0, 0));
                assert_eq!(packet.chunk_data.buffer.len(), 24 * 6);
                assert_eq!(&packet.chunk_data.buffer[..4], &[0x10, 0x00, 0x00, 0x01]);
                assert_eq!(packet.light_data.sky_updates.len(), 20);
                got_chunk = true;
            }
            11 => {
                assert_eq!(body.read_var_int(), 25);
                break;
            }
            _ => {}
        }
    }
    assert!(got_chunk);

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
    drop(client);
    assert_eq!(
        worker.join().unwrap().unwrap_err().kind(),
        io::ErrorKind::UnexpectedEof
    );
}
