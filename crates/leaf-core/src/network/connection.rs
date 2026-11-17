//! Перенос `net.minecraft.network.Connection`.
//!
//! Netty `Channel` заменён трейтами:
//! - `Transport` — запись/flush/close/статус канала.
//! - `PacketWireCodec` — кодек протокола (id + тело пакета).
//!   Реализации появятся с портом ProtocolInfo/ProtocolCodecBuilder.
//!
//! Логика pendingActions/flushQueue/processQueue/tick портирована
//! по Paper-версии (optimize network).

use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketListenerObj};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Транспорт канала (замена Netty Channel).
pub trait Transport {
    /// Записать байты в исходящий буфер.
    fn write(&mut self, bytes: &[u8]);
    /// Принудительный flush.
    fn flush(&mut self);
    /// Закрыть соединение.
    fn close(&mut self);
    /// Активен ли канал.
    fn is_active(&self) -> bool;
    /// Удалённый адрес (для логов).
    fn remote_address(&self) -> Option<String>;
    /// Сериализовать сообщение об отключении (если нужно).
    fn supports_encryption(&self) -> bool {
        false
    }
}

/// Кодек протокола: Packet ↔ wire bytes.
/// В Java это ProtocolInfo + PacketEncoder/Decoder в Netty pipeline.
pub trait PacketWireCodec {
    /// Закодировать пакет целиком (с packet id).
    fn encode(&self, packet: &dyn Packet) -> Result<Vec<u8>, CodecError>;
    /// Декодировать пакет из байт (с packet id).
    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Packet>, CodecError>;
    /// Текущий flow принимаемых пакетов.
    fn receiving_flow(&self) -> PacketFlow;
}

#[derive(Debug)]
pub struct CodecError(pub String);
impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for CodecError {}

// ---------------------------------------------------------- pending actions

/// `WrappedConsumer` — отложенное действие в очереди соединения.
pub enum PendingAction {
    /// `PacketSendAction` — отправить пакет.
    SendPacket {
        packet: Box<dyn Packet>,
        /// `ChannelFutureListener` — в Rust: callback после отправки.
        send_listener: Option<Box<dyn FnOnce(bool)>>,
        flush: bool,
    },
    /// `runOnceConnected`.
    RunOnceConnected(Box<dyn FnOnce(&mut Connection) + Send>),
    /// `flushChannel`.
    Flush,
}

/// `DisconnectionDetails` — перенесён в `network::disconnection_details`.
pub use crate::network::disconnection_details::DisconnectionDetails;

/// `Connection`.
pub struct Connection {
    /// `receiving` — направление принимаемых пакетов.
    pub receiving: PacketFlow,
    /// Канал/транспорт.
    pub transport: Option<Box<dyn Transport>>,
    /// Текущий packet listener.
    packet_listener: Option<Box<dyn PacketListenerObj>>,
    /// Очередь отложенных действий (Paper optimize network).
    pending_actions: Mutex<VecDeque<PendingAction>>,
    /// `disconnectionDetails`
    pub disconnection_details: Option<DisconnectionDetails>,
    /// `delayedDisconnect`
    delayed_disconnect: Option<DisconnectionDetails>,
    /// `encrypted`
    pub encrypted: bool,
    /// `disconnectionHandled`
    pub disconnection_handled: bool,
    /// Счётчики пакетов.
    pub received_packets: u32,
    pub sent_packets: u32,
    /// `tickCount`
    pub tick_count: i32,
    /// `handlingFault`
    handling_fault: bool,
    /// Протокол кодек (codec текущего состояния).
    codec: Option<Box<dyn PacketWireCodec>>,
    /// `preparing` — handshake/config ещё не завершён.
    pub preparing: bool,
    /// `isPending` — Paper: используется в flushQueue.
    pub is_pending: bool,
    /// `stopReadingPackets` — Paper packet limiter.
    stop_reading_packets: bool,
    /// Paper: `queueImmunity`
    pub queue_immunity: bool,
}

impl Connection {
    pub fn new(receiving: PacketFlow) -> Self {
        Self {
            receiving,
            transport: None,
            packet_listener: None,
            pending_actions: Mutex::new(VecDeque::new()),
            disconnection_details: None,
            delayed_disconnect: None,
            encrypted: false,
            disconnection_handled: false,
            received_packets: 0,
            sent_packets: 0,
            tick_count: 0,
            handling_fault: false,
            codec: None,
            preparing: true,
            is_pending: true,
            stop_reading_packets: false,
            queue_immunity: false,
        }
    }

    /// Установить транспорт.
    pub fn set_transport(&mut self, transport: Box<dyn Transport>) {
        self.transport = Some(transport);
    }

    /// Установить кодек протокола.
    pub fn set_codec(&mut self, codec: Box<dyn PacketWireCodec>) {
        self.codec = Some(codec);
    }

