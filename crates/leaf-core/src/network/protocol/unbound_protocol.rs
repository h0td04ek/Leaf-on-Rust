//! Перенос `net.minecraft.network.protocol.UnboundProtocol` и
//! `SimpleUnboundProtocol`.

use crate::network::protocol::protocol_info::{ProtocolInfo, ProtocolInfoDetailsProvider};

/// `UnboundProtocol<T, B, C>` — протокол с контекстом сборки.
/// `B` зафиксирован как `FriendlyByteBuf` (см. IdDispatchCodec).
pub trait UnboundProtocol<C>: ProtocolInfoDetailsProvider {
    /// `bind(bufferFactory, context)` → готовый ProtocolInfo.
    fn bind(&self, context: &C) -> Box<dyn ProtocolInfo>;
}

/// `SimpleUnboundProtocol<T, B>` — протокол без контекста.
pub trait SimpleUnboundProtocol: ProtocolInfoDetailsProvider {
    /// `bind(bufferFactory)` → готовый ProtocolInfo.
    fn bind(&self) -> Box<dyn ProtocolInfo>;
}
