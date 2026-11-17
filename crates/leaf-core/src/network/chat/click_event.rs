//! Перенос `net.minecraft.network.chat.ClickEvent` (модель данных;
//! DFU-кодеки добавятся с портом codec-слоя).

use crate::nbt::Tag;
use crate::resources::identifier::Identifier;

/// `ClickEvent` — действие по клику на компоненте.
#[derive(Clone, Debug, PartialEq)]
pub enum ClickEvent {
    /// `open_url`.
    OpenUrl(String),
    /// `open_file`.
    OpenFile(String),
    /// `run_command`.
    RunCommand(String),
    /// `suggest_command`.
    SuggestCommand(String),
    /// `show_dialog` — id диалога.
    ShowDialog(String),
    /// `change_page`.
    ChangePage(i32),
    /// `copy_to_clipboard`.
    CopyToClipboard(String),
    /// `custom`.
    Custom { id: Identifier, payload: Option<Tag> },
}

impl ClickEvent {
    /// `action()`.
    pub fn action(&self) -> ClickEventAction {
        match self {
            ClickEvent::OpenUrl(_) => ClickEventAction::OpenUrl,
            ClickEvent::OpenFile(_) => ClickEventAction::OpenFile,
            ClickEvent::RunCommand(_) => ClickEventAction::RunCommand,
            ClickEvent::SuggestCommand(_) => ClickEventAction::SuggestCommand,
            ClickEvent::ShowDialog(_) => ClickEventAction::ShowDialog,
            ClickEvent::ChangePage(_) => ClickEventAction::ChangePage,
            ClickEvent::CopyToClipboard(_) => ClickEventAction::CopyToClipboard,
            ClickEvent::Custom { .. } => ClickEventAction::Custom,
        }
    }
}

/// `ClickEvent.Action`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClickEventAction {
    OpenUrl,
    OpenFile,
    RunCommand,
    SuggestCommand,
    ShowDialog,
    ChangePage,
    CopyToClipboard,
    Custom,
}

impl ClickEventAction {
    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            ClickEventAction::OpenUrl => "open_url",
            ClickEventAction::OpenFile => "open_file",
            ClickEventAction::RunCommand => "run_command",
            ClickEventAction::SuggestCommand => "suggest_command",
            ClickEventAction::ShowDialog => "show_dialog",
            ClickEventAction::ChangePage => "change_page",
            ClickEventAction::CopyToClipboard => "copy_to_clipboard",
            ClickEventAction::Custom => "custom",
        }
    }

    /// `isAllowedFromServer`.
    pub fn is_allowed_from_server(&self) -> bool {
        !matches!(self, ClickEventAction::OpenFile)
    }
}
