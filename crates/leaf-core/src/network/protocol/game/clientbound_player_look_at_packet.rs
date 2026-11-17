//! Перенос `ClientboundPlayerLookAtPacket`.

use crate::entity::entity_anchor::Anchor;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundPlayerLookAtPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundPlayerLookAtPacket {
    /// `fromAnchor`.
    pub from_anchor: Anchor,
    /// `x`.
    pub x: f64,
    /// `y`.
    pub y: f64,
    /// `z`.
    pub z: f64,
    /// `atEntity`.
    pub at_entity: bool,
    /// `entity` (id, если `at_entity`).
    pub entity: i32,
    /// `toAnchor` (`at_entity`).
    pub to_anchor: Option<Anchor>,
}

impl ClientboundPlayerLookAtPacket {
    /// `new(fromAnchor, x, y, z)` — точка в мире.
    pub fn at_point(from_anchor: Anchor, x: f64, y: f64, z: f64) -> Self {
        Self {
            from_anchor,
            x,
            y,
            z,
            at_entity: false,
            entity: 0,
            to_anchor: None,
        }
    }

    /// `new(fromAnchor, entityId, x, y, z, toAnchor)` — сущность.
    pub fn at_entity(
        from_anchor: Anchor,
        entity: i32,
        x: f64,
        y: f64,
        z: f64,
        to_anchor: Anchor,
    ) -> Self {
        Self {
            from_anchor,
            x,
            y,
            z,
            at_entity: true,
            entity,
            to_anchor: Some(to_anchor),
        }
    }

    /// `STREAM_CODEC` — `Packet.codec`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundPlayerLookAtPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundPlayerLookAtPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let from_anchor = Anchor::by_id(buf.read_byte() as i32);
                let x = buf.read_double();
                let y = buf.read_double();
                let z = buf.read_double();
                let at_entity = buf.read_bool();
                if at_entity {
                    let entity = buf.read_var_int();
                    let to_anchor = Anchor::by_id(buf.read_byte() as i32);
                    ClientboundPlayerLookAtPacket::at_entity(from_anchor, entity, x, y, z, to_anchor)
                } else {
                    ClientboundPlayerLookAtPacket::at_point(from_anchor, x, y, z)
                }
            },
        )
    }

    /// `getFromAnchor`.
    pub fn get_from_anchor(&self) -> Anchor {
        self.from_anchor
    }
}

impl Packet for ClientboundPlayerLookAtPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_player_look_at::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_byte(self.from_anchor.ordinal());
        buf.write_double(self.x);
        buf.write_double(self.y);
        buf.write_double(self.z);
        buf.write_bool(self.at_entity);
        if self.at_entity {
            buf.write_var_int(self.entity);
            buf.write_byte(self.to_anchor.unwrap_or(Anchor::Feet).ordinal());
        }
    }
}
