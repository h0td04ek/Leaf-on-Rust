//! Перенос `net.minecraft.network.PacketSendListener`.
//!
//! Netty `ChannelFutureListener` заменён на `Box<dyn FnOnce(bool)>` —
//! тот же контракт, что в `Connection::send_with_listener`.

use crate::network::protocol::packet::Packet;

/// `thenRun(Runnable)` — выполнить action после завершения отправки.
/// (В Netty-версии при неудаче исключение шло в pipeline; здесь
/// send-listener получает только bool — действие выполняется всегда,
/// как в Java до fireExceptionCaught.)
pub fn then_run(action: impl FnOnce() + 'static) -> Box<dyn FnOnce(bool)> {
    Box::new(move |_success| {
        action();
    })
}

/// `thenRun` с действием, success.
pub fn then_run_with(
    action: impl FnOnce(bool) + 'static,
) -> Box<dyn FnOnce(bool)> {
    Box::new(action)
}

/// `exceptionallySend(packetSupplier)` — при неудаче отправить
/// fallback-пакет через переданный sender.
/// Возвращает listener; `sender` вызывается только при success=false.
pub fn exceptionally_send_with(
    packet_supplier: impl FnOnce() -> Option<Box<dyn Packet>> + 'static,
    sender: impl FnOnce(Box<dyn Packet>) + 'static,
) -> Box<dyn FnOnce(bool)> {
    Box::new(move |success| {
        if !success {
            if let Some(packet) = packet_supplier() {
                sender(packet);
            }
        }
    })
}

/// `exceptionallySend` без sender — только логирование через callback.
pub fn exceptionally_send_log(
    packet_supplier: impl FnOnce() -> Option<Box<dyn Packet>> + 'static,
    log: impl FnOnce(&dyn Packet) + 'static,
) -> Box<dyn FnOnce(bool)> {
    Box::new(move |success| {
        if !success {
            if let Some(packet) = packet_supplier() {
                log(packet.as_ref());
            }
        }
    })
}
