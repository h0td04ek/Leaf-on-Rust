//! Перенос `ClientboundRecipeBookSettingsPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::stats::recipe_book_settings::RecipeBookSettings;

/// `ClientboundRecipeBookSettingsPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundRecipeBookSettingsPacket {
    /// `bookSettings`.
    pub book_settings: RecipeBookSettings,
}

impl ClientboundRecipeBookSettingsPacket {
    /// `new`.
    pub fn new(book_settings: RecipeBookSettings) -> Self {
        Self { book_settings }
    }

    /// `bookSettings`.
    pub fn book_settings(&self) -> &RecipeBookSettings {
        &self.book_settings
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundRecipeBookSettingsPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundRecipeBookSettingsPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let book_settings = RecipeBookSettings::stream_codec().decode(buf);
                ClientboundRecipeBookSettingsPacket::new(book_settings)
            },
        )
    }
}

impl Packet for ClientboundRecipeBookSettingsPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_recipe_book_settings::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        RecipeBookSettings::stream_codec().encode(buf, &self.book_settings);
    }
}
