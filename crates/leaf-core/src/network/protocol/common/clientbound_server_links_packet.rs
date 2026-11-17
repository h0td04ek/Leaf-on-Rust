//! Перенос `net.minecraft.server.ServerLinks` + `ClientboundServerLinksPacket`.

use crate::network::chat::component::Component;
use crate::network::chat::component_json;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use crate::util::either::Either;
use std::sync::LazyLock;

static SERVER_LINKS_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("server_links").unwrap())
});

/// `ServerLinks` — список ссылок сервера.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ServerLinks {
    pub entries: Vec<Entry>,
}

impl ServerLinks {
    /// `EMPTY`.
    pub fn empty() -> ServerLinks {
        ServerLinks { entries: Vec::new() }
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// `findKnownType`.
    pub fn find_known_type(&self, type_: KnownLinkType) -> Option<&Entry> {
        self.entries.iter().find(|e| e.type_ == Either::Left(type_))
    }

    /// `untrust` — тип + link как строки.
    pub fn untrust(&self) -> Vec<UntrustedEntry> {
        self.entries
            .iter()
            .map(|e| UntrustedEntry {
                type_: e.type_.clone(),
                link: e.link.clone(),
            })
            .collect()
    }
}

/// `ServerLinks.Entry` — (type, link-URI).
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// `type` — Either<KnownLinkType, Component>.
    pub type_: Either<KnownLinkType, Component>,
    /// `link` — URI строкой.
    pub link: String,
}

impl Entry {
    /// `knownType(type, uri)`.
    pub fn known_type(type_: KnownLinkType, link: impl Into<String>) -> Entry {
        Entry { type_: Either::Left(type_), link: link.into() }
    }

    /// `custom(type, uri)`.
    pub fn custom(type_: Component, link: impl Into<String>) -> Entry {
        Entry { type_: Either::Right(type_), link: link.into() }
    }

    /// `displayName`.
    pub fn display_name(&self) -> Component {
        match &self.type_ {
            Either::Left(t) => t.display_name(),
            Either::Right(c) => c.clone(),
        }
    }
}

/// `ServerLinks.KnownLinkType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KnownLinkType {
    BugReport,
    CommunityGuidelines,
    Support,
    Status,
    Feedback,
    Community,
    Website,
    Forums,
    News,
    Announcements,
}

impl KnownLinkType {
    /// Порядок в `values()` (id = ordinal).
    pub const VALUES: [KnownLinkType; 10] = [
        KnownLinkType::BugReport,
        KnownLinkType::CommunityGuidelines,
        KnownLinkType::Support,
        KnownLinkType::Status,
        KnownLinkType::Feedback,
        KnownLinkType::Community,
        KnownLinkType::Website,
        KnownLinkType::Forums,
        KnownLinkType::News,
        KnownLinkType::Announcements,
    ];

    /// `id`.
    pub fn id(&self) -> i32 {
        *self as i32
    }

    /// `BY_ID` (ZERO).
    pub fn by_id(id: i32) -> KnownLinkType {
        KnownLinkType::VALUES[id.clamp(0, 9) as usize]
    }

    /// serialized name (`report_bug` и т.д.).
    pub fn serialized_name(&self) -> &'static str {
        match self {
            KnownLinkType::BugReport => "report_bug",
            KnownLinkType::CommunityGuidelines => "community_guidelines",
            KnownLinkType::Support => "support",
            KnownLinkType::Status => "status",
            KnownLinkType::Feedback => "feedback",
            KnownLinkType::Community => "community",
            KnownLinkType::Website => "website",
            KnownLinkType::Forums => "forums",
            KnownLinkType::News => "news",
            KnownLinkType::Announcements => "announcements",
        }
    }

    /// `displayName` — `known_server_link.<name>`.
    pub fn display_name(&self) -> Component {
        Component::translatable(
            &format!("known_server_link.{}", self.serialized_name()),
            Vec::new(),
        )
    }
}

/// `ServerLinks.UntrustedEntry`.
#[derive(Clone, Debug, PartialEq)]
pub struct UntrustedEntry {
    pub type_: Either<KnownLinkType, Component>,
    pub link: String,
}

/// `TYPE_STREAM_CODEC` — either(KnownLinkType varint, Component json).
fn write_entry_type(buf: &mut FriendlyByteBuf, type_: &Either<KnownLinkType, Component>) {
    match type_ {
        Either::Left(t) => {
            buf.write_bool(true);
            buf.write_byte(t.id());
        }
        Either::Right(c) => {
            buf.write_bool(false);
            component_json::component_stream_codec().encode(buf, c);
        }
    }
}

fn read_entry_type(buf: &mut FriendlyByteBuf) -> Either<KnownLinkType, Component> {
    if buf.read_bool() {
        Either::Left(KnownLinkType::by_id(buf.read_byte() as i32))
    } else {
        Either::Right(component_json::component_stream_codec().decode(buf))
    }
}

/// `UNTRUSTED_LINKS_STREAM_CODEC` — list(UntrustedEntry).
fn write_untrusted_links(buf: &mut FriendlyByteBuf, links: &[UntrustedEntry]) {
    buf.write_var_int(links.len() as i32);
    for link in links {
        write_entry_type(buf, &link.type_);
        buf.write_utf(&link.link).expect("link too long");
    }
}

fn read_untrusted_links(buf: &mut FriendlyByteBuf) -> Vec<UntrustedEntry> {
    let count = buf.read_var_int();
    let mut links = Vec::with_capacity(count.max(0) as usize);
    for _ in 0..count {
        let type_ = read_entry_type(buf);
        let link = buf.read_utf().expect("invalid link");
        links.push(UntrustedEntry { type_, link });
    }
    links
}

/// `ClientboundServerLinksPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundServerLinksPacket {
    pub links: Vec<UntrustedEntry>,
}

impl ClientboundServerLinksPacket {
    pub fn new(links: Vec<UntrustedEntry>) -> Self {
        Self { links }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundServerLinksPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundServerLinksPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundServerLinksPacket::new(read_untrusted_links(buf))
            },
        )
    }
}

impl Packet for ClientboundServerLinksPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        SERVER_LINKS_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        write_untrusted_links(buf, &self.links);
    }
}
