//! Перенос `net.minecraft.ReportedException`.

use crate::crash_report::CrashReport;
use std::fmt;

/// `ReportedException` — исключение с CrashReport.
pub struct ReportedException {
    report: CrashReport,
}

impl ReportedException {
    pub fn new(report: CrashReport) -> Self {
        Self { report }
    }

    /// `getReport`.
    pub fn get_report(&self) -> &CrashReport {
        &self.report
    }

    pub fn into_report(self) -> CrashReport {
        self.report
    }
}

impl fmt::Debug for ReportedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ReportedException: {}", self.report.title)
    }
}

impl fmt::Display for ReportedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ReportedException: {}", self.report.title)
    }
}

impl std::error::Error for ReportedException {}
