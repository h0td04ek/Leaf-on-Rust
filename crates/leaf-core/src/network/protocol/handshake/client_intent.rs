//! Перенос `net.minecraft.network.protocol.handshake.ClientIntent`.

/// `ClientIntent` — намерение клиента в handshake.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClientIntent {
    Status,
    Login,
    Transfer,
}

impl ClientIntent {
    /// `byId(int)`.
    pub fn by_id(id: i32) -> Result<ClientIntent, String> {
        match id {
            1 => Ok(ClientIntent::Status),
            2 => Ok(ClientIntent::Login),
            3 => Ok(ClientIntent::Transfer),
            _ => Err(format!("Unknown connection intent: {}", id)),
        }
    }

    /// `id()`.
    pub fn id(&self) -> i32 {
        match self {
            ClientIntent::Status => 1,
            ClientIntent::Login => 2,
            ClientIntent::Transfer => 3,
        }
    }
}
