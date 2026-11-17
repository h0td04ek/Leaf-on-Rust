//! Перенос `ClientboundClearDialogPacket`.

use crate::network::codec::stream_codec::{unit, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static CLEAR_DIALOG_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("clear_dialog").unwrap())
});

/// `ClientboundClearDialogPacket` — `INSTANCE` синглтон.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundClearDialogPacket;

impl ClientboundClearDialogPacket {
    /// `INSTANCE`.
    pub const INSTANCE: ClientboundClearDialogPacket = ClientboundClearDialogPacket;

    /// `STREAM_CODEC` (`StreamCodec.unit(INSTANCE)`).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundClearDialogPacket> {
        unit(ClientboundClearDialogPacket)
    }
}

impl Packet for ClientboundClearDialogPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        CLEAR_DIALOG_TYPE.clone()
    }
    fn write_packet(&self, _buf: &mut FriendlyByteBuf) {}
}
