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
use crate::resources::resource_key::ResourceKey;

/// `DIRECT_HOLDER_ID`.
pub const DIRECT_HOLDER_ID: i32 = 0;

/// `Holder<T>` — `Direct` (inline-значение) или `Reference` (id в реестре).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Holder<T> {
    /// `Holder.direct(value)`.
    Direct(T),
    /// Ссылка на запись реестра (registry holder id).
    Reference(i32),
    /// `Holder.Reference` с известным `ResourceKey`, но без
    /// присвоенного registry-id (результат декода NBT/тегов).
    /// На stream-проводе кодируется как Reference после привязки id.
    Key(ResourceKey<T>),
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
            Holder::Reference(_) | Holder::Key(_) => HolderKind::Reference,
        }
    }

    /// `value` — для Direct.
    pub fn value(&self) -> Option<&T> {
        match self {
            Holder::Direct(v) => Some(v),
            Holder::Reference(_) | Holder::Key(_) => None,
        }
    }

    /// `unwrapKey` — `Some` для `Key` (и потенциально Reference
    /// после привязки).
    pub fn key(&self) -> Option<&ResourceKey<T>> {
        match self {
            Holder::Key(k) => Some(k),
            _ => None,
        }
    }

    /// `unwrap` для Direct; для Reference паникует — значение
    /// резолвится через реестр снаружи.
    pub fn unwrap_direct(&self) -> &T {
        match self {
            Holder::Direct(v) => v,
            Holder::Reference(id) => panic!("Holder.Reference({id}) has no direct value"),
            Holder::Key(_) => panic!("Holder.Key has no direct value"),
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
            Holder::Key(_) => {
                panic!("Holder.Key has no registry id bound yet")
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

/// `ByteBufCodecs.holderRegistry(registryKey)` — кодек holder'а,
/// который ОБЯЗАН быть registry-ссылкой: на проводе plain VarInt
/// registry-id (0-based, без direct-ветки и без +1).
/// Decode → `Holder::Reference(id)`.
pub fn holder_registry_stream_codec<T>() -> impl StreamCodec<FriendlyByteBuf, Holder<T>> + Send + Sync + 'static
where
    T: Send + Sync + 'static,
{
    crate::network::codec::stream_codec::sync_fn(
        |buf: &mut FriendlyByteBuf, holder: &Holder<T>| match holder {
            Holder::Reference(id) => {
                buf.write_var_int(*id);
            }
            Holder::Direct(_) => panic!("holderRegistry requires registry reference"),
            Holder::Key(_) => panic!("holderRegistry cannot encode unbound key"),
        },
        |buf: &mut FriendlyByteBuf| {
            let id = buf.read_var_int();
            Holder::Reference(id)
        },
    )
}
