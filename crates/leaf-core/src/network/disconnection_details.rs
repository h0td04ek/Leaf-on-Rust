//! Перенос `net.minecraft.network.DisconnectionDetails`.
//!
//! Paper-поле `disconnectionReason` (io.papermc.paper.connection.
//! DisconnectionReason) будет добавлено с портом plugin-API.

use crate::network::chat::Component;
use std::path::PathBuf;

/// `DisconnectionDetails` — причина отключения.
#[derive(Clone, Debug)]
pub struct DisconnectionDetails {
    /// `reason` — компонент причины.
    pub reason: Component,
    /// `report` — путь к crash-report (если есть).
    pub report: Option<PathBuf>,
    /// `bugReportLink` — ссылка на баг-репорт.
    pub bug_report_link: Option<String>,
    /// `quitMessage` — сообщение при выходе.
    pub quit_message: Option<Component>,
}

impl DisconnectionDetails {
    /// `new DisconnectionDetails(reason)`.
    pub fn new(reason: Component) -> Self {
        Self { reason, report: None, bug_report_link: None, quit_message: None }
    }

    /// `new DisconnectionDetails(reason, report, bugReportLink)`.
    pub fn with_report_link(
        reason: Component,
        report: Option<PathBuf>,
        bug_report_link: Option<String>,
    ) -> Self {
        Self { reason, report, bug_report_link, quit_message: None }
    }

    /// `withQuitMessage` (Paper).
    pub fn with_quit_message(mut self, quit_message: Component) -> Self {
        self.quit_message = Some(quit_message);
        self
    }
}
