//! Перенос `net.minecraft.world.waypoints.TrackedWaypoint` —
//! метки, отслеживаемые клиентом (позиция/чанк/азимут/пустая).
//!
//! Методы, зависящие от `Level`/`Entity`/`PartialTickSupplier`
//! (`yawAngleToCamera`, `pitchDirectionToCamera`, `distanceSquared`
//! с entity-lookups), портируются вместе с `Level`/`Entity`;
//! здесь — полная wire-модель, `update` и позиционные помощники.

use crate::core::vec3i::Vec3i;
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::util::either::Either;
use crate::util::uuid::Uuid;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::phys::vec3::Vec3;
use crate::world::waypoints::waypoint::{waypoint_style_assets_default, WaypointIcon};

/// `TrackedWaypoint.Type` — порядок объявления важен (ordinal на проводе).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackedWaypointType {
    /// `EMPTY`.
    Empty = 0,
    /// `VEC3I`.
    Vec3i = 1,
    /// `CHUNK`.
    Chunk = 2,
    /// `AZIMUTH`.
    Azimuth = 3,
}

impl TrackedWaypointType {
    /// Все значения.
    pub const VALUES: [TrackedWaypointType; 4] = [
        TrackedWaypointType::Empty,
        TrackedWaypointType::Vec3i,
        TrackedWaypointType::Chunk,
        TrackedWaypointType::Azimuth,
    ];

    /// `readEnum`.
    pub fn by_ordinal(id: i32) -> TrackedWaypointType {
        Self::VALUES[id.rem_euclid(4) as usize]
    }

    /// `writeEnum`.
    pub fn ordinal(&self) -> i32 {
        *self as i32
    }
}

/// `TrackedWaypoint.PitchDirection`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PitchDirection {
    /// `NONE`.
    None,
    /// `UP`.
    Up,
    /// `DOWN`.
    Down,
}

/// `TrackedWaypoint.Camera` — интерфейс камеры.
pub trait TrackedWaypointCamera {
    /// `yaw()`.
    fn yaw(&self) -> f32;
    /// `position()`.
    fn position(&self) -> Vec3;
}

/// `TrackedWaypoint` — enum вариантов (Java — abstract class + наследники).
#[derive(Clone, Debug, PartialEq)]
pub enum TrackedWaypoint {
    /// `Vec3iWaypoint`.
    Vec3i {
        /// `identifier` — `Either<UUID, String>`.
        identifier: Either<Uuid, String>,
        /// `icon`.
        icon: WaypointIcon,
        /// `vector`.
        vector: Vec3i,
    },
    /// `ChunkWaypoint`.
    Chunk {
        /// `identifier`.
        identifier: Either<Uuid, String>,
        /// `icon`.
        icon: WaypointIcon,
        /// `chunkPos`.
        chunk_pos: ChunkPos,
    },
    /// `AzimuthWaypoint`.
    Azimuth {
        /// `identifier`.
        identifier: Either<Uuid, String>,
        /// `icon`.
        icon: WaypointIcon,
        /// `angle` — радианы.
        angle: f32,
    },
    /// `EmptyWaypoint`.
    Empty {
        /// `identifier`.
        identifier: Either<Uuid, String>,
        /// `icon`.
        icon: WaypointIcon,
    },
}

fn write_identifier(buf: &mut FriendlyByteBuf, id: &Either<Uuid, String>) {
    buf.write_either(
        id,
        |b, u| {
            b.write_uuid(u);
        },
        |b, s| {
            b.write_utf(s).expect("waypoint id");
        },
    );
}

fn read_identifier(buf: &mut FriendlyByteBuf) -> Either<Uuid, String> {
    buf.read_either(|b| b.read_uuid(), |b| b.read_utf().expect("waypoint id"))
}

impl TrackedWaypoint {
    /// `id()`.
    pub fn id(&self) -> &Either<Uuid, String> {
        match self {
            TrackedWaypoint::Vec3i { identifier, .. }
            | TrackedWaypoint::Chunk { identifier, .. }
            | TrackedWaypoint::Azimuth { identifier, .. }
            | TrackedWaypoint::Empty { identifier, .. } => identifier,
        }
    }

    /// `icon()`.
    pub fn icon(&self) -> &WaypointIcon {
        match self {
            TrackedWaypoint::Vec3i { icon, .. }
            | TrackedWaypoint::Chunk { icon, .. }
            | TrackedWaypoint::Azimuth { icon, .. }
            | TrackedWaypoint::Empty { icon, .. } => icon,
        }
    }

    /// `type()`.
    pub fn waypoint_type(&self) -> TrackedWaypointType {
        match self {
            TrackedWaypoint::Vec3i { .. } => TrackedWaypointType::Vec3i,
            TrackedWaypoint::Chunk { .. } => TrackedWaypointType::Chunk,
            TrackedWaypoint::Azimuth { .. } => TrackedWaypointType::Azimuth,
            TrackedWaypoint::Empty { .. } => TrackedWaypointType::Empty,
        }
    }

    /// `setPosition`.
    pub fn set_position(uuid: Uuid, icon: WaypointIcon, position: Vec3i) -> Self {
        TrackedWaypoint::Vec3i {
            identifier: Either::Left(uuid),
            icon,
            vector: position,
        }
    }

    /// `setChunk`.
    pub fn set_chunk(uuid: Uuid, icon: WaypointIcon, chunk_pos: ChunkPos) -> Self {
        TrackedWaypoint::Chunk {
            identifier: Either::Left(uuid),
            icon,
            chunk_pos,
        }
    }

