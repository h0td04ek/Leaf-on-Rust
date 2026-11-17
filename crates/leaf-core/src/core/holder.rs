//! Перенос `net.minecraft.core.Holder` — ссылка на значение реестра.
//!
//! Wire-формат `ByteBufCodecs.holder*`:
//! - VarInt 0 → direct-значение (кодек значения)
//! - VarInt n>0 → registry reference с id = n-1
//!
//! `Reference` хранит registry id; разрешение в значение — через
//! реестр при использовании (порт `Registry` — отдельный этап).

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `DIRECT_HOLDER_ID`.
pub const DIRECT_HOLDER_ID: i32 = 0;

/// `Holder<T>` — `Direct` (inline-значение) или `Reference` (id в реестре).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Holder<T> {
    /// `Holder.direct(value)`.
    Direct(T),
    /// Ссылка на запись реестра (registry holder id).
    Reference(i32),
}

/// `Holder.Kind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HolderKind {
    /// `REFERENCE`.
    Reference,
    /// `DIRECT`.
    Direct,
}

impl<T> Holder<T> {
    /// `Holder.direct`.
    pub fn direct(value: T) -> Self {
        Self::Direct(value)
    }

    /// `Holder.reference`-эквивалент по id.
    pub fn reference(id: i32) -> Self {
        Self::Reference(id)
    }

    /// `kind`.
    pub fn kind(&self) -> HolderKind {
        match self {
            Holder::Direct(_) => HolderKind::Direct,
            Holder::Reference(_) => HolderKind::Reference,
        }
    }

    /// `value` — для Direct.
    pub fn value(&self) -> Option<&T> {
        match self {
            Holder::Direct(v) => Some(v),
            Holder::Reference(_) => None,
        }
    }

    /// `unwrap` для Direct; для Reference паникует — значение
    /// резолвится через реестр снаружи.
    pub fn unwrap_direct(&self) -> &T {
        match self {
            Holder::Direct(v) => v,
            Holder::Reference(id) => panic!("Holder.Reference({id}) has no direct value"),
        }
    }
}

/// `ByteBufCodecs.holder(registryKey, codec)` — StreamCodec для `Holder<T>`.
pub struct HolderStreamCodec<T> {
    value_codec: Box<dyn StreamCodec<FriendlyByteBuf, T> + Send + Sync>,
}

impl<T> HolderStreamCodec<T> {
    /// Создать кодек holder'а с кодеком direct-значения.
    pub fn new(
        value_codec: impl StreamCodec<FriendlyByteBuf, T> + Send + Sync + 'static,
    ) -> Self {
        Self { value_codec: Box::new(value_codec) }
    }
}

impl<T> StreamCodec<FriendlyByteBuf, Holder<T>> for HolderStreamCodec<T> {
    fn encode(&self, buf: &mut FriendlyByteBuf, value: &Holder<T>) {
        match value {
            Holder::Direct(v) => {
                buf.write_var_int(DIRECT_HOLDER_ID);
                self.value_codec.encode(buf, v);
            }
            Holder::Reference(id) => {
                buf.write_var_int(id + 1);
            }
        }
    }

    fn decode(&self, buf: &mut FriendlyByteBuf) -> Holder<T> {
        let i = buf.read_var_int();
        if i == DIRECT_HOLDER_ID {
            Holder::Direct(self.value_codec.decode(buf))
        } else {
            Holder::Reference(i - 1)
        }
    }
}
