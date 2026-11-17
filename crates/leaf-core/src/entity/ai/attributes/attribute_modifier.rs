//! Перенос `net.minecraft.world.entity.ai.attributes.AttributeModifier`.

use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// `AttributeModifier.Operation`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttributeModifierOperation {
    /// `ADD_VALUE` (id 0).
    AddValue,
    /// `ADD_MULTIPLIED_BASE` (id 1).
    AddMultipliedBase,
    /// `ADD_MULTIPLIED_TOTAL` (id 2).
    AddMultipliedTotal,
}

/// Все значения (`values()`).
pub const ATTRIBUTE_MODIFIER_OPERATIONS: [AttributeModifierOperation; 3] = [
    AttributeModifierOperation::AddValue,
    AttributeModifierOperation::AddMultipliedBase,
    AttributeModifierOperation::AddMultipliedTotal,
];

impl AttributeModifierOperation {
    /// `values()`.
    pub fn values() -> &'static [AttributeModifierOperation] {
        &ATTRIBUTE_MODIFIER_OPERATIONS
    }

    /// `id`.
    pub fn id(&self) -> i32 {
        match self {
            AttributeModifierOperation::AddValue => 0,
            AttributeModifierOperation::AddMultipliedBase => 1,
            AttributeModifierOperation::AddMultipliedTotal => 2,
        }
    }

    /// `getSerializedName` (`StringRepresentable`).
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            AttributeModifierOperation::AddValue => "add_value",
            AttributeModifierOperation::AddMultipliedBase => "add_multiplied_base",
            AttributeModifierOperation::AddMultipliedTotal => "add_multiplied_total",
        }
    }

    /// `BY_ID` — `ByIdMap.continuous` + `OutOfBoundsStrategy.ZERO`.
    pub fn by_id(id: i32) -> AttributeModifierOperation {
        if (0..3).contains(&id) {
            ATTRIBUTE_MODIFIER_OPERATIONS[id as usize]
        } else {
            ATTRIBUTE_MODIFIER_OPERATIONS[0]
        }
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.idMapper(BY_ID, id)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, AttributeModifierOperation> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, v: &AttributeModifierOperation| {
                buf.write_var_int(v.id());
            },
            |buf: &mut FriendlyByteBuf| AttributeModifierOperation::by_id(buf.read_var_int()),
        )
    }
}

/// `AttributeModifier` — (id, amount, operation).
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeModifier {
    /// `id` — `Identifier` (в Java — `ResourceLocation`).
    pub id: Identifier,
    /// `amount`.
    pub amount: f64,
    /// `operation`.
    pub operation: AttributeModifierOperation,
}

impl AttributeModifier {
    /// `AttributeModifier::new`.
    pub fn new(id: Identifier, amount: f64, operation: AttributeModifierOperation) -> Self {
        Self { id, amount, operation }
    }

    /// `is(id)`.
    pub fn is(&self, id: &Identifier) -> bool {
        &self.id == id
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, AttributeModifier> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, m: &AttributeModifier| {
                buf.write_identifier(&m.id).expect("attribute modifier id");
                buf.write_double(m.amount);
                buf.write_var_int(m.operation.id());
            },
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_identifier().expect("attribute modifier id");
                let amount = buf.read_double();
                let operation = AttributeModifierOperation::by_id(buf.read_var_int());
                AttributeModifier::new(id, amount, operation)
            },
        )
    }
}
