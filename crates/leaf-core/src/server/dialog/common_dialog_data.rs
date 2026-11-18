//! Перенос `net.minecraft.server.dialog.CommonDialogData`.

use crate::network::chat::component::Component;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::server::dialog::dialog_action::DialogAction;

/// `CommonDialogData` — общие данные dialog.
#[derive(Clone, Debug)]
pub struct CommonDialogData {
    /// `title`.
    pub title: Component,
    /// `externalTitle`.
    pub external_title: Option<Component>,
    /// `canCloseWithEscape`.
    pub can_close_with_escape: bool,
    /// `pause`.
    pub pause: bool,
    /// `afterAction`.
    pub after_action: DialogAction,
}

impl CommonDialogData {
    /// `new(...)`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        title: Component,
        external_title: Option<Component>,
        can_close_with_escape: bool,
        pause: bool,
        after_action: DialogAction,
    ) -> Self {
        Self { title, external_title, can_close_with_escape, pause, after_action }
    }

    /// `computeExternalTitle`.
    pub fn compute_external_title(&self) -> Component {
        self.external_title.clone().unwrap_or_else(|| self.title.clone())
    }

    /// `encode` — сериализация для network.
    pub fn encode(&self, buf: &mut FriendlyByteBuf) {
        crate::network::chat::component_json::component_stream_codec().encode(buf, &self.title);
        // externalTitle — optional
        match &self.external_title {
            Some(t) => {
                buf.write_bool(true);
                crate::network::chat::component_json::component_stream_codec().encode(buf, t);
            }
            None => { buf.write_bool(false); }
        }
        buf.write_bool(self.can_close_with_escape);
        buf.write_bool(self.pause);
        buf.write_var_int(self.after_action as i32);
    }

    /// `decode`.
    pub fn decode(buf: &mut FriendlyByteBuf) -> Self {
        let codec = crate::network::chat::component_json::component_stream_codec();
        let title = crate::network::chat::component_json::component_stream_codec().decode(buf);
        let external_title = if buf.read_bool() {
            Some(crate::network::chat::component_json::component_stream_codec().decode(buf))
        } else {
            None
        };
        let can_close_with_escape = buf.read_bool();
        let pause = buf.read_bool();
        let after_action = DialogAction::from_id(buf.read_var_int());
        Self::new(title, external_title, can_close_with_escape, pause, after_action)
    }
}
