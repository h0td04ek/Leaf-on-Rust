//! Перенос `net.minecraft.server.dialog.CommonButtonData`.

use crate::network::chat::component::Component;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `CommonButtonData` — данные кнопки.
#[derive(Clone, Debug)]
pub struct CommonButtonData {
    /// `label`.
    pub label: Component,
    /// `tooltip`.
    pub tooltip: Option<Component>,
    /// `width`.
    pub width: i32,
}

impl CommonButtonData {
    /// `new(label, tooltip, width)`.
    pub fn new(label: Component, tooltip: Option<Component>, width: i32) -> Self {
        Self { label, tooltip, width }
    }

    /// `encode`.
    pub fn encode(&self, buf: &mut FriendlyByteBuf) {
        crate::network::chat::component_json::component_stream_codec().encode(buf, &self.label);
        match &self.tooltip {
            Some(t) => {
                buf.write_bool(true);
                crate::network::chat::component_json::component_stream_codec().encode(buf, t);
            }
            None => { buf.write_bool(false); }
        }
        buf.write_var_int(self.width);
    }

    /// `decode`.
    pub fn decode(buf: &mut FriendlyByteBuf) -> Self {
        let codec = crate::network::chat::component_json::component_stream_codec();
        let label = crate::network::chat::component_json::component_stream_codec().decode(buf);
        let tooltip = if buf.read_bool() {
            Some(crate::network::chat::component_json::component_stream_codec().decode(buf))
        } else {
            None
        };
        let width = buf.read_var_int();
        Self::new(label, tooltip, width)
    }
}
