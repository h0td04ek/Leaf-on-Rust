//! `ClientboundShowDialogPacket` — показ dialog клиенту.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::{StreamCodec, sync_fn};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::common::common_packet_types;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr, PacketListenerObj};
use crate::server::dialog::dialog::Dialog;

/// `ClientboundShowDialogPacket`.
pub struct ClientboundShowDialogPacket {
    /// `dialog` — Holder<Dialog>.
    pub dialog: Holder<Dialog>,
}

impl ClientboundShowDialogPacket {
    /// `new(dialog)`.
    pub fn new(dialog: Holder<Dialog>) -> Self {
        Self { dialog }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundShowDialogPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundShowDialogPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let dialog = sync_fn(
                    |b: &mut FriendlyByteBuf, d: &Holder<Dialog>| {
                        match d {
                            Holder::Reference(id) => {
                                b.write_var_int(*id);
                            }
                            Holder::Key(key) => {
                                b.write_var_int(0);
                            }
                            Holder::Key(key) => {
                                b.write_var_int(0);
                            }
                            Holder::Direct(dlg) => {
                                b.write_bool(true);
                                dlg.encode(b);
                            }
                        }
                    },
                    |b: &mut FriendlyByteBuf| {
                        if b.read_bool() {
                            Holder::Direct(Dialog::decode(b))
                        } else {
                            Holder::Reference(b.read_var_int())
                        }
                    },
                ).decode(buf);
                Self::new(dialog)
            },
        )
    }
}

impl Packet for ClientboundShowDialogPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn packet_type(&self) -> PacketTypePtr {
        crate::network::protocol::common::common_packet_types::clientbound_show_dialog::<Self>().erase()
    }

    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        sync_fn(
            |b: &mut FriendlyByteBuf, d: &Holder<Dialog>| {
                match d {
                    Holder::Reference(id) => {
                        b.write_var_int(*id);
                    }
                    Holder::Key(key) => {
                        b.write_var_int(0);
                    }
                    Holder::Direct(dlg) => {
                        b.write_bool(true);
                        dlg.encode(b);
                    }
                }
            },
            |b: &mut FriendlyByteBuf| {
                if b.read_bool() {
                    Holder::Direct(Dialog::decode(b))
                } else {
                    Holder::Reference(b.read_var_int())
                }
            },
        ).encode(buf, &self.dialog);
    }

    fn handle(&self, _handler: &mut dyn PacketListenerObj) {}
}
