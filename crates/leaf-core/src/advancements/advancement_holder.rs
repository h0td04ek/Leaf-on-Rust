//! Перенос `net.minecraft.advancements.AdvancementHolder`.

use crate::advancements::advancement::Advancement;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use std::sync::Arc;

/// `AdvancementHolder` — `{ id, value }`.
pub struct AdvancementHolder {
    /// `id`.
    pub id: Identifier,
    /// `value` — Arc для shared ownership.
    pub value: Arc<Advancement>,
}

impl AdvancementHolder {
    /// `new(id, value)`.
    pub fn new(id: Identifier, value: Advancement) -> Self {
        Self { id, value: Arc::new(value) }
    }

    /// `new_arc(id, value)`.
    pub fn new_arc(id: Identifier, value: Arc<Advancement>) -> Self {
        Self { id, value }
    }

    /// `STREAM_CODEC` — encode.
    pub fn encode(&self, buffer: &mut FriendlyByteBuf) {
        buffer.write_identifier(&self.id);
        self.value.write(buffer);
    }

    /// `STREAM_CODEC` — decode.
    pub fn decode(buffer: &mut FriendlyByteBuf) -> Self {
        let id = buffer.read_identifier().unwrap_or_else(|_| Identifier::from_namespace_and_path("minecraft", "unknown").unwrap_or_else(|_| Identifier::parse("minecraft:unknown").unwrap()));
        let value = Advancement::read(buffer);
        Self::new(id, value)
    }

    /// `LIST_STREAM_CODEC` — encode list.
    pub fn encode_list(list: &[AdvancementHolder], buffer: &mut FriendlyByteBuf) {
        buffer.write_collection(list, |b, h| h.encode(b));
    }

    /// `LIST_STREAM_CODEC` — decode list.
    pub fn decode_list(buffer: &mut FriendlyByteBuf) -> Vec<AdvancementHolder> {
        buffer.read_collection(|n| Vec::with_capacity(n), |b| Self::decode(b))
    }
}

impl PartialEq for AdvancementHolder {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for AdvancementHolder {}

impl std::hash::Hash for AdvancementHolder {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl std::fmt::Debug for AdvancementHolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AdvancementHolder{{id: {:?}}}", self.id)
    }
}
