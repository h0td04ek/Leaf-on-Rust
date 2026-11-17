//! Перенос `ServerboundResourcePackPacket` + `ServerboundCustomClickActionPacket`.

use crate::nbt::tag::Tag;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use crate::util::uuid::Uuid;
use std::sync::LazyLock;

// ------------------------------------------------------------ resource_pack

static RESOURCE_PACK_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::SERVERBOUND, Identifier::with_default_namespace("resource_pack").unwrap())
});

/// `ServerboundResourcePackPacket.Action`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourcePackAction {
    SuccessfullyLoaded,
    Declined,
    FailedDownload,
    Accepted,
    Downloaded,
    InvalidUrl,
    FailedReload,
    Discarded,
}

impl ResourcePackAction {
    /// Порядок в `values()`.
    pub const VALUES: [ResourcePackAction; 8] = [
        ResourcePackAction::SuccessfullyLoaded,
        ResourcePackAction::Declined,
        ResourcePackAction::FailedDownload,
        ResourcePackAction::Accepted,
        ResourcePackAction::Downloaded,
        ResourcePackAction::InvalidUrl,
        ResourcePackAction::FailedReload,
        ResourcePackAction::Discarded,
    ];

    /// `isTerminal` — не ACCEPTED и не DOWNLOADED.
    pub fn is_terminal(&self) -> bool {
        !matches!(self, ResourcePackAction::Accepted | ResourcePackAction::Downloaded)
    }

    fn ordinal(&self) -> i32 {
        *self as i32
    }

    fn from_ordinal(i: i32) -> ResourcePackAction {
        ResourcePackAction::VALUES[i.clamp(0, 7) as usize]
    }
}

/// `ServerboundResourcePackPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundResourcePackPacket {
    pub id: Uuid,
    pub action: ResourcePackAction,
}

impl ServerboundResourcePackPacket {
    pub fn new(id: Uuid, action: ResourcePackAction) -> Self {
        Self { id, action }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundResourcePackPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundResourcePackPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_uuid();
                let action = ResourcePackAction::from_ordinal(buf.read_byte() as i32);
                ServerboundResourcePackPacket::new(id, action)
            },
        )
    }
}

impl Packet for ServerboundResourcePackPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        RESOURCE_PACK_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_uuid(&self.id);
        buf.write_byte(self.action.ordinal());
    }
}

// ------------------------------------------------------------ custom_click_action

/// Лимиты `UNTRUSTED_TAG_CODEC` (`optionalTagCodec(NbtAccounter(32768,16))`
/// + `lengthPrefixed(65536)`).
const MAX_CLICK_PAYLOAD_BYTES: usize = 65536;

static CUSTOM_CLICK_ACTION_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::SERVERBOUND, Identifier::with_default_namespace("custom_click_action").unwrap())
});

/// `ServerboundCustomClickActionPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundCustomClickActionPacket {
    pub id: Identifier,
    /// `payload` — Optional<NBT> (untrusted, ≤64KiB).
    pub payload: Option<Tag>,
}

impl ServerboundCustomClickActionPacket {
    pub fn new(id: Identifier, payload: Option<Tag>) -> Self {
        Self { id, payload }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundCustomClickActionPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundCustomClickActionPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_identifier().expect("invalid identifier");
                let payload = read_optional_tag(buf);
                ServerboundCustomClickActionPacket::new(id, payload)
            },
        )
    }
}

/// `optionalTagCodec(untrusted)` + `lengthPrefixed(65536)`:
/// boolean + (varint length + NBT bytes).
fn read_optional_tag(buf: &mut FriendlyByteBuf) -> Option<Tag> {
    if !buf.read_bool() {
        return None;
    }
    let len = buf.read_var_int();
    if !(0..=MAX_CLICK_PAYLOAD_BYTES as i32).contains(&len) {
        panic!("NBT payload too large");
    }
    let bytes: Vec<u8> = (0..len).map(|_| buf.read_byte() as u8).collect();
    let mut reader = crate::nbt::io::Reader::new(&bytes);
    crate::nbt::io::read_any_tag(
        &mut reader,
        &mut crate::nbt::accounter::NbtAccounter::new(32768, 16),
    )
    .ok()
}

fn write_optional_tag(buf: &mut FriendlyByteBuf, tag: &Option<Tag>) {
    match tag {
        None => {
            buf.write_bool(false);
        }
        Some(t) => {
            buf.write_bool(true);
            let bytes = crate::nbt::io::write_any_tag(t).expect("nbt write");
            buf.write_var_int(bytes.len() as i32);
            for b in bytes {
                buf.write_byte(b as i32);
            }
        }
    }
}

impl Packet for ServerboundCustomClickActionPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        CUSTOM_CLICK_ACTION_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_identifier(&self.id).expect("identifier");
        write_optional_tag(buf, &self.payload);
    }
}
