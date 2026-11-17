//! Перенос `ServerboundMovePlayerPacket` (+ вложенные `Pos`, `PosRot`, `Rot`, `StatusOnly`).

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::phys::vec3::Vec3;

const FLAG_ON_GROUND: i32 = 1;
const FLAG_HORIZONTAL_COLLISION: i32 = 2;

fn pack_flags(on_ground: bool, horizontal_collision: bool) -> i32 {
    let mut i = 0;
    if on_ground {
        i |= FLAG_ON_GROUND;
    }
    if horizontal_collision {
        i |= FLAG_HORIZONTAL_COLLISION;
    }
    i
}

fn unpack_on_ground(flags: i32) -> bool {
    flags & FLAG_ON_GROUND != 0
}

fn unpack_horizontal_collision(flags: i32) -> bool {
    flags & FLAG_HORIZONTAL_COLLISION != 0
}

/// `ServerboundMovePlayerPacket` — абстрактная база (в Rust — enum `ServerboundMovePlayerPacketKind`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ServerboundMovePlayerPacketKind {
    /// `Pos`.
    Pos { x: f64, y: f64, z: f64, on_ground: bool, horizontal_collision: bool },
    /// `PosRot`.
    PosRot {
        x: f64,
        y: f64,
        z: f64,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
        horizontal_collision: bool,
    },
    /// `Rot`.
    Rot { y_rot: f32, x_rot: f32, on_ground: bool, horizontal_collision: bool },
    /// `StatusOnly`.
    StatusOnly { on_ground: bool, horizontal_collision: bool },
}

impl ServerboundMovePlayerPacketKind {
    /// `hasPosition`.
    pub fn has_position(&self) -> bool {
        matches!(self, Self::Pos { .. } | Self::PosRot { .. })
    }

    /// `hasRotation`.
    pub fn has_rotation(&self) -> bool {
        matches!(self, Self::PosRot { .. } | Self::Rot { .. })
    }

    /// `isOnGround`.
    pub fn is_on_ground(&self) -> bool {
        match *self {
            Self::Pos { on_ground, .. }
            | Self::PosRot { on_ground, .. }
            | Self::Rot { on_ground, .. }
            | Self::StatusOnly { on_ground, .. } => on_ground,
        }
    }

    /// `horizontalCollision`.
    pub fn horizontal_collision(&self) -> bool {
        match *self {
            Self::Pos { horizontal_collision, .. }
            | Self::PosRot { horizontal_collision, .. }
            | Self::Rot { horizontal_collision, .. }
            | Self::StatusOnly { horizontal_collision, .. } => horizontal_collision,
        }
    }
}

/// `ServerboundMovePlayerPacket.Pos`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovePlayerPosPacket {
    /// `x`.
    pub x: f64,
    /// `y`.
    pub y: f64,
    /// `z`.
    pub z: f64,
    /// `onGround`.
    pub on_ground: bool,
    /// `horizontalCollision`.
    pub horizontal_collision: bool,
}

impl MovePlayerPosPacket {
    /// `new Pos(Vec3, onGround, horizontalCollision)`.
    pub fn new(pos: Vec3, on_ground: bool, horizontal_collision: bool) -> Self {
        Self::new_xyz(pos.x, pos.y, pos.z, on_ground, horizontal_collision)
    }

    /// `new Pos(x, y, z, onGround, horizontalCollision)`.
    pub fn new_xyz(x: f64, y: f64, z: f64, on_ground: bool, horizontal_collision: bool) -> Self {
        Self { x, y, z, on_ground, horizontal_collision }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, MovePlayerPosPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &MovePlayerPosPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let x = buf.read_double();
                let y = buf.read_double();
                let z = buf.read_double();
                let flags = buf.read_unsigned_byte();
                MovePlayerPosPacket::new_xyz(
                    x,
                    y,
                    z,
                    unpack_on_ground(flags),
                    unpack_horizontal_collision(flags),
                )
            },
        )
    }
}

