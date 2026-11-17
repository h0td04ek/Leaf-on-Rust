//! Перенос `net.minecraft.network.ConnectionProtocol`.

/// `ConnectionProtocol` — фаза handshake/play/login/... соединения.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConnectionProtocol {
    Handshaking,
    Play,
    Status,
    Login,
    Configuration,
}

impl ConnectionProtocol {
    /// `id()` — строковый id протокола.
    pub fn id(&self) -> &'static str {
        match self {
            ConnectionProtocol::Handshaking => "handshake",
            ConnectionProtocol::Play => "play",
            ConnectionProtocol::Status => "status",
            ConnectionProtocol::Login => "login",
            ConnectionProtocol::Configuration => "configuration",
        }
    }

    /// `isBoundToInbound` — статус/login принимают только serverbound.
    pub fn is_bound_to_inbound(&self) -> bool {
        matches!(self, ConnectionProtocol::Status | ConnectionProtocol::Login)
    }
}
