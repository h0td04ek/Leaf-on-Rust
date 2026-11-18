//! Перенос `net.minecraft.server.dialog`.

pub mod action;
pub mod action_button;
pub mod common_button_data;
pub mod common_dialog_data;
pub mod dialog;
pub mod dialog_action;
pub mod dialog_list_entry;

pub use action::Action;
pub use action_button::ActionButton;
pub use common_button_data::CommonButtonData;
pub use common_dialog_data::CommonDialogData;
pub use dialog::Dialog;
pub use dialog_action::DialogAction;
pub use dialog_list_entry::DialogListEntry;
