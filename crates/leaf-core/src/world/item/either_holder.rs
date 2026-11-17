//! Перенос `net.minecraft.world.item.EitherHolder<T>` —
//! `Either<Holder<T>, ResourceKey<T>>` (registry-holder ИЛИ ключ).
//!
//! Wire: `ByteBufCodecs.either(holderCodec, resourceKeyCodec)` —
//! BOOL + payload; `ResourceKey.streamCodec` пишет только
//! `Identifier` (registry известен из контекста).

use crate::core::holder::Holder;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use crate::util::either::Either;

/// `EitherHolder<T>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EitherHolder<T> {
    /// `contents`.
    pub contents: Either<Holder<T>, ResourceKey<T>>,
}

impl<T> EitherHolder<T> {
    /// `new EitherHolder(holder)`.
    pub fn of_holder(holder: Holder<T>) -> Self {
        Self { contents: Either::Left(holder) }
    }

    /// `new EitherHolder(key)`.
    pub fn of_key(key: ResourceKey<T>) -> Self {
        Self { contents: Either::Right(key) }
    }

    /// `key()` — `Optional<ResourceKey>`: для Left возвращает
    /// ключ registry-holder'а (у нас `Reference` хранит только id
    /// без привязки к реестру → None до порта реестров).
    pub fn key(&self) -> Option<&ResourceKey<T>> {
        match &self.contents {
            Either::Right(k) => Some(k),
            Either::Left(_) => None,
        }
    }

    /// `streamCodec(registryKey, holderCodec)` — registryKey нужен
    /// для восстановления `ResourceKey` при decode (на проводе —
    /// только Identifier).
    pub fn stream_codec(
        registry_name: Identifier,
        holder_codec: impl StreamCodec<FriendlyByteBuf, Holder<T>> + Send + Sync + 'static,
    ) -> impl StreamCodec<FriendlyByteBuf, EitherHolder<T>> + Send + Sync + 'static
    where
        T: Clone + Send + Sync + 'static,
    {
        let either = std::sync::Arc::new(byte_buf_codecs::either(
            holder_codec,
            byte_buf_codecs::IDENTIFIER,
        ));
        let either2 = either.clone();
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &EitherHolder<T>| {
                match &v.contents {
                    Either::Left(h) => either.encode(buf, &Either::Left(h.clone())),
                    Either::Right(k) => {
                        either.encode(buf, &Either::Right(k.identifier.clone()))
                    }
                }
            },
            move |buf: &mut FriendlyByteBuf| {
                match either2.decode(buf) {
                    Either::Left(h) => EitherHolder { contents: Either::Left(h) },
                    Either::Right(id) => EitherHolder {
                        contents: Either::Right(ResourceKey::create_with_registry_name(
                            registry_name.clone(),
                            id,
                        )),
                    },
                }
            },
        )
    }
}
