//! Перенос `ClientboundRecipeBookRemovePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::item::crafting::display::RecipeDisplayId;

/// `ClientboundRecipeBookRemovePacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundRecipeBookRemovePacket {
    /// `recipes`.
    pub recipes: Vec<RecipeDisplayId>,
}

impl ClientboundRecipeBookRemovePacket {
    /// `new`.
    pub fn new(recipes: Vec<RecipeDisplayId>) -> Self {
        Self { recipes }
    }

    /// `recipes`.
    pub fn recipes(&self) -> &[RecipeDisplayId] {
        &self.recipes
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundRecipeBookRemovePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundRecipeBookRemovePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let recipes = buf.read_list(|b| RecipeDisplayId::stream_codec().decode(b));
                ClientboundRecipeBookRemovePacket::new(recipes)
            },
        )
    }
}

impl Packet for ClientboundRecipeBookRemovePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_recipe_book_remove::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_collection(&self.recipes, |b, id| {
            RecipeDisplayId::stream_codec().encode(b, id);
        });
    }
}
