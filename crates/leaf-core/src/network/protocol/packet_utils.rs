//! Перенос `net.minecraft.network.protocol.PacketUtils`.
//!
//! `ensureRunningOnSameThread` работает через `PacketProcessorLike` —
//! реальный `PacketProcessor` в `network::packet_processor`.

use crate::network::protocol::packet::{Packet, PacketListenerObj};
use crate::reported_exception::ReportedException;

/// Минимальный контракт процессора пакетов для `PacketUtils`.
pub trait PacketProcessorLike {
    /// `isSameThread`.
    fn is_same_thread(&self) -> bool;
    /// `scheduleIfPossible(listener, packet)`.
    fn schedule_if_possible(
        &mut self,
        listener: std::sync::Arc<std::sync::Mutex<dyn PacketListenerObj>>,
        packet: Box<dyn Packet>,
    );
}

/// `RunningOnDifferentThreadException.RUNNING_ON_DIFFERENT_THREAD`.
#[derive(Debug)]
pub struct RunningOnDifferentThreadException;

impl std::fmt::Display for RunningOnDifferentThreadException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Running on different thread")
    }
}
impl std::error::Error for RunningOnDifferentThreadException {}

pub const RUNNING_ON_DIFFERENT_THREAD: RunningOnDifferentThreadException =
    RunningOnDifferentThreadException;

/// `ensureRunningOnSameThread(packet, listener, processor)` — boxed-вариант.
/// Если не тот поток — кладёт пакет в очередь и возвращает Err.
pub fn ensure_running_on_same_thread<T: PacketProcessorLike + ?Sized>(
    packet: Box<dyn Packet>,
    listener: &std::sync::Arc<std::sync::Mutex<dyn PacketListenerObj>>,
    processor: &mut T,
) -> Result<(), RunningOnDifferentThreadException> {
    if !processor.is_same_thread() {
        processor.schedule_if_possible(listener.clone(), packet);
        return Err(RUNNING_ON_DIFFERENT_THREAD);
    }
    Ok(())
}

/// `makeReportedException(exception, packet, listener)`.
pub fn make_reported_exception<L: PacketListenerObj + ?Sized>(
    exception: &(dyn std::error::Error + Send + Sync + 'static),
    packet: &dyn Packet,
    listener: &L,
) -> ReportedException {
    // Если уже ReportedException — дополняем его репорт.
    if let Some(re) = exception.downcast_ref::<ReportedException>() {
        let mut report = re.get_report().clone_resolved();
        fill_crash_report(&mut report, listener, Some(packet));
        return ReportedException::new(report);
    }
    let mut report = crate::crash_report::CrashReport::for_throwable(
        exception,
        "Main thread packet handler",
    );
    fill_crash_report(&mut report, listener, Some(packet));
    ReportedException::new(report)
}

/// `fillCrashReport(crashReport, listener, packet)`.
pub fn fill_crash_report<L: PacketListenerObj + ?Sized>(
    report: &mut dyn crate::crash_report::CrashReportSink,
    listener: &L,
    packet: Option<&dyn Packet>,
) {
    if let Some(p) = packet {
        let ty = p.packet_type().to_string();
        let is_terminal = p.is_terminal();
        let is_skippable = p.is_skippable();
        report.add_category("Incoming Packet", &mut |cat| {
            cat.set_detail_str("Type", ty.clone());
            cat.set_detail_str("Is Terminal", is_terminal.to_string());
            cat.set_detail_str("Is Skippable", is_skippable.to_string());
        });
    }
    listener.fill_crash_report(report);
}
