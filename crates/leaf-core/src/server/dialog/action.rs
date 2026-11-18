//! Перенос `net.minecraft.server.dialog.action.Action` — упрощённая версия.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// `Action` — действие dialog (упрощённая версия).
#[derive(Clone, Debug)]
pub enum Action {
    /// `StaticAction` — статическое действие.
    Static {
        /// `id` — идентификатор действия.
        id: Identifier,
    },
    /// `CommandTemplate` — шаблон команды.
    CommandTemplate {
        /// `command` — команда.
        command: String,
        /// `output` — Optional<Component>.
        output: Option<crate::network::chat::component::Component>,
    },
    /// `CustomAll` — custom action.
    CustomAll {
        /// `id` — идентификатор.
        id: Identifier,
    },
}

impl Action {
    /// `type()` — registry id.
    pub fn type_id(&self) -> i32 {
        match self {
            Action::Static { .. } => 0,
            Action::CommandTemplate { .. } => 1,
            Action::CustomAll { .. } => 2,
        }
    }

    /// `encode` — упрощённая сериализация.
    pub fn encode(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.type_id());
        match self {
            Action::Static { id } | Action::CustomAll { id } => {
                let _ = buf.write_identifier(id);
            }
            Action::CommandTemplate { command, output } => {
                let _ = buf.write_utf(command);
                match output {
                    Some(c) => {
                        buf.write_bool(true);
                        crate::network::chat::component_json::component_stream_codec().encode(buf, c);
                    }
                    None => { buf.write_bool(false); }
                }
            }
        }
    }

    /// `decode`.
    pub fn decode(buf: &mut FriendlyByteBuf) -> Self {
        let type_id = buf.read_var_int();
        match type_id {
            0 => Action::Static {
                id: buf.read_identifier().unwrap_or_else(|_| Identifier::from_namespace_and_path("minecraft", "unknown").unwrap()),
            },
            1 => {
                let command = buf.read_utf().unwrap_or_default();
                let output = if buf.read_bool() {
                    Some(crate::network::chat::component_json::component_stream_codec().decode(buf))
                } else {
                    None
                };
                Action::CommandTemplate { command, output }
            }
            2 => Action::CustomAll {
                id: buf.read_identifier().unwrap_or_else(|_| Identifier::from_namespace_and_path("minecraft", "unknown").unwrap()),
            },
            _ => Action::Static {
                id: Identifier::from_namespace_and_path("minecraft", "unknown").unwrap(),
            },
        }
    }
}
