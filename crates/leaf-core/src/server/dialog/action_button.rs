//! Перенос `net.minecraft.server.dialog.ActionButton` — упрощённая версия.

use crate::network::chat::component::Component;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::server::dialog::common_button_data::CommonButtonData;

/// `ActionButton` — кнопка dialog.
#[derive(Clone, Debug)]
pub struct ActionButton {
    /// `button` — CommonButtonData.
    pub button: CommonButtonData,
    /// `action` — Optional<Action>.
    pub action: Option<crate::server::dialog::action::Action>,
}

impl ActionButton {
    /// `new(button, action)`.
    pub fn new(button: CommonButtonData, action: Option<crate::server::dialog::action::Action>) -> Self {
        Self { button, action }
    }

    /// `encode` — упрощённая сериализация.
    pub fn encode(&self, buf: &mut FriendlyByteBuf) {
        self.button.encode(buf);
        match &self.action {
            Some(a) => {
                buf.write_bool(true);
                a.encode(buf);
            }
            None => { buf.write_bool(false); }
        }
    }

    /// `decode`.
    pub fn decode(buf: &mut FriendlyByteBuf) -> Self {
        let button = CommonButtonData::decode(buf);
        let action = if buf.read_bool() {
            Some(crate::server::dialog::action::Action::decode(buf))
        } else {
            None
        };
        Self::new(button, action)
    }
}
