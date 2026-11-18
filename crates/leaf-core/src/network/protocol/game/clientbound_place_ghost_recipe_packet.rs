//! `ClientboundPlaceGhostRecipePacket` — ghost recipe в инвентаре.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr, PacketListenerObj};
use crate::world::item::crafting::display::recipe_display::RecipeDisplay;

/// `ClientboundPlaceGhostRecipePacket`.
pub struct ClientboundPlaceGhostRecipePacket {
    /// `containerId`.
    pub container_id: i32,
    /// `recipeDisplay`.
    pub recipe_display: RecipeDisplay,
}

impl ClientboundPlaceGhostRecipePacket {
    /// `new(containerId, recipeDisplay)`.
    pub fn new(container_id: i32, recipe_display: RecipeDisplay) -> Self {
        Self { container_id, recipe_display }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundPlaceGhostRecipePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundPlaceGhostRecipePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let container_id = buf.read_var_int();
                let recipe_display = RecipeDisplay::decode(buf);
                Self::new(container_id, recipe_display)
            },
        )
    }
}

impl Packet for ClientboundPlaceGhostRecipePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_place_ghost_recipe::<Self>().erase()
    }

    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.container_id);
        self.recipe_display.encode(buf);
    }

    fn handle(&self, _handler: &mut dyn PacketListenerObj) {}
}
