//! Перенос `net.minecraft.network.protocol.PacketType`.

use crate::network::protocol::packet_flow::PacketFlow;
use crate::resources::identifier::Identifier;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// `PacketType<T extends Packet<?>>` — (flow, id).
/// Реализации Clone/Debug/Eq/Hash не требуют границ на T —
/// тип пакета участвует только в phantom-типе (как в Java type erasure).
pub struct PacketType<T> {
    pub flow: PacketFlow,
    pub id: Identifier,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> PacketType<T> {
    pub fn new(flow: PacketFlow, id: Identifier) -> Self {
        Self { flow, id, _phantom: PhantomData }
    }
}

impl<T> Clone for PacketType<T> {
    fn clone(&self) -> Self {
        PacketType { flow: self.flow, id: self.id.clone(), _phantom: PhantomData }
    }
}

impl<T> fmt::Debug for PacketType<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PacketType[{}:{}]", self.flow.id(), self.id)
    }
}

impl<T> PartialEq for PacketType<T> {
    fn eq(&self, other: &Self) -> bool {
        self.flow == other.flow && self.id == other.id
    }
}
impl<T> Eq for PacketType<T> {}

impl<T> Hash for PacketType<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.flow.hash(state);
        self.id.hash(state);
    }
}

impl<T> fmt::Display for PacketType<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.flow.id(), self.id)
    }
}

impl<T> PacketType<T> {
    /// Стирание типа пакета → `PacketTypePtr` (аналог unchecked cast
    /// `PacketType<T>` → `PacketType<? extends Packet<?>>`).
    pub fn erase(self) -> crate::network::protocol::packet::PacketTypePtr {
        PacketType::new(self.flow, self.id)
    }
}
