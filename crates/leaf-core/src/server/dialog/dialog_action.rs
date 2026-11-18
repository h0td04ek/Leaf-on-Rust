//! Перенос `net.minecraft.server.dialog.DialogAction`.

/// `DialogAction` — действие после dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogAction {
    /// `CLOSE`.
    Close,
    /// `NONE`.
    None,
    /// `WAIT_FOR_RESPONSE`.
    WaitForResponse,
}

impl DialogAction {
    /// `values()`.
    pub const VALUES: [DialogAction; 3] = [
        DialogAction::Close,
        DialogAction::None,
        DialogAction::WaitForResponse,
    ];

    /// `from_id`.
    pub fn from_id(id: i32) -> Self {
        Self::VALUES.get(id as usize).copied().unwrap_or(DialogAction::Close)
    }

    /// `willUnpause`.
    pub fn will_unpause(&self) -> bool {
        matches!(self, DialogAction::Close | DialogAction::WaitForResponse)
    }
}
