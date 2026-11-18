//! Перенос `net.minecraft.advancements.Advancement`.

use crate::advancements::advancement_requirements::AdvancementRequirements;
use crate::advancements::advancement_rewards::AdvancementRewards;
use crate::advancements::display_info::DisplayInfo;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use std::collections::HashMap;

/// `Criterion` — критерий advancement (упрощённая версия для network).
#[derive(Debug, Default)]
pub struct Criterion {
    /// `trigger` — имя триггера.
    pub trigger: String,
    /// `triggerInstance` — type-erased условия.
    pub trigger_instance: Option<Box<dyn std::any::Any + Send + Sync>>,
}

/// `Advancement` — advancement.
#[derive(Debug)]
pub struct Advancement {
    /// `parent`.
    pub parent: Option<Identifier>,
    /// `display`.
    pub display: Option<DisplayInfo>,
    /// `rewards`.
    pub rewards: AdvancementRewards,
    /// `criteria`.
    pub criteria: HashMap<String, Criterion>,
    /// `requirements`.
    pub requirements: AdvancementRequirements,
    /// `sendsTelemetryEvent`.
    pub sends_telemetry_event: bool,
    /// `name` — вычисленное имя.
    pub name: Option<crate::network::chat::component::Component>,
}

impl Advancement {
    /// `new(...)` — полная версия.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        parent: Option<Identifier>,
        display: Option<DisplayInfo>,
        rewards: AdvancementRewards,
        criteria: HashMap<String, Criterion>,
        requirements: AdvancementRequirements,
        sends_telemetry_event: bool,
    ) -> Self {
        Self {
            parent, display, rewards, criteria, requirements,
            sends_telemetry_event,
            name: None,
        }
    }

    /// `write` — сериализация для network.
    pub fn write(&self, buffer: &mut FriendlyByteBuf) {
        match &self.parent {
            Some(id) => {
                buffer.write_bool(true);
                let _ = buffer.write_identifier(id);
            }
            None => {
                buffer.write_bool(false);
            }
        }
        match &self.display {
            Some(d) => {
                buffer.write_bool(true);
                d.serialize_to_network(buffer);
            }
            None => {
                buffer.write_bool(false);
            }
        }
        self.requirements.write(buffer);
        buffer.write_bool(self.sends_telemetry_event);
    }

    /// `read` — десериализация для network.
    pub fn read(buffer: &mut FriendlyByteBuf) -> Self {
        let parent = if buffer.read_bool() {
            Some(buffer.read_identifier().unwrap_or_else(|_| Identifier::from_namespace_and_path("minecraft", "unknown").unwrap_or_else(|_| Identifier::parse("minecraft:unknown").unwrap())))
        } else {
            None
        };
        let display = if buffer.read_bool() {
            Some(DisplayInfo::from_network(buffer))
        } else {
            None
        };
        let requirements = AdvancementRequirements::from_network(buffer);
        let sends_telemetry_event = buffer.read_bool();
        Self {
            parent,
            display,
            rewards: AdvancementRewards::EMPTY,
            criteria: HashMap::new(),
            requirements,
            sends_telemetry_event,
            name: None,
        }
    }

    /// `isRoot`.
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}
