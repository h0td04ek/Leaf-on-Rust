//! Перенос `net.minecraft.server.dialog.Dialog` — упрощённая версия.

use crate::network::codec::stream_codec::{StreamCodec, sync_fn};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::server::dialog::common_dialog_data::CommonDialogData;
use crate::server::dialog::dialog_action::DialogAction;

/// `Dialog` — dialog (упрощённая версия для network).
/// Полная версия требует Codec-сериализацию (SNBT) — здесь binary format.
#[derive(Clone, Debug)]
pub enum Dialog {
    /// `NoticeDialog`.
    Notice {
        /// `common`.
        common: CommonDialogData,
        /// `action` — ActionButton.
        action: Option<crate::server::dialog::action_button::ActionButton>,
    },
    /// `ConfirmationDialog`.
    Confirmation {
        /// `common`.
        common: CommonDialogData,
        /// `yesButton`.
        yes_button: crate::server::dialog::action_button::ActionButton,
        /// `noButton`.
        no_button: crate::server::dialog::action_button::ActionButton,
    },
    /// `MultiActionDialog`.
    MultiAction {
        /// `common`.
        common: CommonDialogData,
        /// `actions`.
        actions: Vec<crate::server::dialog::action_button::ActionButton>,
    },
    /// `ServerLinksDialog`.
    ServerLinks {
        /// `common`.
        common: CommonDialogData,
        /// `actions`.
        actions: Vec<crate::server::dialog::action_button::ActionButton>,
    },
    /// `DialogListDialog`.
    DialogList {
        /// `common`.
        common: CommonDialogData,
        /// `dialog`.
        dialog: Box<crate::core::holder::Holder<Dialog>>,
        /// `columns`.
        columns: i32,
        /// `entries`.
        entries: Vec<crate::server::dialog::dialog_list_entry::DialogListEntry>,
        /// `buttonWidth`.
        button_width: i32,
        /// `buttonSpacing`.
        button_spacing: i32,
    },
    /// `ButtonListDialog`.
    ButtonList {
        /// `common`.
        common: CommonDialogData,
        /// `columns`.
        columns: i32,
        /// `entries`.
        entries: Vec<crate::server::dialog::dialog_list_entry::DialogListEntry>,
        /// `buttonWidth`.
        button_width: i32,
        /// `buttonSpacing`.
        button_spacing: i32,
    },
}

impl Dialog {
    /// `type()` — registry id.
    pub fn type_id(&self) -> i32 {
        match self {
            Dialog::Notice { .. } => 0,
            Dialog::Confirmation { .. } => 1,
            Dialog::MultiAction { .. } => 2,
            Dialog::ServerLinks { .. } => 3,
            Dialog::DialogList { .. } => 4,
            Dialog::ButtonList { .. } => 5,
        }
    }

    /// `common()`.
    pub fn common(&self) -> &CommonDialogData {
        match self {
            Dialog::Notice { common, .. } => common,
            Dialog::Confirmation { common, .. } => common,
            Dialog::MultiAction { common, .. } => common,
            Dialog::ServerLinks { common, .. } => common,
            Dialog::DialogList { common, .. } => common,
            Dialog::ButtonList { common, .. } => common,
        }
    }

