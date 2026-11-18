//! Перенос `net.minecraft.server.dialog.DialogListEntry` — упрощённая версия.

use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::server::dialog::action_button::ActionButton;

/// `DialogListEntry` — запись списка dialog.
#[derive(Clone, Debug)]
pub struct DialogListEntry {
    /// `id` — идентификатор.
    pub id: String,
    /// `button` — ActionButton.
    pub button: ActionButton,
}

impl DialogListEntry {
    /// `new(id, button)`.
    pub fn new(id: String, button: ActionButton) -> Self {
        Self { id, button }
    }

    /// `encode`.
    pub fn encode(&self, buf: &mut FriendlyByteBuf) {
        let _ = buf.write_utf(&self.id);
        self.button.encode(buf);
    }

    /// `decode`.
    pub fn decode(buf: &mut FriendlyByteBuf) -> Self {
        let id = buf.read_utf().unwrap_or_default();
        let button = ActionButton::decode(buf);
        Self::new(id, button)
    }
}