impl Packet for MovePlayerPosPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_move_player_pos::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_double(self.x);
        buf.write_double(self.y);
        buf.write_double(self.z);
        buf.write_byte(pack_flags(self.on_ground, self.horizontal_collision));
    }

}

/// `ServerboundMovePlayerPacket.PosRot`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovePlayerPosRotPacket {
    /// `x`.
    pub x: f64,
    /// `y`.
    pub y: f64,
    /// `z`.
    pub z: f64,
    /// `yRot`.
    pub y_rot: f32,
    /// `xRot`.
    pub x_rot: f32,
    /// `onGround`.
    pub on_ground: bool,
    /// `horizontalCollision`.
    pub horizontal_collision: bool,
}

impl MovePlayerPosRotPacket {
    /// `new PosRot(Vec3, yRot, xRot, onGround, horizontalCollision)`.
    pub fn new(
        pos: Vec3,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
        horizontal_collision: bool,
    ) -> Self {
        Self::new_xyz(pos.x, pos.y, pos.z, y_rot, x_rot, on_ground, horizontal_collision)
    }

    /// `new PosRot(x, y, z, yRot, xRot, onGround, horizontalCollision)`.
    #[allow(clippy::too_many_arguments)]
    pub fn new_xyz(
        x: f64,
        y: f64,
        z: f64,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
        horizontal_collision: bool,
    ) -> Self {
        Self { x, y, z, y_rot, x_rot, on_ground, horizontal_collision }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, MovePlayerPosRotPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &MovePlayerPosRotPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let x = buf.read_double();
                let y = buf.read_double();
                let z = buf.read_double();
                let y_rot = buf.read_float();
                let x_rot = buf.read_float();
                let flags = buf.read_unsigned_byte();
                MovePlayerPosRotPacket::new_xyz(
                    x,
                    y,
                    z,
                    y_rot,
                    x_rot,
                    unpack_on_ground(flags),
                    unpack_horizontal_collision(flags),
                )
            },
        )
    }
}

impl Packet for MovePlayerPosRotPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_move_player_pos_rot::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_double(self.x);
        buf.write_double(self.y);
        buf.write_double(self.z);
        buf.write_float(self.y_rot);
        buf.write_float(self.x_rot);
        buf.write_byte(pack_flags(self.on_ground, self.horizontal_collision));
    }

}

/// `ServerboundMovePlayerPacket.Rot`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovePlayerRotPacket {
    /// `yRot`.
    pub y_rot: f32,
    /// `xRot`.
    pub x_rot: f32,
    /// `onGround`.
    pub on_ground: bool,
    /// `horizontalCollision`.
    pub horizontal_collision: bool,
}

impl MovePlayerRotPacket {
    /// `new Rot(yRot, xRot, onGround, horizontalCollision)`.
    pub fn new(y_rot: f32, x_rot: f32, on_ground: bool, horizontal_collision: bool) -> Self {
        Self { y_rot, x_rot, on_ground, horizontal_collision }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, MovePlayerRotPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &MovePlayerRotPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let y_rot = buf.read_float();
                let x_rot = buf.read_float();
                let flags = buf.read_unsigned_byte();
                MovePlayerRotPacket::new(
                    y_rot,
                    x_rot,
                    unpack_on_ground(flags),
                    unpack_horizontal_collision(flags),
                )
            },
        )
    }
}

impl Packet for MovePlayerRotPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_move_player_rot::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_float(self.y_rot);
        buf.write_float(self.x_rot);
        buf.write_byte(pack_flags(self.on_ground, self.horizontal_collision));
    }

}

/// `ServerboundMovePlayerPacket.StatusOnly`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MovePlayerStatusOnlyPacket {
    /// `onGround`.
    pub on_ground: bool,
    /// `horizontalCollision`.
    pub horizontal_collision: bool,
}

impl MovePlayerStatusOnlyPacket {
    /// `new StatusOnly(onGround, horizontalCollision)`.
    pub fn new(on_ground: bool, horizontal_collision: bool) -> Self {
        Self { on_ground, horizontal_collision }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, MovePlayerStatusOnlyPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &MovePlayerStatusOnlyPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let flags = buf.read_unsigned_byte();
                MovePlayerStatusOnlyPacket::new(
                    unpack_on_ground(flags),
                    unpack_horizontal_collision(flags),
                )
            },
        )
    }
}

impl Packet for MovePlayerStatusOnlyPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_move_player_status_only::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_byte(pack_flags(self.on_ground, self.horizontal_collision));
    }

}

/// Общие геттеры `ServerboundMovePlayerPacket` для всех 4 подтипов.
pub trait MovePlayerPacketCommon {
    /// `getX(defaultValue)`.
    fn get_x(&self, default_value: f64) -> f64;
    /// `getY(defaultValue)`.
    fn get_y(&self, default_value: f64) -> f64;
    /// `getZ(defaultValue)`.
    fn get_z(&self, default_value: f64) -> f64;
    /// `getYRot(defaultValue)`.
    fn get_y_rot(&self, default_value: f32) -> f32;
    /// `getXRot(defaultValue)`.
    fn get_x_rot(&self, default_value: f32) -> f32;
    /// `isOnGround`.
    fn is_on_ground(&self) -> bool;
    /// `horizontalCollision`.
    fn horizontal_collision(&self) -> bool;
    /// `hasPosition`.
    fn has_position(&self) -> bool;
    /// `hasRotation`.
    fn has_rotation(&self) -> bool;
}

impl MovePlayerPacketCommon for MovePlayerPosPacket {
    fn get_x(&self, _d: f64) -> f64 { self.x }
    fn get_y(&self, _d: f64) -> f64 { self.y }
    fn get_z(&self, _d: f64) -> f64 { self.z }
    fn get_y_rot(&self, d: f32) -> f32 { d }
    fn get_x_rot(&self, d: f32) -> f32 { d }
    fn is_on_ground(&self) -> bool { self.on_ground }
    fn horizontal_collision(&self) -> bool { self.horizontal_collision }
    fn has_position(&self) -> bool { true }
    fn has_rotation(&self) -> bool { false }
}

impl MovePlayerPacketCommon for MovePlayerPosRotPacket {
    fn get_x(&self, _d: f64) -> f64 { self.x }
    fn get_y(&self, _d: f64) -> f64 { self.y }
    fn get_z(&self, _d: f64) -> f64 { self.z }
    fn get_y_rot(&self, _d: f32) -> f32 { self.y_rot }
    fn get_x_rot(&self, _d: f32) -> f32 { self.x_rot }
    fn is_on_ground(&self) -> bool { self.on_ground }
    fn horizontal_collision(&self) -> bool { self.horizontal_collision }
    fn has_position(&self) -> bool { true }
    fn has_rotation(&self) -> bool { true }
}

impl MovePlayerPacketCommon for MovePlayerRotPacket {
    fn get_x(&self, d: f64) -> f64 { d }
    fn get_y(&self, d: f64) -> f64 { d }
    fn get_z(&self, d: f64) -> f64 { d }
    fn get_y_rot(&self, _d: f32) -> f32 { self.y_rot }
    fn get_x_rot(&self, _d: f32) -> f32 { self.x_rot }
    fn is_on_ground(&self) -> bool { self.on_ground }
    fn horizontal_collision(&self) -> bool { self.horizontal_collision }
    fn has_position(&self) -> bool { false }
    fn has_rotation(&self) -> bool { true }
}

impl MovePlayerPacketCommon for MovePlayerStatusOnlyPacket {
    fn get_x(&self, d: f64) -> f64 { d }
    fn get_y(&self, d: f64) -> f64 { d }
    fn get_z(&self, d: f64) -> f64 { d }
    fn get_y_rot(&self, d: f32) -> f32 { d }
    fn get_x_rot(&self, d: f32) -> f32 { d }
    fn is_on_ground(&self) -> bool { self.on_ground }
    fn horizontal_collision(&self) -> bool { self.horizontal_collision }
    fn has_position(&self) -> bool { false }
    fn has_rotation(&self) -> bool { false }
}
