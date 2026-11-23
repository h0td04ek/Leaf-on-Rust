mod audience;
mod blocks;
mod chunk_stream;
mod commands;
mod crafting;
mod drops;
mod gameplay;
mod items;
mod maps;
mod mining;
mod mob_types;
mod mobs;
mod operators;
mod peers;
mod player_data;
mod player_environment;
mod recipes;
mod registries;
mod session;
mod simulation;
mod special_recipes;
mod ticking;
mod wire;
mod workers;
mod world;

use std::io;
use std::net::{Ipv4Addr, TcpListener};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use player_data::PlayerStore;
use session::{Server, MAX_PLAYERS};
use world::WorldType;

fn main() -> ExitCode {
    if let Err(error) = run() {
        eprintln!("leaf-native: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> io::Result<()> {
    let Some(options) = options()? else {
        return Ok(());
    };
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, options.port))?;
    let store = PlayerStore::open(&options.world)?;
    let generation = store.generation(options.world_type, options.seed)?;
    let server = Arc::new(Server::new(store, options.workers, generation)?);
    println!(
        "World generator: {}, seed {}",
        generation.world_type.name(),
        generation.seed
    );
    println!(
        "Player data directory: {}",
        options.world.join("playerdata").display()
    );
    println!(
        "Leaf native preview ready at {} (Java 1.21.11, offline login, up to {MAX_PLAYERS} sessions)",
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

struct Options {
    port: u16,
    world: PathBuf,
    workers: usize,
    world_type: Option<WorldType>,
    seed: Option<i64>,
}

const USAGE: &str = "leaf-native [--port PORT] [--data-dir PATH] [--world-type normal|flat] [--seed SEED] [--worker-threads COUNT]";

fn options() -> io::Result<Option<Options>> {
    let mut args = std::env::args_os().skip(1);
    let mut port = 25565;
    let mut world = None;
    let mut world_type = None;
    let mut seed = None;
    let mut worker_count = workers::available_threads();
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
                world = Some(
                    args.next()
                        .filter(|value| {
                            !value.is_empty() && !value.to_string_lossy().starts_with("--")
                        })
                        .map(PathBuf::from)
                        .ok_or_else(|| io::Error::other("--data-dir expects a directory"))?,
                );
            }
            Some("--world-type") => {
                world_type = Some(
                    args.next()
                        .and_then(|value| WorldType::parse(value.to_str()?))
                        .ok_or_else(|| io::Error::other("--world-type expects normal or flat"))?,
                );
            }
            Some("--seed") => {
                seed = Some(
                    args.next()
                        .and_then(|value| value.to_str()?.parse().ok())
                        .ok_or_else(|| {
                            io::Error::other("--seed expects a signed 64-bit integer")
                        })?,
                );
            }
            Some("--worker-threads") => {
                worker_count = args
                    .next()
                    .and_then(|value| value.to_str()?.parse::<std::num::NonZeroUsize>().ok())
                    .ok_or_else(|| io::Error::other("--worker-threads expects a positive integer"))?
                    .get()
                    .min(workers::available_threads());
            }
            Some("--help" | "-h") => {
                println!("{USAGE}\nLocal Minecraft Java 1.21.11 server.");
                return Ok(None);
            }
            _ => return Err(io::Error::other(format!("usage: {USAGE}"))),
        }
    }
    let world = world.unwrap_or_else(|| {
        PathBuf::from(if world_type == Some(WorldType::Flat) {
            "native-world"
        } else {
            "native-overworld"
        })
    });
    Ok(Some(Options {
        port,
        world,
        workers: worker_count,
        world_type,
        seed,
    }))
}

#[cfg(test)]
mod tests;