    /// `setAzimuth`.
    pub fn set_azimuth(uuid: Uuid, icon: WaypointIcon, angle: f32) -> Self {
        TrackedWaypoint::Azimuth {
            identifier: Either::Left(uuid),
            icon,
            angle,
        }
    }

    /// `empty`.
    pub fn empty(uuid: Uuid) -> Self {
        TrackedWaypoint::Empty {
            identifier: Either::Left(uuid),
            icon: WaypointIcon::default(),
        }
    }

    /// `update` — скопировать содержимое из waypoint того же типа.
    pub fn update(&mut self, waypoint: &TrackedWaypoint) {
        match (self, waypoint) {
            (
                TrackedWaypoint::Vec3i { vector, .. },
                TrackedWaypoint::Vec3i {
                    vector: other, ..
                },
            ) => *vector = *other,
            (
                TrackedWaypoint::Chunk { chunk_pos, .. },
                TrackedWaypoint::Chunk {
                    chunk_pos: other, ..
                },
            ) => *chunk_pos = *other,
            (
                TrackedWaypoint::Azimuth { angle, .. },
                TrackedWaypoint::Azimuth { angle: other, .. },
            ) => *angle = *other,
            (TrackedWaypoint::Empty { .. }, TrackedWaypoint::Empty { .. }) => {}
            _ => {}
        }
    }

    /// `write` — Either + Icon + Type + contents.
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        write_identifier(buf, self.id());
        WaypointIcon::stream_codec().encode(buf, self.icon());
        buf.write_var_int(self.waypoint_type().ordinal());
        self.write_contents(buf);
    }

    fn write_contents(&self, buf: &mut FriendlyByteBuf) {
        match self {
            TrackedWaypoint::Vec3i { vector, .. } => {
                buf.write_var_int(vector.x);
                buf.write_var_int(vector.y);
                buf.write_var_int(vector.z);
            }
            TrackedWaypoint::Chunk { chunk_pos, .. } => {
                buf.write_var_int(chunk_pos.x);
                buf.write_var_int(chunk_pos.z);
            }
            TrackedWaypoint::Azimuth { angle, .. } => {
                buf.write_float(*angle);
            }
            TrackedWaypoint::Empty { .. } => {}
        }
    }

    /// `read`.
    pub fn read(buf: &mut FriendlyByteBuf) -> TrackedWaypoint {
        let identifier = read_identifier(buf);
        let icon = WaypointIcon::stream_codec().decode(buf);
        let ty = TrackedWaypointType::by_ordinal(buf.read_var_int());
        match ty {
            TrackedWaypointType::Empty => TrackedWaypoint::Empty { identifier, icon },
            TrackedWaypointType::Vec3i => TrackedWaypoint::Vec3i {
                identifier,
                icon,
                vector: Vec3i {
                    x: buf.read_var_int(),
                    y: buf.read_var_int(),
                    z: buf.read_var_int(),
                },
            },
            TrackedWaypointType::Chunk => TrackedWaypoint::Chunk {
                identifier,
                icon,
                chunk_pos: ChunkPos {
                    x: buf.read_var_int(),
                    z: buf.read_var_int(),
                },
            },
            TrackedWaypointType::Azimuth => TrackedWaypoint::Azimuth {
                identifier,
                icon,
                angle: buf.read_float(),
            },
        }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, TrackedWaypoint> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, w: &TrackedWaypoint| w.write(buf),
            |buf: &mut FriendlyByteBuf| TrackedWaypoint::read(buf),
        )
    }

    // ---- позиционные помощники (без Level/Entity-зависимостей) ----

    /// `yawAngleToCamera` для AZIMUTH (не требует Level).
    pub fn azimuth_yaw_angle_to_camera(&self, camera: &dyn TrackedWaypointCamera) -> f64 {
        if let TrackedWaypoint::Azimuth { angle, .. } = self {
            let diff = crate::util::mth::degrees_difference(
                camera.yaw(),
                *angle * (180.0 / std::f32::consts::PI),
            );
            diff as f64
        } else {
            f64::NAN
        }
    }

    /// `yawAngleToCamera` для CHUNK (позиция = центр чанка на высоте y).
    pub fn chunk_yaw_angle_to_camera(&self, camera: &dyn TrackedWaypointCamera, y: f64) -> f64 {
        if let TrackedWaypoint::Chunk { chunk_pos, .. } = self {
            let mid_x = chunk_pos.get_middle_block_x() as f64 + 0.5;
            let mid_z = chunk_pos.get_middle_block_z() as f64 + 0.5;
            // position(y) = Vec3.atCenterOf(middleBlockPosition(y)) = (mid, y+0.5?, mid)
            // Java: Vec3.atCenterOf(BlockPos) = pos + 0.5 по всем осям.
            let target = Vec3::new(mid_x, y + 0.5, mid_z);
            let delta = camera.position().subtract_vec(&target);
            // rotateClockwise90: (-z, y, x)
            let rotated = Vec3::new(-delta.z, delta.y, delta.x);
            let f = crate::util::mth::atan2(rotated.z, rotated.x) * (180.0 / std::f64::consts::PI);
            crate::util::mth::degrees_difference(camera.yaw(), f as f32) as f64
        } else {
            f64::NAN
        }
    }

    // `pitchDirectionToCamera` для AZIMUTH/CHUNK — требует `Projector`;
    // портируется с клиентской проекцией.
}
