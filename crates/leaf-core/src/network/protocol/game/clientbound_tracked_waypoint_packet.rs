//! Перенос `ClientboundTrackedWaypointPacket`.

use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::util::uuid::Uuid;
use crate::world::waypoints::tracked_waypoint::TrackedWaypoint;

/// `ClientboundTrackedWaypointPacket.Operation`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaypointOperation {
    /// `TRACK`.
    Track,
    /// `UNTRACK`.
    Untrack,
    /// `UPDATE`.
    Update,
}

/// Все значения (`values()`).
pub const WAYPOINT_OPERATIONS: [WaypointOperation; 3] = [
    WaypointOperation::Track,
    WaypointOperation::Untrack,
    WaypointOperation::Update,
];

impl WaypointOperation {
    /// `BY_ID` — `ByIdMap.continuous` + `OutOfBoundsStrategy.WRAP`.
    pub fn by_id(id: i32) -> WaypointOperation {
        WAYPOINT_OPERATIONS[id.rem_euclid(3) as usize]
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.idMapper(BY_ID, ordinal)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, WaypointOperation> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, v: &WaypointOperation| {
                buf.write_var_int(*v as i32);
            },
            |buf: &mut FriendlyByteBuf| WaypointOperation::by_id(buf.read_var_int()),
        )
    }
}

/// `ClientboundTrackedWaypointPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundTrackedWaypointPacket {
    /// `operation`.
    pub operation: WaypointOperation,
    /// `waypoint`.
    pub waypoint: TrackedWaypoint,
}

impl ClientboundTrackedWaypointPacket {
    /// `ClientboundTrackedWaypointPacket::new`.
    pub fn new(operation: WaypointOperation, waypoint: TrackedWaypoint) -> Self {
        Self { operation, waypoint }
    }

    /// `removeWaypoint`.
    pub fn remove_waypoint(uuid: Uuid) -> Self {
        Self::new(WaypointOperation::Untrack, TrackedWaypoint::empty(uuid))
    }

    /// `addWaypointPosition`.
    pub fn add_waypoint_position(uuid: Uuid, icon: crate::world::waypoints::waypoint::WaypointIcon, position: crate::core::vec3i::Vec3i) -> Self {
        Self::new(WaypointOperation::Track, TrackedWaypoint::set_position(uuid, icon, position))
    }

    /// `updateWaypointPosition`.
    pub fn update_waypoint_position(uuid: Uuid, icon: crate::world::waypoints::waypoint::WaypointIcon, position: crate::core::vec3i::Vec3i) -> Self {
        Self::new(WaypointOperation::Update, TrackedWaypoint::set_position(uuid, icon, position))
    }

    /// `addWaypointChunk`.
    pub fn add_waypoint_chunk(uuid: Uuid, icon: crate::world::waypoints::waypoint::WaypointIcon, chunk_pos: crate::world::level::chunk_pos::ChunkPos) -> Self {
        Self::new(WaypointOperation::Track, TrackedWaypoint::set_chunk(uuid, icon, chunk_pos))
    }

    /// `updateWaypointChunk`.
    pub fn update_waypoint_chunk(uuid: Uuid, icon: crate::world::waypoints::waypoint::WaypointIcon, chunk_pos: crate::world::level::chunk_pos::ChunkPos) -> Self {
        Self::new(WaypointOperation::Update, TrackedWaypoint::set_chunk(uuid, icon, chunk_pos))
    }

    /// `addWaypointAzimuth`.
    pub fn add_waypoint_azimuth(uuid: Uuid, icon: crate::world::waypoints::waypoint::WaypointIcon, angle: f32) -> Self {
        Self::new(WaypointOperation::Track, TrackedWaypoint::set_azimuth(uuid, icon, angle))
    }

    /// `updateWaypointAzimuth`.
    pub fn update_waypoint_azimuth(uuid: Uuid, icon: crate::world::waypoints::waypoint::WaypointIcon, angle: f32) -> Self {
        Self::new(WaypointOperation::Update, TrackedWaypoint::set_azimuth(uuid, icon, angle))
    }

    /// `operation`/`waypoint`.
    pub fn operation(&self) -> WaypointOperation {
        self.operation
    }
    /// `waypoint`.
    pub fn waypoint(&self) -> &TrackedWaypoint {
        &self.waypoint
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundTrackedWaypointPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundTrackedWaypointPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let operation = WaypointOperation::stream_codec().decode(buf);
                let waypoint = TrackedWaypoint::stream_codec().decode(buf);
                ClientboundTrackedWaypointPacket::new(operation, waypoint)
            },
        )
    }
}

impl Packet for ClientboundTrackedWaypointPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_waypoint::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        WaypointOperation::stream_codec().encode(buf, &self.operation);
        TrackedWaypoint::stream_codec().encode(buf, &self.waypoint);
    }
}
