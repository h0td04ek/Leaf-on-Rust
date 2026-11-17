//! Перенос `net.minecraft.network.protocol.CodecModifier`.

use crate::network::protocol::protocol_info::PacketCodec;
use std::sync::Arc;

/// `CodecModifier<B, V, C>` — модификатор пакетного кодека
/// (например, обёртка шифрованием/сжатием на уровне тела пакета).
pub trait CodecModifier<C> {
    /// `apply(codec, context)` → модифицированный кодек.
    fn apply(&self, codec: PacketCodec, context: &C) -> PacketCodec;
}

impl<C, F> CodecModifier<C> for F
where
    F: Fn(PacketCodec, &C) -> PacketCodec,
{
    fn apply(&self, codec: PacketCodec, context: &C) -> PacketCodec {
        (self)(codec, context)
    }
}

/// Type-erased модификатор.
pub type CodecModifierPtr<C> = Arc<dyn CodecModifier<C>>;
