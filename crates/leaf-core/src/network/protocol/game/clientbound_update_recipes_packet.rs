//! `ClientboundUpdateRecipesPacket` — обновление рецептов.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr, PacketListenerObj};
use crate::resources::identifier::Identifier;
use crate::world::item::crafting::recipe_property_set::RecipePropertySet;
use crate::world::item::crafting::selectable_recipe::SingleInputSet;
use std::collections::HashMap;

/// `ClientboundUpdateRecipesPacket`.
pub struct ClientboundUpdateRecipesPacket {
    /// `itemSets` — map ResourceKey<RecipePropertySet> → RecipePropertySet.
    pub item_sets: HashMap<Identifier, RecipePropertySet>,
    /// `stonecutterRecipes` — SelectableRecipe.SingleInputSet<StonecutterRecipe>.
    pub stonecutter_recipes: SingleInputSet,
}

impl ClientboundUpdateRecipesPacket {
    /// `new(itemSets, stonecutterRecipes)`.
    pub fn new(item_sets: HashMap<Identifier, RecipePropertySet>, stonecutter_recipes: SingleInputSet) -> Self {
        Self { item_sets, stonecutter_recipes }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundUpdateRecipesPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundUpdateRecipesPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let item_sets = buf.read_map(
                    |n| HashMap::with_capacity(n),
                    |b| b.read_identifier().unwrap_or_else(|_| Identifier::from_namespace_and_path("minecraft", "unknown").unwrap()),
                    |b| RecipePropertySet::decode(b),
                );
                let stonecutter_recipes = SingleInputSet::decode_no_recipe(buf);
                Self::new(item_sets, stonecutter_recipes)
            },
        )
    }
}

impl Packet for ClientboundUpdateRecipesPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_update_recipes::<Self>().erase()
    }

    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_map(&self.item_sets, |b, k| {
            let _ = b.write_identifier(k);
        }, |b, v| {
            v.encode(b);
        });
        self.stonecutter_recipes.encode_no_recipe(buf);
    }

    fn handle(&self, _handler: &mut dyn PacketListenerObj) {}
}
