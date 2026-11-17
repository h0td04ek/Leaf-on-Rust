//! Перенос `ServerboundEditBookPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `MAX_PAGES` (100) и `MAX_PAGE_LENGTH` (1024), `MAX_TITLE_LENGTH` (32).
pub const MAX_PAGES: usize = 100;
pub const MAX_PAGE_LENGTH: usize = 1024;
pub const MAX_TITLE_LENGTH: usize = 32;

/// `ServerboundEditBookPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundEditBookPacket {
    /// `slot`.
    pub slot: i32,
    /// `pages`.
    pub pages: Vec<String>,
    /// `title` — `None` = режим записи.
    pub title: Option<String>,
}

impl ServerboundEditBookPacket {
    /// `new ServerboundEditBookPacket(slot, pages, title)`.
    pub fn new(slot: i32, pages: Vec<String>, title: Option<String>) -> Self {
        Self { slot, pages, title }
    }

    /// `slot`.
    pub fn slot(&self) -> i32 {
        self.slot
    }

    /// `pages`.
    pub fn pages(&self) -> &[String] {
        &self.pages
    }

    /// `title`.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// `STREAM_CODEC` — VAR_INT + list(utf8(1024), 100) + optional(utf8(32)).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundEditBookPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundEditBookPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let slot = buf.read_var_int();
                let pages = buf
                    .read_collection(|_| Vec::new(), |b| {
                        b.read_utf_n(MAX_PAGE_LENGTH).expect("invalid page")
                    })
                    .into_iter()
                    .take(MAX_PAGES)
                    .collect();
                let title = buf
                    .read_optional(|b| b.read_utf_n(MAX_TITLE_LENGTH))
                    .and_then(|r| r.ok());
                ServerboundEditBookPacket::new(slot, pages, title)
            },
        )
    }
}

impl Packet for ServerboundEditBookPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_edit_book::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.slot);
        buf.write_collection(&self.pages, |b, p| {
            b.write_utf_n(p, MAX_PAGE_LENGTH).expect("page too long");
        });
        buf.write_optional(&self.title, |b, t| {
            b.write_utf_n(t, MAX_TITLE_LENGTH).expect("title too long");
        });
    }
}
