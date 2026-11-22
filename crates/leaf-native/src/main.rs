mod audience;
mod commands;
mod gameplay;
mod items;
mod registries;
mod session;
mod wire;
mod world;

use std::io;
use std::net::{Ipv4Addr, TcpListener};
use std::process::ExitCode;
use std::sync::Arc;

use session::{Server, MAX_PLAYERS};

fn main() -> ExitCode {
    if let Err(error) = run() {
        eprintln!("leaf-native: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> io::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let port = match args.as_slice() {
        [] => 25565,
        [arg] if arg == "--help" || arg == "-h" => {
            println!("leaf-native [--port PORT]\nLocal Minecraft Java 1.21.11 flat-world preview.");
            return Ok(());
        }
        [flag, port] if flag == "--port" => port
            .parse::<u16>()
            .map_err(|_| io::Error::other("--port expects an integer between 0 and 65535"))?,
        _ => return Err(io::Error::other("usage: leaf-native [--port PORT]")),
    };
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
    let server = Arc::new(Server::new());
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

#[cfg(test)]
mod tests;
