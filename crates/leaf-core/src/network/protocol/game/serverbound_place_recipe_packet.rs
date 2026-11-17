//! Перенос `ServerboundPlaceRecipePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::item::crafting::display::RecipeDisplayId;

/// `ServerboundPlaceRecipePacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundPlaceRecipePacket {
    /// `containerId`.
    pub container_id: i32,
    /// `recipe`.
    pub recipe: RecipeDisplayId,
    /// `useMaxItems`.
    pub use_max_items: bool,
}

impl ServerboundPlaceRecipePacket {
    /// `new`.
    pub fn new(container_id: i32, recipe: RecipeDisplayId, use_max_items: bool) -> Self {
        Self { container_id, recipe, use_max_items }
    }

    /// `containerId`/`recipe`/`useMaxItems`.
    pub fn container_id(&self) -> i32 {
        self.container_id
    }
    /// `recipe`.
    pub fn recipe(&self) -> RecipeDisplayId {
        self.recipe
    }
    /// `useMaxItems`.
    pub fn use_max_items(&self) -> bool {
        self.use_max_items
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundPlaceRecipePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundPlaceRecipePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let container_id = buf.read_container_id();
                let recipe = RecipeDisplayId::stream_codec().decode(buf);
                let use_max_items = buf.read_bool();
                ServerboundPlaceRecipePacket::new(container_id, recipe, use_max_items)
            },
        )
    }
}

impl Packet for ServerboundPlaceRecipePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_place_recipe::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_container_id(self.container_id);
        RecipeDisplayId::stream_codec().encode(buf, &self.recipe);
        buf.write_bool(self.use_max_items);
    }
}
