//! `ClientboundCommandsPacket` — отправка дерева команд клиенту.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr, PacketListenerObj};
use crate::resources::identifier::Identifier;
use std::collections::HashMap;

/// `ClientboundCommandsPacket`.
pub struct ClientboundCommandsPacket {
    /// `rootIndex`.
    pub root_index: i32,
    /// `entries`.
    pub entries: Vec<Entry>,
}

/// `Entry` — запись узла дерева команд.
#[derive(Clone, Debug)]
pub struct Entry {
    /// `stub` — тип узла (None = root).
    pub stub: Option<NodeStub>,
    /// `flags`.
    pub flags: u8,
    /// `redirect`.
    pub redirect: i32,
    /// `children` — индексы дочерних узлов.
    pub children: Vec<i32>,
}

/// `NodeStub` — заглушка узла (literal или argument).
#[derive(Clone, Debug)]
pub enum NodeStub {
    /// `LiteralNodeStub`.
    Literal(LiteralNodeStub),
    /// `ArgumentNodeStub`.
    Argument(ArgumentNodeStub),
}

/// `LiteralNodeStub`.
#[derive(Clone, Debug)]
pub struct LiteralNodeStub {
    /// `id` — литерал.
    pub id: String,
}

/// `ArgumentNodeStub`.
#[derive(Clone, Debug)]
pub struct ArgumentNodeStub {
    /// `id` — имя аргумента.
    pub id: String,
    /// `argumentType` — тип аргумента (registry id).
    pub argument_type: i32,
    /// `suggestionId` — ID suggestions (optional).
    pub suggestion_id: Option<Identifier>,
}

// Флаги
pub const MASK_TYPE: u8 = 3;
pub const FLAG_EXECUTABLE: u8 = 4;
pub const FLAG_REDIRECT: u8 = 8;
pub const FLAG_CUSTOM_SUGGESTIONS: u8 = 16;
pub const FLAG_RESTRICTED: u8 = 32;
pub const TYPE_ROOT: u8 = 0;
pub const TYPE_LITERAL: u8 = 1;
pub const TYPE_ARGUMENT: u8 = 2;

impl ClientboundCommandsPacket {
    /// `new(rootIndex, entries)`.
    pub fn new(root_index: i32, entries: Vec<Entry>) -> Self {
        Self { root_index, entries }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundCommandsPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundCommandsPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let entries = buf.read_collection(|n| Vec::with_capacity(n), |b| Self::read_node(b));
                let root_index = buf.read_var_int();
                Self::new(root_index, entries)
            },
        )
    }

    /// `readNode`.
    fn read_node(buf: &mut FriendlyByteBuf) -> Entry {
        let flags = buf.read_byte() as u8;
        let children = buf.read_var_int_array();
        let redirect = if (flags & FLAG_REDIRECT) != 0 {
            buf.read_var_int()
        } else {
            0
        };
        let stub = Self::read_stub(buf, flags);
        Entry { stub, flags, redirect, children }
    }

    /// `read` — чтение NodeStub.
    fn read_stub(buf: &mut FriendlyByteBuf, flags: u8) -> Option<NodeStub> {
        let node_type = flags & MASK_TYPE;
        match node_type {
            TYPE_ARGUMENT => {
                let id = buf.read_utf().unwrap_or_default();
                let arg_type = buf.read_var_int();
                // ArgumentTypeInfo.deserializeFromNetwork — упрощённо
                let suggestion_id = if (flags & FLAG_CUSTOM_SUGGESTIONS) != 0 {
                    Some(buf.read_identifier().unwrap_or_else(|_| {
                        Identifier::from_namespace_and_path("minecraft", "unknown").unwrap()
                    }))
                } else {
                    None
                };
                Some(NodeStub::Argument(ArgumentNodeStub {
                    id,
                    argument_type: arg_type,
                    suggestion_id,
                }))
            }
            TYPE_LITERAL => {
                let id = buf.read_utf().unwrap_or_default();
                Some(NodeStub::Literal(LiteralNodeStub { id }))
            }
            _ => None,
        }
    }

    /// `enumerateNodes` — BFS по дереву команд.
    /// Требует Brigadier CommandNode — будет добавлено после порта.
    pub fn enumerate_nodes<S>(_root: &S) -> HashMap<usize, usize> {
        HashMap::new()
    }

    /// `createEntries` — создание entries из узлов.
    /// Требует Brigadier CommandNode — будет добавлено после порта.
    pub fn create_entries<S>(_nodes: &HashMap<usize, usize>, _inspector: &dyn Fn(&S) -> (bool, bool, Option<Identifier>)) -> Vec<Entry> {
        Vec::new()
    }
}

impl Entry {
    /// `write`.
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_byte(self.flags as i32);
        buf.write_var_int_array(&self.children);
        if (self.flags & FLAG_REDIRECT) != 0 {
            buf.write_var_int(self.redirect);
        }
        if let Some(stub) = &self.stub {
            match stub {
                NodeStub::Literal(l) => {
                    buf.write_utf(&l.id).ok();
                }
                NodeStub::Argument(a) => {
                    buf.write_utf(&a.id).ok();
                    buf.write_var_int(a.argument_type);
                    // ArgumentTypeInfo.serializeToNetwork — упрощённо
                    if let Some(sid) = &a.suggestion_id {
                        let _ = buf.write_identifier(sid);
                    }
                }
            }
        }
    }

    /// `canBuild`.
    pub fn can_build(&self, children: &std::collections::HashSet<i32>) -> bool {
        (self.flags & FLAG_REDIRECT) == 0 || !children.contains(&self.redirect)
    }

    /// `canResolve`.
    pub fn can_resolve(&self, children: &std::collections::HashSet<i32>) -> bool {
        !self.children.iter().any(|&i| children.contains(&i))
    }
}

impl Packet for ClientboundCommandsPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_commands::<Self>().erase()
    }

    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_collection(&self.entries, |b, e| e.write(b));
        buf.write_var_int(self.root_index);
    }

    fn handle(&self, _handler: &mut dyn PacketListenerObj) {}
}
