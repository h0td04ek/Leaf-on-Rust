//! Перенос `net.minecraft.tags.TagKey<T>` — (registry, location).
//!
//! `streamCodec` пишет только `Identifier` (registry известен
//! из контекста типа компонента).

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use crate::core::registry::Registry;
use std::marker::PhantomData;

/// `TagKey<T>` — ключ тега в реестре `registry`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TagKey<T> {
    /// `registry`.
    pub registry: ResourceKey<Registry<T>>,
    /// `location`.
    pub location: Identifier,
    _t: PhantomData<fn() -> T>,
}

impl<T> TagKey<T> {
    /// `TagKey.create(registry, location)`.
    pub fn create(registry: ResourceKey<Registry<T>>, location: Identifier) -> Self {
        Self { registry, location, _t: PhantomData }
    }

    /// `TagKey.location` getter.
    pub fn location(&self) -> &Identifier {
        &self.location
    }

    /// `TagKey.registry` getter.
    pub fn registry(&self) -> &ResourceKey<Registry<T>> {
        &self.registry
    }

    /// `streamCodec(registry)` — пишет только location.
    pub fn stream_codec(
        registry: ResourceKey<Registry<T>>,
    ) -> impl StreamCodec<FriendlyByteBuf, TagKey<T>> + Send + Sync + 'static
    where
        T: Send + Sync + 'static,
    {
        sync_fn(
            move |buf: &mut FriendlyByteBuf, tag: &TagKey<T>| {
                buf.write_identifier(&tag.location).expect("tag key id");
            },
            move |buf: &mut FriendlyByteBuf| {
                let location = buf.read_identifier().expect("tag key id");
                TagKey::create(registry.clone(), location)
            },
        )
    }
}
