//! Перенос `ServerboundRecipeBookChangeSettingsPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::inventory::recipe_book_type::RecipeBookType;

/// `ServerboundRecipeBookChangeSettingsPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundRecipeBookChangeSettingsPacket {
    /// `bookType`.
    pub book_type: RecipeBookType,
    /// `isOpen`.
    pub is_open: bool,
    /// `isFiltering`.
    pub is_filtering: bool,
}

impl ServerboundRecipeBookChangeSettingsPacket {
    /// `new`.
    pub fn new(book_type: RecipeBookType, is_open: bool, is_filtering: bool) -> Self {
        Self { book_type, is_open, is_filtering }
    }

    /// `getBookType`/`isOpen`/`isFiltering`.
    pub fn get_book_type(&self) -> RecipeBookType {
        self.book_type
    }
    /// `isOpen`.
    pub fn is_open(&self) -> bool {
        self.is_open
    }
    /// `isFiltering`.
    pub fn is_filtering(&self) -> bool {
        self.is_filtering
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundRecipeBookChangeSettingsPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundRecipeBookChangeSettingsPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let ordinal = buf.read_var_int();
                let book_type = RecipeBookType::by_ordinal(ordinal);
                let is_open = buf.read_bool();
                let is_filtering = buf.read_bool();
                ServerboundRecipeBookChangeSettingsPacket::new(book_type, is_open, is_filtering)
            },
        )
    }
}

impl Packet for ServerboundRecipeBookChangeSettingsPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_recipe_book_change_settings::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.book_type as i32);
        buf.write_bool(self.is_open);
        buf.write_bool(self.is_filtering);
    }
}
