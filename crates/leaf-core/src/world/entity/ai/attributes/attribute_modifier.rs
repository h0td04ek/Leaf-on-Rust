//! Перенос `net.minecraft.world.entity.ai.attributes.AttributeModifier`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// `AttributeModifier.Operation` — id = wire.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttributeModifierOperation {
    /// `ADD_VALUE`.
    ADD_VALUE,
    /// `ADD_MULTIPLIED_BASE`.
    ADD_MULTIPLIED_BASE,
    /// `ADD_MULTIPLIED_TOTAL`.
    ADD_MULTIPLIED_TOTAL,
}

impl AttributeModifierOperation {
    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            AttributeModifierOperation::ADD_VALUE => "add_value",
            AttributeModifierOperation::ADD_MULTIPLIED_BASE => "add_multiplied_base",
            AttributeModifierOperation::ADD_MULTIPLIED_TOTAL => "add_multiplied_total",
        }
    }

    /// `getId`.
    pub fn get_id(&self) -> i32 {
        *self as i32
    }

    /// `BY_ID` — `OutOfBoundsStrategy.ZERO`.
    pub fn by_id(id: i32) -> AttributeModifierOperation {
        match id {
            0 => AttributeModifierOperation::ADD_VALUE,
            1 => AttributeModifierOperation::ADD_MULTIPLIED_BASE,
            2 => AttributeModifierOperation::ADD_MULTIPLIED_TOTAL,
            _ => AttributeModifierOperation::ADD_VALUE,
        }
    }

    /// `STREAM_CODEC` — idMapper.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, AttributeModifierOperation> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &AttributeModifierOperation| {
                buf.write_var_int(v.get_id());
            },
            |buf: &mut FriendlyByteBuf| AttributeModifierOperation::by_id(buf.read_var_int()),
        )
    }
}

/// `AttributeModifier` — (id, amount, operation).
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeModifier {
    /// `id`.
    pub id: Identifier,
    /// `amount`.
    pub amount: f64,
    /// `operation`.
    pub operation: AttributeModifierOperation,
}

impl AttributeModifier {
    /// `new`.
    pub fn new(id: Identifier, amount: f64, operation: AttributeModifierOperation) -> Self {
        Self { id, amount, operation }
    }

    /// `is(id)`.
    pub fn is(&self, id: &Identifier) -> bool {
        self.id == *id
    }

    /// `STREAM_CODEC` — IDENTIFIER + DOUBLE + Operation.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, AttributeModifier> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &AttributeModifier| {
                buf.write_identifier(&v.id).expect("attribute modifier id");
                buf.write_double(v.amount);
                AttributeModifierOperation::stream_codec().encode(buf, &v.operation);
            },
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_identifier().expect("attribute modifier id");
                let amount = buf.read_double();
                let operation = AttributeModifierOperation::stream_codec().decode(buf);
                AttributeModifier::new(id, amount, operation)
            },
        )
    }
}