    /// Установить packet listener.
    pub fn set_packet_listener(&mut self, listener: Box<dyn PacketListenerObj>) {
        self.packet_listener = Some(listener);
    }

    /// `isConnected` — канал активен и нет delayed disconnect.
    pub fn is_connected(&self) -> bool {
        self.transport.as_ref().map_or(false, |t| t.is_active())
            && self.delayed_disconnect.is_none()
    }

    /// `getRemoteAddress`.
    pub fn get_remote_address(&self) -> Option<String> {
        self.transport.as_ref().and_then(|t| t.remote_address())
    }

    /// `getLoggableAddress`.
    pub fn get_loggable_address(&self, log_ips: bool) -> String {
        if log_ips {
            self.get_remote_address().unwrap_or_else(|| "<local>".into())
        } else {
            "<redacted>".into()
        }
    }

    /// `isMemoryConnection` — in-memory (для тестов/локальной игры).
    pub fn is_memory_connection(&self) -> bool {
        // LocalChannel в Java; здесь — по типу транспорта (помечается через флаг)
        self.transport.as_ref().map_or(false, |t| t.supports_encryption() == false && t.remote_address().is_none())
    }

    // ---------------------------------------------------------- send

    /// `send(Packet)`.
    pub fn send(&mut self, packet: Box<dyn Packet>) {
        self.send_with_listener(packet, None, true);
    }

    /// `send(Packet, ChannelFutureListener)`.
    pub fn send_with_listener(
        &mut self,
        packet: Box<dyn Packet>,
        mut send_listener: Option<Box<dyn FnOnce(bool)>>,
        flush: bool,
    ) {
        let connected = self.is_connected();
        if !connected && !self.preparing {
            return;
        }

        packet.on_packet_dispatch(None);
        if connected && packet.is_ready() {
            self.send_packet(packet, send_listener, flush);
        } else {
            let extra = packet.get_extra_packets();
            match extra {
                None => {
                    self.pending_actions.lock().unwrap().push_back(PendingAction::SendPacket {
                        packet,
                        send_listener,
                        flush,
                    });
                }
                Some(extras) if extras.is_empty() => {
                    self.pending_actions.lock().unwrap().push_back(PendingAction::SendPacket {
                        packet,
                        send_listener,
                        flush,
                    });
                }
                Some(extras) => {
                    let mut actions = VecDeque::new();
                    actions.push_back(PendingAction::SendPacket {
                        packet,
                        send_listener: None,
                        flush: false,
                    });
                    let last = extras.len() - 1;
                    for (i, extra) in extras.into_iter().enumerate() {
                        let is_last = i == last;
                        actions.push_back(PendingAction::SendPacket {
                            packet: extra,
                            send_listener: if is_last { send_listener.take() } else { None },
                            flush: is_last,
                        });
                    }
                    self.pending_actions.lock().unwrap().extend(actions);
                }
            }
            self.flush_queue();
        }
    }

    /// `runOnceConnected`.
    pub fn run_once_connected(&mut self, action: impl FnOnce(&mut Connection) + Send + 'static) {
        if self.is_connected() {
            self.flush_queue();
            action(self);
        } else {
            self.pending_actions.lock().unwrap().push_back(PendingAction::RunOnceConnected(Box::new(action)));
        }
    }

    fn send_packet(
        &mut self,
        packet: Box<dyn Packet>,
        send_listener: Option<Box<dyn FnOnce(bool)>>,
        flush: bool,
    ) {
        self.sent_packets += 1;
        self.do_send_packet(packet, send_listener, flush);
    }

    fn do_send_packet(
        &mut self,
        packet: Box<dyn Packet>,
        send_listener: Option<Box<dyn FnOnce(bool)>>,
        flush: bool,
    ) {
        if !self.is_connected() {
            packet.on_packet_dispatch_finish(None, None);
            if let Some(l) = send_listener {
                l(false);
            }
            return;
        }
        let result = self.codec.as_ref().map(|c| c.encode(packet.as_ref()));
        match result {
            Some(Ok(bytes)) => {
                if let Some(t) = self.transport.as_mut() {
                    t.write(&bytes);
                    if flush {
                        t.flush();
                    }
                }
                if let Some(l) = send_listener {
                    l(true);
                }
                if packet.has_finish_listener() {
                    packet.on_packet_dispatch_finish(None, Some(&()));
                }
            }
            Some(Err(e)) => {
                // Java: ClientboundDisconnectPacket + disconnect —
                // конкретный пакет появится с протоколом login/play.
                self.disconnect(DisconnectionDetails::new(crate::network::chat::Component::literal(format!(
                    "Internal Exception: {}",
                    e
                ))));
                packet.on_packet_dispatch_finish(None, None);
            }
            None => {
                self.disconnect(DisconnectionDetails::new(crate::network::chat::Component::literal(
                    "No protocol codec configured",
                )));
            }
        }
    }

