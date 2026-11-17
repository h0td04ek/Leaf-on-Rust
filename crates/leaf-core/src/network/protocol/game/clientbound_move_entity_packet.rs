//! Перенос `ClientboundMoveEntityPacket` (+ вложенные `Pos`, `PosRot`, `Rot`).

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::util::mth;

/// `ClientboundMoveEntityPacket` — абстрактный базовый пакет движения сущности.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundMoveEntityPacket {
    /// `entityId`.
    pub entity_id: i32,
    /// `xa`.
    pub xa: i16,
    /// `ya`.
    pub ya: i16,
    /// `za`.
    pub za: i16,
    /// `yRot` (packed byte).
    pub y_rot: i8,
    /// `xRot` (packed byte).
    pub x_rot: i8,
    /// `onGround`.
    pub on_ground: bool,
    /// `hasRot`.
    pub has_rot: bool,
    /// `hasPos`.
    pub has_pos: bool,
}

impl ClientboundMoveEntityPacket {
    /// Базовый конструктор.
    pub fn new(
        entity_id: i32,
        xa: i16,
        ya: i16,
        za: i16,
        y_rot: i8,
        x_rot: i8,
        on_ground: bool,
        has_rot: bool,
        has_pos: bool,
    ) -> Self {
        Self { entity_id, xa, ya, za, y_rot, x_rot, on_ground, has_rot, has_pos }
    }

    /// `getXa`.
    pub fn get_xa(&self) -> i16 {
        self.xa
    }

    /// `getYa`.
    pub fn get_ya(&self) -> i16 {
        self.ya
    }

    /// `getZa`.
    pub fn get_za(&self) -> i16 {
        self.za
    }

    /// `getYRot` (`Mth.unpackDegrees`).
    pub fn get_y_rot(&self) -> f32 {
        mth::unpack_degrees(self.y_rot)
    }

    /// `getXRot`.
    pub fn get_x_rot(&self) -> f32 {
        mth::unpack_degrees(self.x_rot)
    }

    /// `hasRotation`.
    pub fn has_rotation(&self) -> bool {
        self.has_rot
    }

    /// `hasPosition`.
    pub fn has_position(&self) -> bool {
        self.has_pos
    }

    /// `isOnGround`.
    pub fn is_on_ground(&self) -> bool {
        self.on_ground
    }

    /// Запись общих полей.
    fn write_common(&self, buf: &mut FriendlyByteBuf, with_pos: bool, with_rot: bool) {
        buf.write_var_int(self.entity_id);
        if with_pos {
            buf.write_short(self.xa as i32);
            buf.write_short(self.ya as i32);
            buf.write_short(self.za as i32);
        }
        if with_rot {
            buf.write_byte(self.y_rot as i32);
            buf.write_byte(self.x_rot as i32);
        }
        buf.write_bool(self.on_ground);
    }
}

/// `ClientboundMoveEntityPacket.Pos` — только позиция.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PosPacket {
    inner: ClientboundMoveEntityPacket,
}

impl PosPacket {
    /// `Pos::new`.
    pub fn new(entity_id: i32, xa: i16, ya: i16, za: i16, on_ground: bool) -> Self {
        Self {
            inner: ClientboundMoveEntityPacket::new(
                entity_id, xa, ya, za, 0, 0, on_ground, false, true,
            ),
        }
    }

    /// Доступ к общим полям.
    pub fn inner(&self) -> &ClientboundMoveEntityPacket {
        &self.inner
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, PosPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &PosPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let entity_id = buf.read_var_int();
                let xa = buf.read_short();
                let ya = buf.read_short();
                let za = buf.read_short();
                let on_ground = buf.read_bool();
                PosPacket::new(entity_id, xa, ya, za, on_ground)
            },
        )
    }
}

impl Packet for PosPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_move_entity_pos::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        self.inner.write_common(buf, true, false);
    }
}

/// `ClientboundMoveEntityPacket.PosRot` — позиция + поворот.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PosRotPacket {
    inner: ClientboundMoveEntityPacket,
}

impl PosRotPacket {
    /// `PosRot::new`.
    pub fn new(
        entity_id: i32,
        xa: i16,
        ya: i16,
        za: i16,
        y_rot: i8,
        x_rot: i8,
        on_ground: bool,
    ) -> Self {
        Self {
            inner: ClientboundMoveEntityPacket::new(
                entity_id, xa, ya, za, y_rot, x_rot, on_ground, true, true,
            ),
        }
    }

    /// Доступ к общим полям.
    pub fn inner(&self) -> &ClientboundMoveEntityPacket {
        &self.inner
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, PosRotPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &PosRotPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let entity_id = buf.read_var_int();
                let xa = buf.read_short();
                let ya = buf.read_short();
                let za = buf.read_short();
                let y_rot = buf.read_byte();
                let x_rot = buf.read_byte();
                let on_ground = buf.read_bool();
                PosRotPacket::new(entity_id, xa, ya, za, y_rot, x_rot, on_ground)
            },
        )
    }
}

impl Packet for PosRotPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_move_entity_pos_rot::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        self.inner.write_common(buf, true, true);
    }
}

/// `ClientboundMoveEntityPacket.Rot` — только поворот.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RotPacket {
    inner: ClientboundMoveEntityPacket,
}

impl RotPacket {
    /// `Rot::new`.
    pub fn new(entity_id: i32, y_rot: i8, x_rot: i8, on_ground: bool) -> Self {
        Self {
            inner: ClientboundMoveEntityPacket::new(
                entity_id, 0, 0, 0, y_rot, x_rot, on_ground, true, false,
            ),
        }
    }

    /// Доступ к общим полям.
    pub fn inner(&self) -> &ClientboundMoveEntityPacket {
        &self.inner
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, RotPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &RotPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let entity_id = buf.read_var_int();
                let y_rot = buf.read_byte();
                let x_rot = buf.read_byte();
                let on_ground = buf.read_bool();
                RotPacket::new(entity_id, y_rot, x_rot, on_ground)
            },
        )
    }
}

impl Packet for RotPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_move_entity_rot::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        self.inner.write_common(buf, false, true);
    }
}
