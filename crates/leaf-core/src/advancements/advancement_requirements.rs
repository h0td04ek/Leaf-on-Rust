//! Перенос `net.minecraft.advancements.AdvancementRequirements`.

use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `AdvancementRequirements` — `List<List<String>>` требований.
#[derive(Clone, Debug, Default)]
pub struct AdvancementRequirements {
    /// `requirements` — список списков имён критериев.
    pub requirements: Vec<Vec<String>>,
}

impl AdvancementRequirements {
    /// `EMPTY`.
    pub const EMPTY: Self = Self { requirements: Vec::new() };

    /// `new(requirements)`.
    pub fn new(requirements: Vec<Vec<String>>) -> Self {
        Self { requirements }
    }

    /// `allOf` — каждый критерий отдельным требованием.
    pub fn all_of(requirements: Vec<String>) -> Self {
        Self {
            requirements: requirements.into_iter().map(|s| vec![s]).collect(),
        }
    }

    /// `anyOf` — все критерии в одном требовании (OR).
    pub fn any_of(criteria: Vec<String>) -> Self {
        Self { requirements: vec![criteria] }
    }

    /// `size`.
    pub fn size(&self) -> usize {
        self.requirements.len()
    }

    /// `write`.
    pub fn write(&self, buffer: &mut FriendlyByteBuf) {
        buffer.write_collection(&self.requirements, |b, req| {
            b.write_collection(req, |b2, s| {
                let _ = b2.write_utf_n(s, 32767);
            });
        });
    }

    /// `AdvancementRequirements(buffer)`.
    pub fn from_network(buffer: &mut FriendlyByteBuf) -> Self {
        let requirements = buffer.read_collection(|n| Vec::with_capacity(n), |b| {
            b.read_collection(|n2| Vec::with_capacity(n2), |b2| b2.read_utf_n(32767).unwrap_or_default())
        });
        Self { requirements }
    }
}
