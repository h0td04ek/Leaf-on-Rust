//! `ClientboundCommandsPacket` — отправка дерева команд клиенту.

use crate::command::brigadier::{CommandDispatcher, NodeKind};
use crate::command::brigadier_argument::ArgumentType;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketListenerObj, PacketTypePtr};
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
    /// Данные сетевого шаблона для типов Brigadier.
    pub template: Option<ArgumentType>,
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
        Self {
            root_index,
            entries,
        }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundCommandsPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundCommandsPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let entries =
                    buf.read_collection(|n| Vec::with_capacity(n), |b| Self::read_node(b));
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
        Entry {
            stub,
            flags,
            redirect,
            children,
        }
    }

    /// `read` — чтение NodeStub.
    fn read_stub(buf: &mut FriendlyByteBuf, flags: u8) -> Option<NodeStub> {
        let node_type = flags & MASK_TYPE;
        match node_type {
            TYPE_ARGUMENT => {
                let id = buf.read_utf().unwrap_or_default();
                let arg_type = buf.read_var_int();
                let template = ArgumentType::read_template(arg_type, buf);
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
                    template,
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

    /// Создаёт сетевой граф команд, доступных для источника.
    pub fn from_dispatcher<S>(dispatcher: &CommandDispatcher<S>, source: &S) -> Self {
        let root = dispatcher.root();
        let mut nodes = vec![root];
        let mut indices = HashMap::from([(root, 0)]);
        let mut entries = Vec::new();
        // Индексы по BFS: сначала дети, затем цель перенаправления.
        while let Some(&id) = nodes.get(entries.len()) {
            let node = dispatcher.node(id);
            let mut visit = |id| {
                *indices.entry(id).or_insert_with(|| {
                    let index = i32::try_from(nodes.len()).expect("command tree index overflow");
                    nodes.push(id);
                    index
                })
            };
            let children = node
                .children
                .iter()
                .copied()
                .filter(|&id| dispatcher.can_use(id, source))
                .map(&mut visit)
                .collect();
            let redirect = node.redirect.map(&mut visit);
            let (mut flags, stub) = match node.kind {
                NodeKind::Root => (TYPE_ROOT, None),
                NodeKind::Literal => (
                    TYPE_LITERAL,
                    Some(NodeStub::Literal(LiteralNodeStub {
                        id: node.name.clone(),
                    })),
                ),
                NodeKind::Argument(template) => (
                    TYPE_ARGUMENT,
                    Some(NodeStub::Argument(ArgumentNodeStub {
                        id: node.name.clone(),
                        argument_type: template.registry_id(),
                        template: Some(template),
                        suggestion_id: node.suggestion_provider.clone(),
                    })),
                ),
            };
            if node.executable() {
                flags |= FLAG_EXECUTABLE;
            }
            if redirect.is_some() {
                flags |= FLAG_REDIRECT;
            }
            if matches!(node.kind, NodeKind::Argument(_)) && node.suggestion_provider.is_some() {
                flags |= FLAG_CUSTOM_SUGGESTIONS;
            }
            if node.restricted {
                flags |= FLAG_RESTRICTED;
            }
            entries.push(Entry {
                stub,
                flags,
                redirect: redirect.unwrap_or(0),
                children,
            });
        }
        Self::new(0, entries)
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
                    if let Some(template) = a.template {
                        template.write_template(buf);
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::brigadier::CommandNode;

    #[test]
    fn synchronizes_argument_bounds_and_filters_hidden_commands() {
        let mut dispatcher = CommandDispatcher::<bool>::new();
        let root = dispatcher.root();
        let set = dispatcher
            .add_node(root, CommandNode::literal("set"))
            .unwrap();
        let hidden = dispatcher
            .add_node(
                root,
                CommandNode::literal("admin").requires(|enabled: &bool| *enabled),
            )
            .unwrap();
        dispatcher
            .add_node(root, CommandNode::literal("a").redirects_to(hidden))
            .unwrap();
        let argument = ArgumentType::Integer { min: 1, max: 3 };
        dispatcher
            .add_node(
                set,
                CommandNode::argument("count", argument).executes(|_, _| 1),
            )
            .unwrap();

        let packet = ClientboundCommandsPacket::from_dispatcher(&dispatcher, &false);
        let mut buf = FriendlyByteBuf::new();
        packet.write_packet(&mut buf);
        assert_eq!(
            buf.readable_bytes_vec(),
            [
                3, 0, 1, 1, // Entry count, root
                1, 1, 2, 3, b's', b'e', b't', // Literal
                6, 0, 5, b'c', b'o', b'u', b'n', b't', 3, 3, 0, 0, 0, 1, 0, 0, 0, 3,
                0, // Root index
            ]
        );
        let decoded = ClientboundCommandsPacket::stream_codec().decode(&mut buf);
        assert_eq!(buf.readable_bytes(), 0);
        let Some(NodeStub::Argument(node)) = &decoded.entries[2].stub else {
            panic!("expected an argument node");
        };
        assert_eq!(node.template, Some(argument));
        assert_eq!(node.id, "count");

        let visible = ClientboundCommandsPacket::from_dispatcher(&dispatcher, &true);
        assert_eq!(visible.entries[3].flags, TYPE_LITERAL | FLAG_REDIRECT);
        assert_eq!(visible.entries[3].redirect, 2);
    }
}
