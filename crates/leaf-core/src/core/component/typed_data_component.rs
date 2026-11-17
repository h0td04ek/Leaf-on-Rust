//! Перенос `net.minecraft.core.component.TypedDataComponent`.

use crate::core::component::data_component_type::{
    data_component_type_stream_codec, ComponentValue, DataComponentType,
};
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use std::sync::Arc;

/// `TypedDataComponent<T>` — (type, value).
#[derive(Clone)]
pub struct TypedDataComponent {
    /// `type`.
    pub component_type: Arc<DataComponentType>,
    /// `value`.
    pub value: ComponentValue,
}

impl TypedDataComponent {
    /// `createUnchecked(type, value)`.
    pub fn create_unchecked(component_type: Arc<DataComponentType>, value: ComponentValue) -> Self {
        Self { component_type, value }
    }

    /// Типизированный доступ к значению.
    pub fn value_as<T: 'static>(&self) -> Option<&T> {
        self.value.as_any().downcast_ref::<T>()
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, TypedDataComponent> {
        sync_fn(
            |buf: &mut FriendlyByteBuf, c: &TypedDataComponent| {
                data_component_type_stream_codec().encode(buf, &c.component_type);
                let codec = c.component_type.stream_codec.as_ref().expect(
                    "component type has no stream codec",
                );
                codec.encode(buf, c.value.as_any());
            },
            |buf: &mut FriendlyByteBuf| {
                let component_type = data_component_type_stream_codec().decode(buf);
                let codec = component_type
                    .stream_codec
                    .as_ref()
                    .expect("component type has no stream codec");
                let value = codec.decode(buf);
                TypedDataComponent { component_type, value }
            },
        )
    }
}

impl PartialEq for TypedDataComponent {
    fn eq(&self, other: &Self) -> bool {
        self.component_type.id == other.component_type.id && self.value.value_eq(other.value.as_ref())
    }
}

impl std::fmt::Debug for TypedDataComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypedDataComponent")
            .field("type", &self.component_type.id)
            .field("value", &self.value.value_debug())
            .finish()
    }
}
