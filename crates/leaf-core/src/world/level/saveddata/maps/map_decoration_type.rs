//! `MapDecorationType` — тип декорации карты.

use crate::resources::Identifier;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::core::holder::Holder;

/// `MapDecorationType` — `{ assetId, showOnItemFrame, mapColor, explorationMapElement, trackCount }`.
#[derive(Clone, Debug)]
pub struct MapDecorationType {
    /// `assetId`.
    pub asset_id: Identifier,
    /// `showOnItemFrame`.
    pub show_on_item_frame: bool,
    /// `mapColor` — `-1` = no color.
    pub map_color: i32,
    /// `explorationMapElement`.
    pub exploration_map_element: bool,
    /// `trackCount`.
    pub track_count: bool,
}

impl MapDecorationType {
    /// `NO_MAP_COLOR`.
    pub const NO_MAP_COLOR: i32 = -1;

    /// `new`.
    pub fn new(
        asset_id: Identifier,
        show_on_item_frame: bool,
        map_color: i32,
        exploration_map_element: bool,
        track_count: bool,
    ) -> Self {
        Self { asset_id, show_on_item_frame, map_color, exploration_map_element, track_count }
    }

    /// `hasMapColor`.
    pub fn has_map_color(&self) -> bool {
        self.map_color != Self::NO_MAP_COLOR
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.holderRegistry(Registries.MAP_DECORATION_TYPE)`.
    /// Type-erased: `Holder<MapDecorationType>` через registry id.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<MapDecorationType>> + Send + Sync + 'static {
        crate::core::holder::holder_registry_stream_codec()
    }
}
