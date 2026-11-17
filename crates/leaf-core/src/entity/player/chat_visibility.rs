//! Перенос `net.minecraft.world.entity.player.ChatVisiblity`.

use crate::network::chat::component::Component;

/// `ChatVisiblity` — видимость чата.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

impl ChatVisibility {
    /// Порядок в `values()`.
    pub const VALUES: [ChatVisibility; 3] = [
        ChatVisibility::Full,
        ChatVisibility::System,
        ChatVisibility::Hidden,
    ];

    /// `id` (0,1,2).
    pub fn id(&self) -> i32 {
        *self as i32
    }

    /// `BY_ID` (WRAP).
    pub fn by_id_wrapped(id: i32) -> ChatVisibility {
        let n = id.rem_euclid(3);
        ChatVisibility::VALUES[n as usize]
    }

    /// `caption()`.
    pub fn caption(&self) -> Component {
        let key = match self {
            ChatVisibility::Full => "options.chat.visibility.full",
            ChatVisibility::System => "options.chat.visibility.system",
            ChatVisibility::Hidden => "options.chat.visibility.hidden",
        };
        Component::translatable(key, Vec::new())
    }
}
