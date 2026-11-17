//! Перенос `net.minecraft.core.component.DataComponentExactPredicate` —
//! точное совпадение набора компонентов.
//!
//! Network (`STREAM_CODEC`) — `list(TypedDataComponent.STREAM_CODEC)`.
//! NBT (`CODEC` = `DataComponentType.VALUE_MAP_CODEC`) — compound
//! `{componentId: value}`; значение кодируется persistent-CODEC
//! компонента (`nbt_codec` в `DataComponentType`, портируется
//! инкрементально — для типов без NBT-кодека encode вернёт ошибку,
//! как `codecOrThrow` в Java).

use crate::core::component::data_component_type::{
    data_component_type_id_by_identifier, data_component_type_arc_by_id, ComponentValue,
    DataComponentType,
};
use crate::core::component::typed_data_component::TypedDataComponent;
use std::any::Any;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::resources::identifier::Identifier;
use std::sync::Arc;

/// `DataComponentExactPredicate` — список ожидаемых компонентов.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DataComponentExactPredicate {
    /// `expectedComponents`.
    pub expected_components: Vec<TypedDataComponent>,
}

impl DataComponentExactPredicate {
    /// `EMPTY`.
    pub const EMPTY: DataComponentExactPredicate =
        DataComponentExactPredicate { expected_components: Vec::new() };

    /// `new`.
    pub fn new(expected_components: Vec<TypedDataComponent>) -> Self {
        Self { expected_components }
    }

    /// `expect(component, value)` — один компонент.
    pub fn expect<T>(component: Arc<DataComponentType>, value: T) -> Self
    where
        T: AnyValueComponent,
    {
        Self::new(vec![TypedDataComponent::create_unchecked(
            component,
            Box::new(value) as ComponentValue,
        )])
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.expected_components.is_empty()
    }

    /// `test(DataComponentGetter)` — все ожидаемые компоненты
    /// точно равны значениям геттера.
    pub fn test(&self, getter: &dyn crate::core::component::data_component_getter::DataComponentGetter) -> bool {
        self.expected_components.iter().all(|expected| {
            getter
                .get_raw(&expected.component_type)
                .is_some_and(|actual| expected.value.value_eq(actual.as_ref()))
        })
    }

    /// `alwaysMatches`.
    pub fn always_matches(&self) -> bool {
        self.expected_components.is_empty()
    }

    /// `STREAM_CODEC` — `list(TypedDataComponent.STREAM_CODEC)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, DataComponentExactPredicate> + Send + Sync + 'static {
        let list_enc = byte_buf_codecs::list(TypedDataComponent::stream_codec());
        let list_dec = byte_buf_codecs::list(TypedDataComponent::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &DataComponentExactPredicate| {
                list_enc.encode(buf, &v.expected_components);
            },
            move |buf: &mut FriendlyByteBuf| DataComponentExactPredicate {
                expected_components: list_dec.decode(buf),
            },
        )
    }

    /// `CODEC` → NBT (`VALUE_MAP_CODEC` — compound
    /// `{componentId: value}`).
    pub fn encode_nbt(&self) -> Result<Tag, String> {
        let mut c = CompoundTag::new();
        for component in &self.expected_components {
            let id = component.component_type.id.to_string();
            let tag = component
                .component_type
                .nbt_codec
                .as_ref()
                .ok_or_else(|| {
                    format!("component '{}' has no persistent codec", id)
                })?
                .encode(component.value.as_any());
            c.put(id, tag?);
        }
        Ok(Tag::Compound(c))
    }

    /// `CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = match tag {
            Tag::Compound(c) => c,
            _ => return Err("expected compound for DataComponentExactPredicate".into()),
        };
        let mut out = Vec::new();
        for (key, value_tag) in c.iter() {
            let id = Identifier::parse(key).map_err(|e| format!("bad component id: {e}"))?;
            let type_id = data_component_type_id_by_identifier(&id)
                .ok_or_else(|| format!("unknown component type '{key}'"))?;
            let component_type = crate::core::component::data_component_type::data_component_type_arc_by_id(type_id)
                .ok_or_else(|| format!("unknown component type id {type_id}"))?;
            let value = component_type
                .nbt_codec
                .as_ref()
                .ok_or_else(|| format!("component '{key}' has no persistent codec"))?
                .decode(value_tag)?;
            out.push(TypedDataComponent::create_unchecked(component_type.clone(), value));
        }
        Ok(Self::new(out))
    }
}

/// Маркер для `expect` — значение должно быть `Any + Send + Sync`.
pub trait AnyValueComponent: Any + Send + Sync + Clone + PartialEq + std::fmt::Debug + 'static {}
impl<T: Any + Send + Sync + Clone + PartialEq + std::fmt::Debug + 'static> AnyValueComponent for T {}
