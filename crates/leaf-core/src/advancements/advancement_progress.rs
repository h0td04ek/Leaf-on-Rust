//! Перенос `net.minecraft.advancements.AdvancementProgress`.

use crate::advancements::criterion_progress::CriterionProgress;
use crate::advancements::advancement_requirements::AdvancementRequirements;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use std::collections::HashMap;

/// `AdvancementProgress` — прогресс advancement.
#[derive(Clone, Debug, Default)]
pub struct AdvancementProgress {
    /// `criteria` — map criterion name → progress.
    pub criteria: HashMap<String, CriterionProgress>,
    /// `requirements` — требования (не сериализуются).
    pub requirements: AdvancementRequirements,
}

impl AdvancementProgress {
    /// `new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `serializeToNetwork`.
    pub fn serialize_to_network(&self, buffer: &mut FriendlyByteBuf) {
        buffer.write_map(&self.criteria, |b, k| {
            let _ = b.write_utf_n(k, 32767);
        }, |b, v| {
            v.serialize_to_network(b);
        });
    }

    /// `fromNetwork`.
    pub fn from_network(buffer: &mut FriendlyByteBuf) -> Self {
        let criteria = buffer.read_map(
            |n| HashMap::with_capacity(n),
            |b| b.read_utf_n(32767).unwrap_or_default(),
            |b| CriterionProgress::from_network(b),
        );
        Self { criteria, requirements: AdvancementRequirements::EMPTY }
    }

    /// `getCriterion`.
    pub fn get_criterion(&self, name: &str) -> Option<&CriterionProgress> {
        self.criteria.get(name)
    }

    /// `isDone`.
    pub fn is_done(&self) -> bool {
        self.criteria.values().all(|c| c.is_done())
    }

    /// `getPercent`.
    pub fn get_percent(&self) -> f32 {
        if self.criteria.is_empty() {
            return 0.0;
        }
        let total = self.requirements.size() as f32;
        if total == 0.0 {
            return 0.0;
        }
        let done = self.criteria.values().filter(|c| c.is_done()).count() as f32;
        done / total
    }
}

impl PartialEq for AdvancementProgress {
    fn eq(&self, other: &Self) -> bool {
        self.criteria == other.criteria
    }
}
impl Eq for AdvancementProgress {}
impl std::hash::Hash for AdvancementProgress {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (k, v) in &self.criteria {
            k.hash(state);
            v.hash(state);
        }
    }
}