    // ---------------------------------------------------------- flush queue

    /// `flushChannel`.
    pub fn flush_channel(&mut self) {
        if self.is_connected() {
            self.flush();
        } else {
            self.pending_actions.lock().unwrap().push_back(PendingAction::Flush);
        }
    }

    fn flush(&mut self) {
        if let Some(t) = self.transport.as_mut() {
            t.flush();
        }
    }

    /// `flushQueue` — Paper-версия.
    fn flush_queue(&mut self) -> bool {
        if !self.is_connected() {
            return true;
        }
        // isMainThread в Java; здесь однопоточный раннер — всегда true.
        self.process_queue()
    }

    /// `processQueue` — Paper-версия.
    fn process_queue(&mut self) -> bool {
        loop {
            // Пакет в голове не готов — стоп (как в Java).
            let ready = {
                let queue = self.pending_actions.lock().unwrap();
                match queue.front() {
                    Some(PendingAction::SendPacket { packet, .. }) => packet.is_ready(),
                    Some(_) => true,
                    None => return true,
                }
            };
            if !ready {
                return false;
            }
            let action = self.pending_actions.lock().unwrap().pop_front();
            match action {
                Some(a) => self.execute_action(a),
                None => return true,
            }
        }
    }

    fn execute_action(&mut self, action: PendingAction) {
        match action {
            PendingAction::SendPacket { packet, send_listener, flush } => {
                self.send_packet(packet, send_listener, flush);
            }
            PendingAction::RunOnceConnected(f) => {
                f(self);
            }
            PendingAction::Flush => {
                self.flush();
            }
        }
    }

    // ---------------------------------------------------------- receive

    /// `channelRead0` — обработка входящего пакета.
    pub fn handle_incoming_packet(&mut self, packet: Box<dyn Packet>) {
        if !self.is_connected() && self.transport.is_none() {
            return;
        }
        if self.stop_reading_packets {
            return;
        }
        let listener = match self.packet_listener.as_mut() {
            Some(l) => l,
            None => panic!("Received a packet before the packet listener was initialized"),
        };
        if listener.should_handle_message(packet.as_ref()) {
            packet.handle(&mut **listener);
            self.received_packets += 1;
        }
    }

    /// Декодировать и обработать входящие байты.
    pub fn handle_incoming_bytes(&mut self, bytes: &[u8]) {
        if let Some(codec) = self.codec.as_ref() {
            match codec.decode(bytes) {
                Ok(packet) => self.handle_incoming_packet(packet),
                Err(e) => {
                    self.disconnect(DisconnectionDetails::new(crate::network::chat::Component::literal(format!(
                        "Invalid packet: {}",
                        e
                    ))));
                }
            }
        }
    }

    // ---------------------------------------------------------- tick/disconnect

    /// `tick` — Paper-версия с maxJoinsPerTick буфером.
    pub fn tick(&mut self) {
        self.tick_count += 1;
        self.flush_queue();
        if let Some(d) = self.delayed_disconnect.take() {
            self.disconnect(d);
        }
    }

    /// `disconnect(Component/DisconnectionDetails)`.
    pub fn disconnect(&mut self, details: DisconnectionDetails) {
        if self.disconnection_handled {
            return;
        }
        self.disconnection_handled = true;
        self.delayed_disconnect = Some(details.clone());
        if let Some(t) = self.transport.as_mut() {
            t.close();
        }
        self.disconnection_details = Some(details);
    }

    /// `handleDisconnection` — финализация.
    pub fn handle_disconnection(&mut self) {
        if self.disconnection_handled {
            return;
        }
        self.disconnection_handled = true;
        let details = DisconnectionDetails::new(crate::network::chat::Component::literal("Disconnected"));
        if let Some(listener) = self.packet_listener.as_mut() {
            listener.on_disconnect(&details);
        }
        self.disconnection_details = Some(details);
        self.packet_listener = None;
    }

    pub fn is_disconnected(&self) -> bool {
        self.disconnection_handled || self.delayed_disconnect.is_some()
    }

    /// `setupInboundProtocol` — после handshake.
    pub fn setup_inbound_protocol(&mut self, codec: Box<dyn PacketWireCodec>, listener: Box<dyn PacketListenerObj>) {
        self.codec = Some(codec);
        self.set_packet_listener(listener);
        self.preparing = false;
    }

    /// `setupOutboundProtocol`.
    pub fn setup_outbound_protocol(&mut self, codec: Box<dyn PacketWireCodec>) {
        self.codec = Some(codec);
        self.preparing = false;
    }
}


/// `ConnectionProtocol` — перенесён в `network::connection_protocol::ConnectionProtocol`.
pub use crate::network::connection_protocol::ConnectionProtocol;
