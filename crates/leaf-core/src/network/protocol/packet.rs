//! Перенос `net.minecraft.network.protocol.Packet` и listener-трейтов.
//!
//! Отличия от Java:
//! - `Packet<T extends PacketListener>` → `Packet` без дженерика;
//!   `handle` принимает type-erased listener (`PacketListenerObj`),
//!   конкретные пакеты будут диспетчеризоваться через downcast
//!   в `PacketProcessor`/`Connection`.
//! - `PacketType` хранится как `Arc<dyn Any>`-ptr, т.к. тип пакета
//!   не всегда 'static в дженерик-смысле Java.
//! - `onPacketDispatch` получает `Option<&dyn DispatchPlayer>`
//!   (ServerPlayer реализует трейт позже).

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::connection::Connection;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet_type::PacketType;
use std::sync::Arc;

/// `PacketListener` — маркерный интерфейс слушателя пакетов.
pub trait PacketListener {}

/// `ClientboundPacketListener` — пакеты сервера→клиент.
pub trait ClientboundPacketListener: PacketListener {}

/// `ServerboundPacketListener` — пакеты клиента→сервер.
pub trait ServerboundPacketListener: PacketListener {}

/// `TickablePacketListener` — слушатель с тиком.
pub trait TickablePacketListener: PacketListener {
    fn tick(&mut self);
}

/// Type-erased listener для `Packet::handle`.
/// Методы соответствуют `PacketListener` в Java.
pub trait PacketListenerObj: PacketListener {
    /// `isAcceptingMessages` — default true.
    fn is_accepting_messages(&self) -> bool {
        true
    }

    /// `shouldHandleMessage` — default = isAcceptingMessages.
    fn should_handle_message(&self, _packet: &dyn Packet) -> bool {
        self.is_accepting_messages()
    }

    /// `onDisconnect(DisconnectionDetails)`.
    fn on_disconnect(&mut self, _details: &crate::network::disconnection_details::DisconnectionDetails) {}

    /// `onPacketError` — default оборачивает ошибку в ReportedException
    /// (`PacketUtils.makeReportedException`).
    /// Возвращает Err(ReportedException) для проброса, Ok(()) если обработано.
    fn on_packet_error(
        &mut self,
        packet: &dyn Packet,
        error: &(dyn std::error::Error + Send + Sync + 'static),
    ) -> Result<(), crate::reported_exception::ReportedException> {
        let mut report = crate::crash_report::CrashReport::for_throwable(
            error,
            "Main thread packet handler",
        );
        let ty = packet.packet_type().to_string();
        let is_terminal = packet.is_terminal();
        let is_skippable = packet.is_skippable();
        report.add_category("Incoming Packet", |cat| {
            cat.set_detail_str("Type", ty.clone());
            cat.set_detail_str("Is Terminal", is_terminal.to_string());
            cat.set_detail_str("Is Skippable", is_skippable.to_string());
        });
        self.fill_crash_report(&mut report);
        Err(crate::reported_exception::ReportedException::new(report))
    }

    /// `createDisconnectionInfo(reason, error)`.
    fn create_disconnection_info(
        &self,
        reason: crate::network::chat::Component,
        _error: Option<&(dyn std::error::Error + Send + Sync + 'static)>,
    ) -> crate::network::disconnection_details::DisconnectionDetails {
        crate::network::disconnection_details::DisconnectionDetails::new(reason)
    }

    /// `fillCrashReport` — через `CrashReportSink`.
    fn fill_crash_report(&self, report: &mut dyn crate::crash_report::CrashReportSink) {
        report.add_category("Connection", &mut |cat| {
            cat.set_detail_str("Protocol", self.protocol().id().to_string());
            cat.set_detail_str("Flow", self.flow().to_string());
            self.fill_listener_specific_crash_details(cat);
        });
    }

    /// `fillListenerSpecificCrashDetails` — default no-op.
    fn fill_listener_specific_crash_details(&self, _category: &mut dyn crate::crash_report::CrashReportCategorySink) {}

    /// `flow()` — направление listener'а.
    fn flow(&self) -> crate::network::protocol::packet_flow::PacketFlow;

    /// `protocol()` — вид протокола.
    fn protocol(&self) -> crate::network::connection_protocol::ConnectionProtocol;
}

/// Маркер для игрока в Paper-хуках (`onPacketDispatch`).
/// `ServerPlayer` реализует позже.
pub trait DispatchPlayer {}

/// Type-erased `PacketType` (аналог `PacketType<? extends Packet<?>>`).
/// `PacketType<T>` не требует границ на T — T только в phantom-типе.
pub type PacketTypePtr = PacketType<Box<dyn Packet>>;

/// `Packet` — контракт пакета.
/// `write_packet` обязателен: каждый пакет сериализует себя
/// (в Java это делает codec из ProtocolInfo — у нас метод на пакете,
/// т.к. пакеты в порте — конкретные структуры).
pub trait Packet {
    /// `Any`-доступ для downcast (codec-обёртки, bundle-пакеты).
    fn as_any(&self) -> &dyn std::any::Any;

    /// `BundlePacket.subPackets()` — Some для bundle-пакетов.
    /// Потребляет boxed-пакет и возвращает вложенные пакеты.
    fn into_bundle_sub_packets(self: Box<Self>) -> Option<Vec<Box<dyn Packet>>> {
        let _ = self;
        None
    }

    /// Тип пакета.
    fn packet_type(&self) -> PacketTypePtr;

    /// Сериализация тела пакета (без packet id — id пишет протокол).
    fn write_packet(&self, buf: &mut FriendlyByteBuf);

    /// Десериализация — статический метод, в трейте не выразить;
    /// реализуется в реестре кодеков протокола.

    /// Обработка на listener'е. Конкретные пакеты делают downcast
    /// `handler` к своему типу слушателя.
    fn handle(&self, _handler: &mut dyn PacketListenerObj) {}

    /// `isSkippable` — пакет можно пропустить при переполнении.
    fn is_skippable(&self) -> bool {
        false
    }

    /// `isTerminal` — последний пакет соединения.
    fn is_terminal(&self) -> bool {
        false
    }

    /// `hasLargePacketFallback` (Paper).
    fn has_large_packet_fallback(&self) -> bool {
        false
    }

    /// `packetTooLarge` (Paper).
    fn packet_too_large(&self, _manager: &Connection) -> bool {
        false
    }

    /// `onPacketDispatch` (Paper).
    fn on_packet_dispatch(&self, _player: Option<&dyn DispatchPlayer>) {}

    /// `onPacketDispatchFinish` (Paper).
    fn on_packet_dispatch_finish(&self, _player: Option<&dyn DispatchPlayer>, _future: Option<&()>) {}

    /// `hasFinishListener` (Paper).
    fn has_finish_listener(&self) -> bool {
        false
    }

    /// `isReady` (Paper).
    fn is_ready(&self) -> bool {
        true
    }

    /// `getExtraPackets` (Paper).
    fn get_extra_packets(&self) -> Option<Vec<Box<dyn Packet>>> {
        None
    }
}

/// `Packet.codec(encoder, decoder)` — конструктор member-кодека.
pub fn packet_codec<B, T, E, D>(encoder: E, decoder: D) -> crate::network::codec::stream_codec::MemberCodecStream<B, T, E, D>
where
    E: FnMut(&mut B, &T),
    D: FnMut(&mut B) -> T,
{
    crate::network::codec::stream_codec::MemberCodecStream::new(encoder, decoder)
}