    /// `encode` — сериализация для network (упрощённая).
    pub fn encode(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.type_id());
        self.common().encode(buf);
        // Type-specific data — упрощённо
        match self {
            Dialog::Notice { action, .. } => {
                // ActionButton — упрощённо
                match action {
                    Some(a) => {
                        buf.write_bool(true);
                        a.encode(buf);
                    }
                    None => { buf.write_bool(false); }
                }
            }
            Dialog::Confirmation { yes_button, no_button, .. } => {
                yes_button.encode(buf);
                no_button.encode(buf);
            }
            Dialog::MultiAction { actions, .. } | Dialog::ServerLinks { actions, .. } => {
                buf.write_collection(actions, |b, a| a.encode(b));
            }
            Dialog::DialogList { dialog, columns, entries, button_width, button_spacing, .. } => {
                sync_fn(
                    |b: &mut FriendlyByteBuf, d: &crate::core::holder::Holder<Dialog>| {
                        match d {
                            crate::core::holder::Holder::Reference(id) => {
                                b.write_var_int(*id);
                            }
                            crate::core::holder::Holder::Key(key) => {
                                // Key — serialize as Reference with registry id
                                // For now, write 0 as placeholder
                                b.write_var_int(0);
                            }
                            crate::core::holder::Holder::Direct(dlg) => {
                                b.write_bool(true);
                                dlg.encode(b);
                            }
                        }
                    },
                    |b: &mut FriendlyByteBuf| {
                        if b.read_bool() {
                            crate::core::holder::Holder::Direct(Dialog::decode(b))
                        } else {
                            crate::core::holder::Holder::Reference(b.read_var_int())
                        }
                    },
                ).encode(buf, &*dialog);
                buf.write_var_int(*columns);
                buf.write_collection(entries, |b, e| e.encode(b));
                buf.write_var_int(*button_width);
                buf.write_var_int(*button_spacing);
            }
            Dialog::ButtonList { columns, entries, button_width, button_spacing, .. } => {
                buf.write_var_int(*columns);
                buf.write_collection(entries, |b, e| e.encode(b));
                buf.write_var_int(*button_width);
                buf.write_var_int(*button_spacing);
            }
        }
    }

    /// `decode`.
    pub fn decode(buf: &mut FriendlyByteBuf) -> Self {
        let type_id = buf.read_var_int();
        let common = CommonDialogData::decode(buf);
        match type_id {
            0 => {
                let action = if buf.read_bool() {
                    Some(crate::server::dialog::action_button::ActionButton::decode(buf))
                } else {
                    None
                };
                Dialog::Notice { common, action }
            }
            1 => {
                let yes_button = crate::server::dialog::action_button::ActionButton::decode(buf);
                let no_button = crate::server::dialog::action_button::ActionButton::decode(buf);
                Dialog::Confirmation { common, yes_button, no_button }
            }
            2 | 3 => {
                let actions = buf.read_collection(|n| Vec::with_capacity(n), |b| {
                    crate::server::dialog::action_button::ActionButton::decode(b)
                });
                if type_id == 2 {
                    Dialog::MultiAction { common, actions }
                } else {
                    Dialog::ServerLinks { common, actions }
                }
            }
            4 => {
                let dialog = Box::new(sync_fn(
                    |b: &mut FriendlyByteBuf, d: &crate::core::holder::Holder<Dialog>| {
                        match d {
                            crate::core::holder::Holder::Reference(id) => {
                                b.write_var_int(*id);
                            }
                            crate::core::holder::Holder::Key(key) => {
                                // Key — serialize as Reference with registry id
                                // For now, write 0 as placeholder
                                b.write_var_int(0);
                            }
                            crate::core::holder::Holder::Direct(dlg) => {
                                b.write_bool(true);
                                dlg.encode(b);
                            }
                        }
                    },
                    |b: &mut FriendlyByteBuf| {
                        if b.read_bool() {
                            crate::core::holder::Holder::Direct(Dialog::decode(b))
                        } else {
                            crate::core::holder::Holder::Reference(b.read_var_int())
                        }
                    },
                ).decode(buf));
                let columns = buf.read_var_int();
                let entries = buf.read_collection(|n| Vec::with_capacity(n), |b| {
                    crate::server::dialog::dialog_list_entry::DialogListEntry::decode(b)
                });
                let button_width = buf.read_var_int();
                let button_spacing = buf.read_var_int();
                Dialog::DialogList { common, dialog, columns, entries, button_width, button_spacing }
            }
            5 => {
                let columns = buf.read_var_int();
                let entries = buf.read_collection(|n| Vec::with_capacity(n), |b| {
                    crate::server::dialog::dialog_list_entry::DialogListEntry::decode(b)
                });
                let button_width = buf.read_var_int();
                let button_spacing = buf.read_var_int();
                Dialog::ButtonList { common, columns, entries, button_width, button_spacing }
            }
            _ => Dialog::Notice { common, action: None },
        }
    }
}
