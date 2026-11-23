mod audience;
mod blocks;
mod chunk_stream;
mod commands;
mod crafting;
mod drops;
mod gameplay;
mod items;
mod maps;
mod mob_types;
mod mobs;
mod operators;
mod peers;
mod player_data;
mod recipes;
mod registries;
mod session;
mod special_recipes;
mod wire;
mod world;

use std::io;
use std::net::{Ipv4Addr, TcpListener};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use player_data::PlayerStore;
use session::{Server, MAX_PLAYERS};

fn main() -> ExitCode {
    if let Err(error) = run() {
        eprintln!("leaf-native: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> io::Result<()> {
    let (port, world) = match options()? {
        Some(options) => options,
        None => return Ok(()),
    };
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
    let server = Arc::new(Server::new(PlayerStore::open(&world)?)?);
    println!(
        "Player data directory: {}",
        world.join("playerdata").display()
    );
    println!(
        "Leaf native flat-world preview ready at {} (Java 1.21.11, offline login, up to {MAX_PLAYERS} sessions)",
        listener.local_addr()?,
    );

    for stream in listener.incoming() {
        let stream = stream?;
        let Some(slot) = server.open_session() else {
            continue;
        };
        let peer = stream.peer_addr().ok();
        std::thread::Builder::new()
            .name("leaf-native-client".into())
            .spawn(move || {
                if let Err(error) = slot.server().handle(stream) {
                    if !matches!(
                        error.kind(),
                        io::ErrorKind::UnexpectedEof
                            | io::ErrorKind::ConnectionReset
                            | io::ErrorKind::ConnectionAborted
                    ) {
                        eprintln!("leaf-native session {peer:?}: {error}");
                    }
                }
            })?;
    }
    Ok(())
}

fn options() -> io::Result<Option<(u16, PathBuf)>> {
    let mut args = std::env::args_os().skip(1);
    let mut port = 25565;
    let mut world = PathBuf::from("native-world");
    while let Some(flag) = args.next() {
        match flag.to_str() {
            Some("--port") => {
                port = args
                    .next()
                    .and_then(|value| value.to_str()?.parse().ok())
                    .ok_or_else(|| {
                        io::Error::other("--port expects an integer between 0 and 65535")
                    })?;
            }
            Some("--data-dir") => {
                world = args
                    .next()
                    .filter(|value| !value.is_empty() && value != "--port" && value != "--help")
                    .map(PathBuf::from)
                    .ok_or_else(|| io::Error::other("--data-dir expects a directory"))?;
            }
            Some("--help" | "-h") => {
                println!("leaf-native [--port PORT] [--data-dir PATH]\nLocal Minecraft Java 1.21.11 flat-world preview.");
                return Ok(None);
            }
            _ => {
                return Err(io::Error::other(
                    "usage: leaf-native [--port PORT] [--data-dir PATH]",
                ))
            }
        }
    }
    Ok(Some((port, world)))
}

#[cfg(test)]
mod tests;
