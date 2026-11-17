//! Перенос `net.minecraft.network.chat.HoverEvent` (модель данных).

use crate::network::chat::component::Component;
use crate::resources::identifier::Identifier;
use crate::util::uuid::Uuid;
use crate::world::item::ItemStack;
use std::sync::OnceLock;

/// `HoverEvent` — действие по наведению на компоненте.
#[derive(Clone, Debug, PartialEq)]
pub enum HoverEvent {
    /// `show_text`.
    ShowText(Box<Component>),
    /// `show_item`.
    ShowItem(ItemStack),
    /// `ShowEntity(EntityTooltipInfo)`.
    ShowEntity(EntityTooltipInfo),
}

impl HoverEvent {
    /// `action()`.
    pub fn action(&self) -> HoverEventAction {
        match self {
            HoverEvent::ShowText(_) => HoverEventAction::ShowText,
            HoverEvent::ShowItem(_) => HoverEventAction::ShowItem,
            HoverEvent::ShowEntity(_) => HoverEventAction::ShowEntity,
        }
    }
}

/// `HoverEvent.Action`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HoverEventAction {
    ShowText,
    ShowItem,
    ShowEntity,
}

impl HoverEventAction {
    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            HoverEventAction::ShowText => "show_text",
            HoverEventAction::ShowItem => "show_item",
            HoverEventAction::ShowEntity => "show_entity",
        }
    }

    /// `isAllowedFromServer`.
    pub fn is_allowed_from_server(&self) -> bool {
        true
    }
}

/// `HoverEvent.EntityTooltipInfo` — тип сущности хранится как registry-id
/// (`EntityType`-реестр портируется отдельно).
#[derive(Clone, Debug)]
pub struct EntityTooltipInfo {
    /// Registry-ключ типа сущности.
    pub entity_type: Identifier,
    pub uuid: Uuid,
    pub name: Option<Box<Component>>,
    /// Кэш строк tooltip (как `linesCache` в Java).
    lines_cache: std::cell::RefCell<Option<Vec<Component>>>,
}

impl EntityTooltipInfo {
    pub fn new(entity_type: Identifier, uuid: Uuid, name: Option<Component>) -> Self {
        Self {
            entity_type,
            uuid,
            name: name.map(Box::new),
            lines_cache: std::cell::RefCell::new(None),
        }
    }

    /// `getTooltipLines` — имя, тип, uuid.
    pub fn get_tooltip_lines(&self) -> Vec<Component> {
        if let Some(lines) = self.lines_cache.borrow().as_ref() {
            return lines.clone();
        }
        let mut lines = Vec::new();
        if let Some(name) = &self.name {
            lines.push((**name).clone());
        }
        lines.push(Component::translatable(
            "gui.entity_tooltip.type",
            vec![crate::network::chat::component_contents::TranslatableArg::Component(
                Box::new(Component::literal(&self.entity_type.to_string())),
            )],
        ));
        lines.push(Component::literal(&self.uuid.to_string()));
        *self.lines_cache.borrow_mut() = Some(lines.clone());
        lines
    }
}

impl PartialEq for EntityTooltipInfo {
    fn eq(&self, other: &Self) -> bool {
        self.entity_type == other.entity_type
            && self.uuid == other.uuid
            && self.name == other.name
    }
}

impl Eq for EntityTooltipInfo {}
