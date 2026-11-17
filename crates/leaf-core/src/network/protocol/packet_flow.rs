//! Перенос `net.minecraft.network.protocol.PacketFlow`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PacketFlow {
    SERVERBOUND,
    CLIENTBOUND,
}

impl PacketFlow {
    pub fn get_opposite(&self) -> PacketFlow {
        match self {
            PacketFlow::CLIENTBOUND => PacketFlow::SERVERBOUND,
            PacketFlow::SERVERBOUND => PacketFlow::CLIENTBOUND,
        }
    }

    /// `id()`
    pub fn id(&self) -> &'static str {
        match self {
            PacketFlow::SERVERBOUND => "serverbound",
            PacketFlow::CLIENTBOUND => "clientbound",
        }
    }
}

impl std::fmt::Display for PacketFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.id())
    }
}
