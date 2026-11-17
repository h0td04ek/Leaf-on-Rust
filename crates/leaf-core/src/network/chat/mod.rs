//! Перенос `net.minecraft.network.chat` — модель текстовых компонентов.

pub mod chat_formatting;
pub mod chat_type;
pub mod chat_type_decoration;
pub mod click_event;
pub mod component;
pub mod component_contents;
pub mod common_components;
pub mod component_json;
pub mod component_utils;
pub mod font_description;
pub mod formatted_text;
pub mod hover_event;
pub mod message_signature;
pub mod style;
pub mod text_color;

pub use chat_formatting::ChatFormatting;
pub use click_event::ClickEvent;
pub use component::Component;
pub use component_contents::{
    BlockDataSource, ComponentContents, DataSource, EntityDataSource, ObjectInfo,
    SelectorPattern, StorageDataSource, TranslatableArg, TranslatableContents,
};
pub use font_description::FontDescription;
pub use formatted_text::FormattedText;
pub use hover_event::HoverEvent;
pub use style::Style;
pub use text_color::TextColor;
pub mod filter_mask;
pub mod last_seen_messages;
pub mod local_chat_session;
pub mod message_signature_cache;
pub mod remote_chat_session;
pub mod signed_message_body;
pub mod signed_message_link;
