//! `ClientboundMapItemDataPacket` — данные карты.

use std::sync::Arc;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::world::level::saveddata::maps::{MapId, MapDecoration, MapPatch};

/// `ClientboundMapItemDataPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundMapItemDataPacket {
    /// `mapId`.
    pub map_id: MapId,
    /// `scale`.
    pub scale: i8,
    /// `locked`.
    pub locked: bool,
    /// `decorations` — `Optional<List<MapDecoration>>`.
    pub decorations: Option<Vec<MapDecoration>>,
    /// `colorPatch` — `Optional<MapPatch>`.
    pub color_patch: Option<MapPatch>,
}

impl ClientboundMapItemDataPacket {
    /// `new(mapId, scale, locked, decorations, colorPatch)`.
    pub fn new(
        map_id: MapId,
        scale: i8,
        locked: bool,
        decorations: Option<Vec<MapDecoration>>,
        color_patch: Option<MapPatch>,
    ) -> Self {
        Self { map_id, scale, locked, decorations, color_patch }
    }

    /// `STREAM_CODEC` — `StreamCodec.composite(MapId, BYTE, BOOL, list(MapDecoration).optional, MapPatch, new)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundMapItemDataPacket> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, pkt: &ClientboundMapItemDataPacket| {
                MapId::stream_codec().encode(buf, &pkt.map_id);
                buf.write_byte(pkt.scale as i32);
                buf.write_bool(pkt.locked);
                // decorations — optional list
                match &pkt.decorations {
                    Some(decs) => {
                        buf.write_bool(true);
                        buf.write_var_int(decs.len() as i32);
                        for d in decs {
                            MapDecoration::stream_codec().encode(buf, d);
                        }
                    },
                    None => { buf.write_bool(false); },
                }
                // colorPatch — optional
                MapPatch::stream_codec().encode(buf, &pkt.color_patch);
            },
            |buf: &mut FriendlyByteBuf| {
                let map_id = MapId::stream_codec().decode(buf);
                let scale = buf.read_byte();
                let locked = buf.read_bool();
                let has_decs = buf.read_bool();
                let decorations = if has_decs {
                    let len = buf.read_var_int();
                    let mut decs = Vec::new();
                    for _ in 0..len {
                        decs.push(MapDecoration::stream_codec().decode(buf));
                    }
                    Some(decs)
                } else {
                    None
                };
                let color_patch = MapPatch::stream_codec().decode(buf);
                Self::new(map_id, scale, locked, decorations, color_patch)
            },
        )
    }
}

impl Packet for ClientboundMapItemDataPacket {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_map_item_data::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        MapId::stream_codec().encode(buf, &self.map_id);
        buf.write_byte(self.scale as i32);
        buf.write_bool(self.locked);
        match &self.decorations {
            Some(decs) => {
                buf.write_bool(true);
                buf.write_var_int(decs.len() as i32);
                for d in decs {
                    MapDecoration::stream_codec().encode(buf, d);
                }
            },
            None => { buf.write_bool(false); },
        }
        MapPatch::stream_codec().encode(buf, &self.color_patch);
    }
}
