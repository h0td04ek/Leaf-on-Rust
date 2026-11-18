//! `ClientboundRecipeBookAddPacket` — добавление рецептов в книгу.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr, PacketListenerObj};
use crate::world::item::crafting::display::recipe_display_entry::RecipeDisplayEntry;

/// `ClientboundRecipeBookAddPacket`.
pub struct ClientboundRecipeBookAddPacket {
    /// `entries`.
    pub entries: Vec<Entry>,
    /// `replace`.
    pub replace: bool,
}

/// `Entry` — `{ contents, flags }`.
#[derive(Clone, Debug)]
pub struct Entry {
    /// `contents`.
    pub contents: RecipeDisplayEntry,
    /// `flags`.
    pub flags: u8,
}

impl Entry {
    /// `FLAG_NOTIFICATION`.
    pub const FLAG_NOTIFICATION: u8 = 1;
    /// `FLAG_HIGHLIGHT`.
    pub const FLAG_HIGHLIGHT: u8 = 2;

    /// `new(contents, flags)`.
    pub fn new(contents: RecipeDisplayEntry, flags: u8) -> Self {
        Self { contents, flags }
    }

    /// `new(contents, notification, highlight)`.
    pub fn with_flags(contents: RecipeDisplayEntry, notification: bool, highlight: bool) -> Self {
        let flags = (if notification { Self::FLAG_NOTIFICATION } else { 0 })
            | (if highlight { Self::FLAG_HIGHLIGHT } else { 0 });
        Self::new(contents, flags)
    }

    /// `notification()`.
    pub fn notification(&self) -> bool {
        (self.flags & Self::FLAG_NOTIFICATION) != 0
    }

    /// `highlight()`.
    pub fn highlight(&self) -> bool {
        (self.flags & Self::FLAG_HIGHLIGHT) != 0
    }

    /// `encode`.
    pub fn encode(&self, buf: &mut FriendlyByteBuf) {
        self.contents.encode(buf);
        buf.write_byte(self.flags as i32);
    }

    /// `decode`.
    pub fn decode(buf: &mut FriendlyByteBuf) -> Self {
        let contents = RecipeDisplayEntry::decode(buf);
        let flags = buf.read_byte() as u8;
        Self::new(contents, flags)
    }
}

impl ClientboundRecipeBookAddPacket {
    /// `new(entries, replace)`.
    pub fn new(entries: Vec<Entry>, replace: bool) -> Self {
        Self { entries, replace }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundRecipeBookAddPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundRecipeBookAddPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let entries = buf.read_collection(|n| Vec::with_capacity(n), |b| Entry::decode(b));
                let replace = buf.read_bool();
                Self::new(entries, replace)
            },
        )
    }
}

impl Packet for ClientboundRecipeBookAddPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_recipe_book_add::<Self>().erase()
    }

    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_collection(&self.entries, |b, e| e.encode(b));
        buf.write_bool(self.replace);
    }

    fn handle(&self, _handler: &mut dyn PacketListenerObj) {}
}
