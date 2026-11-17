//! Перенос `DiscardedQueryPayload` — см. [`super::custom_query_payload::CustomQueryPayload`].

use crate::resources::identifier::Identifier;

/// Конструктор `DiscardedQueryPayload(id)`.
pub fn discarded_query_payload(id: Identifier) -> super::custom_query_payload::CustomQueryPayload {
    super::custom_query_payload::CustomQueryPayload::Discarded { id }
}
