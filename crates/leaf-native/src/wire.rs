use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use leaf_core::network::friendly_byte_buf::FriendlyByteBuf;
use leaf_core::network::protocol::packet::Packet;
use leaf_core::network::protocol::protocol_info::ProtocolInfo;

const MAX_INBOUND_FRAME: usize = 256 * 1024;

pub struct PacketIds {
    by_id: HashMap<i32, String>,
    by_name: HashMap<String, i32>,
}

impl PacketIds {
    pub fn from_protocol(protocol: Box<dyn ProtocolInfo>) -> Self {
        let mut ids = Self {
            by_id: HashMap::new(),
            by_name: HashMap::new(),
        };
        protocol.details().list_packets(&mut |kind, id| {
            let name = kind.id.get_path().to_owned();
            ids.by_id.insert(id as i32, name.clone());
            ids.by_name.insert(name, id as i32);
        });
        ids
    }

    pub fn name(&self, id: i32) -> io::Result<&str> {
        self.by_id
            .get(&id)
            .map(String::as_str)
            .ok_or_else(|| invalid(format!("unknown packet ID {id}")))
    }

    pub fn id(&self, name: &str) -> io::Result<i32> {
        self.by_name
            .get(name)
            .copied()
            .ok_or_else(|| invalid(format!("unregistered packet {name}")))
    }
}

pub struct Wire {
    stream: BufReader<TcpStream>,
}

impl Wire {
    pub fn new(stream: TcpStream) -> io::Result<Self> {
        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(Duration::from_secs(8)))?;
        stream.set_write_timeout(Some(Duration::from_secs(8)))?;
        Ok(Self {
            stream: BufReader::new(stream),
        })
    }

    pub fn receive(&mut self) -> io::Result<(i32, Cursor)> {
        let len = read_varint(&mut self.stream, 3)? as usize;
        if !(1..=MAX_INBOUND_FRAME).contains(&len) {
            return Err(invalid("invalid incoming frame size"));
        }
        let mut bytes = vec![0; len];
        self.stream.read_exact(&mut bytes)?;
        let mut input = Cursor::new(bytes);
        let id = input.varint()?;
        Ok((id, input))
    }

    pub fn expect(&mut self, ids: &PacketIds, name: &str) -> io::Result<Cursor> {
        let (id, body) = self.receive()?;
        let actual = ids.name(id)?;
        if actual != name {
            return Err(invalid(format!("expected {name}, received {actual}")));
        }
        Ok(body)
    }

    pub fn send(&mut self, ids: &PacketIds, packet: &impl Packet) -> io::Result<()> {
        let kind = packet.packet_type();
        let mut body = FriendlyByteBuf::new();
        body.write_var_int(ids.id(kind.id.get_path())?);
        packet.write_packet(&mut body);
        let bytes = body.copy_bytes();
        let mut header = FriendlyByteBuf::new();
        header.write_var_int(bytes.len() as i32);
        self.stream.get_mut().write_all(&header.copy_bytes())?;
        self.stream.get_mut().write_all(&bytes)
    }

    pub fn play_timeout(&mut self) -> io::Result<()> {
        self.stream
            .get_ref()
            .set_read_timeout(Some(Duration::from_millis(250)))
    }

    pub fn ready(&mut self) -> io::Result<bool> {
        match self.stream.fill_buf() {
            Ok([]) => Err(io::Error::from(io::ErrorKind::UnexpectedEof)),
            Ok(_) => Ok(true),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                ) =>
            {
                Ok(false)
            }
            Err(error) => Err(error),
        }
    }
}

fn read_varint(reader: &mut impl Read, max_bytes: usize) -> io::Result<i32> {
    let mut value = 0u32;
    for index in 0..max_bytes {
        let mut byte = [0];
        reader.read_exact(&mut byte)?;
        if index == 4 && byte[0] > 15 {
            break;
        }
        value |= u32::from(byte[0] & 127) << (7 * index);
        if byte[0] & 128 == 0 {
            return Ok(value as i32);
        }
    }
    Err(invalid("VarInt is too large"))
}

pub struct Cursor(io::Cursor<Vec<u8>>);

impl Cursor {
    fn new(bytes: Vec<u8>) -> Self {
        Self(io::Cursor::new(bytes))
    }

    pub fn varint(&mut self) -> io::Result<i32> {
        read_varint(&mut self.0, 5)
    }

    pub fn string(&mut self, max: usize) -> io::Result<String> {
        let len = usize::try_from(self.varint()?).map_err(|_| invalid("negative string length"))?;
        if len > self.remaining() || len > max * 3 {
            return Err(invalid("invalid string length"));
        }
        let mut field = vec![0; len];
        self.0.read_exact(&mut field)?;
        let text = String::from_utf8(field).map_err(|_| invalid("invalid UTF-8"))?;
        if text.encode_utf16().count() > max {
            return Err(invalid("string too long"));
        }
        Ok(text)
    }

    fn remaining(&self) -> usize {
        self.0.get_ref().len() - self.0.position() as usize
    }

    pub fn fixed<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        let mut field = [0; N];
        self.0.read_exact(&mut field)?;
        Ok(field)
    }

    pub fn finish(self) -> io::Result<()> {
        if self.remaining() != 0 {
            return Err(invalid("unexpected trailing packet data"));
        }
        Ok(())
    }
}

pub fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
