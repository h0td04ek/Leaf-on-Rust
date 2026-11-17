//! Перенос `net.minecraft.network.PacketProcessor`.
//!
//! Очередь пакетов для главного потока. Listener разделяется через
//! `Arc<Mutex<dyn PacketListenerObj>>` — как общий объект listener'а
//! в Java.

use crate::network::protocol::packet::{Packet, PacketListenerObj};
use crate::network::protocol::packet_utils::{PacketProcessorLike, RUNNING_ON_DIFFERENT_THREAD};
use crate::reported_exception::ReportedException;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread::{self, ThreadId};

/// Общий тип listener'а для процессора.
pub type SharedListener = Arc<Mutex<dyn PacketListenerObj>>;

/// `PacketProcessor.ListenerAndPacket`.
struct ListenerAndPacket {
    listener: SharedListener,
    packet: Box<dyn Packet>,
}

impl ListenerAndPacket {
    /// `handle()` — shouldHandleMessage → packet.handle(listener),
    /// ошибки → listener.onPacketError.
    fn handle(self) -> Result<(), ReportedException> {
        let mut listener = self.listener.lock().unwrap();
        if listener.should_handle_message(self.packet.as_ref()) {
            // Java ловит Exception из packet.handle; у нас panic.
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.packet.handle(&mut *listener);
            }));
            if let Err(payload) = result {
                // Преобразуем панику в ошибку для onPacketError.
                let msg = payload
                    .downcast_ref::<&str>()
                    .map(|s| s.to_string())
                    .or_else(|| payload.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "panic in packet handler".to_string());
                let err = PacketPanicError(msg);
                match listener.on_packet_error(self.packet.as_ref(), &err) {
                    Ok(()) => return Ok(()),
                    Err(re) => return Err(re),
                }
            }
        }
        Ok(())
    }
}

/// Ошибка-представление паники для `onPacketError`.
#[derive(Debug)]
struct PacketPanicError(String);
impl std::fmt::Display for PacketPanicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for PacketPanicError {}

/// `PacketProcessor`.
pub struct PacketProcessor {
    packets_to_be_handled: Mutex<VecDeque<ListenerAndPacket>>,
    running_thread: ThreadId,
    closed: bool,
    /// Paper: счётчик обработанных пакетов.
    pub total_processed: u64,
}

impl PacketProcessor {
    /// `new PacketProcessor(runningThread)` — создаётся на главном потоке.
    pub fn new() -> Self {
        Self {
            packets_to_be_handled: Mutex::new(VecDeque::new()),
            running_thread: thread::current().id(),
            closed: false,
            total_processed: 0,
        }
    }

    /// Paper `executeSinglePacket` — обработать один пакет из очереди.
    /// Возвращает Ok(true) если пакет был, Ok(false) если очередь пуста.
    pub fn execute_single_packet(&mut self) -> Result<bool, ReportedException> {
        if self.closed {
            return Ok(false);
        }
        let task = self.packets_to_be_handled.lock().unwrap().pop_front();
        match task {
            None => Ok(false),
            Some(t) => {
                let r = t.handle();
                if r.is_ok() {
                    self.total_processed += 1;
                }
                r.map(|_| true)
            }
        }
    }

    /// `isSameThread`.
    pub fn is_same_thread(&self) -> bool {
        thread::current().id() == self.running_thread
    }

    /// `scheduleIfPossible(listener, packet)`.
    pub fn schedule_if_possible(
        &mut self,
        listener: SharedListener,
        packet: Box<dyn Packet>,
    ) {
        if self.closed {
            panic!("Server already shutting down");
        }
        let mut queue = self.packets_to_be_handled.lock().unwrap();
        queue.push_back(ListenerAndPacket { listener, packet });
    }

    /// `processQueuedPackets`.
    pub fn process_queued_packets(&mut self) -> Result<(), ReportedException> {
        if self.closed {
            return Ok(());
        }
        loop {
            let task = self.packets_to_be_handled.lock().unwrap().pop_front();
            match task {
                None => return Ok(()),
                Some(t) => {
                    t.handle()?;
                    self.total_processed += 1;
                }
            }
        }
    }

    /// `close`.
    pub fn close(&mut self) {
        self.closed = true;
    }

    /// Размер очереди (для диагностики).
    pub fn queued_packets(&self) -> usize {
        self.packets_to_be_handled.lock().unwrap().len()
    }
}

impl PacketProcessorLike for PacketProcessor {
    fn is_same_thread(&self) -> bool {
        PacketProcessor::is_same_thread(self)
    }

    fn schedule_if_possible(
        &mut self,
        listener: SharedListener,
        packet: Box<dyn Packet>,
    ) {
        PacketProcessor::schedule_if_possible(self, listener, packet);
    }
}

/// Утилита для `ensure_running_on_same_thread`: константа-ошибка.
pub fn running_on_different_thread() -> crate::network::protocol::packet_utils::RunningOnDifferentThreadException {
    RUNNING_ON_DIFFERENT_THREAD
}
