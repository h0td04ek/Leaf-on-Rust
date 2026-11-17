//! Перенос `ServerboundRecipeBookSeenRecipePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::item::crafting::display::RecipeDisplayId;

/// `ServerboundRecipeBookSeenRecipePacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundRecipeBookSeenRecipePacket {
    /// `recipe`.
    pub recipe: RecipeDisplayId,
}

impl ServerboundRecipeBookSeenRecipePacket {
    /// `new`.
    pub fn new(recipe: RecipeDisplayId) -> Self {
        Self { recipe }
    }

    /// `recipe`.
    pub fn recipe(&self) -> RecipeDisplayId {
        self.recipe
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundRecipeBookSeenRecipePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundRecipeBookSeenRecipePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let recipe = RecipeDisplayId::stream_codec().decode(buf);
                ServerboundRecipeBookSeenRecipePacket::new(recipe)
            },
        )
    }
}

impl Packet for ServerboundRecipeBookSeenRecipePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_recipe_book_seen_recipe::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        RecipeDisplayId::stream_codec().encode(buf, &self.recipe);
    }
}
