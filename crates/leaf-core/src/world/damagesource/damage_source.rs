//! Перенос `net.minecraft.world.damagesource.DamageSource` — источник урона.

use crate::core::holder::Holder;
use crate::world::damagesource::damage_type::DamageType;
use crate::world::phys::vec3::Vec3;

/// `DamageSource` — источник урона.
#[derive(Clone)]
pub struct DamageSource {
    /// `type` — тип урона.
    pub ty: Holder<DamageType>,
    /// `causingEntity` — сущность-причина.
    pub causing_entity: Option<crate::world::entity::entity::Entity>,
    /// `directEntity` — прямая сущность.
    pub direct_entity: Option<crate::world::entity::entity::Entity>,
    /// `damageSourcePosition` — позиция источника.
    pub damage_source_position: Option<Vec3>,
}

impl DamageSource {
    /// `new(type)`.
    pub fn new(ty: Holder<DamageType>) -> Self {
        Self {
            ty,
            causing_entity: None,
            direct_entity: None,
            damage_source_position: None,
        }
    }

    /// `new(type, causingEntity, directEntity)`.
    pub fn with_entities(
        ty: Holder<DamageType>,
        causing_entity: Option<crate::world::entity::entity::Entity>,
        direct_entity: Option<crate::world::entity::entity::Entity>,
    ) -> Self {
        Self {
            ty,
            causing_entity,
            direct_entity,
            damage_source_position: None,
        }
    }

    /// `new(type, position)`.
    pub fn with_position(ty: Holder<DamageType>, pos: Vec3) -> Self {
        Self {
            ty,
            causing_entity: None,
            direct_entity: None,
            damage_source_position: Some(pos),
        }
    }

    /// `getType()`.
    pub fn get_type(&self) -> &Holder<DamageType> {
        &self.ty
    }

    /// `getCausingEntity()`.
    pub fn get_causing_entity(&self) -> Option<&crate::world::entity::entity::Entity> {
        self.causing_entity.as_ref()
    }

    /// `getDirectEntity()`.
    pub fn get_direct_entity(&self) -> Option<&crate::world::entity::entity::Entity> {
        self.direct_entity.as_ref()
    }

    /// `getDirectEntity() -> Option<usize>` для ID.
    pub fn get_direct_entity_id(&self) -> Option<i32> {
        self.direct_entity.as_ref().map(|e| e.get_id())
    }

    /// `getCausingEntity() -> Option<usize>` для ID.
    pub fn get_causing_entity_id(&self) -> Option<i32> {
        self.causing_entity.as_ref().map(|e| e.get_id())
    }

    /// `getSourcePosition()`.
    pub fn get_source_position(&self) -> Option<Vec3> {
        self.damage_source_position
    }

    /// `is(DamageType)` — проверка типа.
    pub fn is(&self, damage_type: &Holder<DamageType>) -> bool {
        match (self.ty.value(), damage_type.value()) {
            (Some(a), Some(b)) => std::ptr::eq(a, b) || a == b,
            _ => false,
        }
    }

    /// `is(tag)` — проверка по тегу (упрощённо).
    pub fn is_tag(&self, _tag: &crate::tags::tag_key::TagKey<DamageType>) -> bool {
        false
    }

    /// `getLocalizedDeathMessage(entity)`.
    pub fn get_localized_death_message(&self, entity: &crate::world::entity::entity::Entity) -> crate::network::chat::component::Component {
        let name = entity.get_display_name();
        match self.ty.value() {
            Some(dt) => {
                crate::network::chat::component::Component::translatable(
                    format!("death.attack.{}", dt.msg_id),
                    vec![crate::network::chat::component_contents::TranslatableArg::Component(Box::new(name))],
                )
            }
            None => crate::network::chat::component::Component::literal(format!("{} died", entity.get_name_string())),
        }
    }

    /// `getEntityDamageSource()` — для API совместимости.
    pub fn get_entity_damage_source(&self) -> Option<&crate::world::entity::entity::Entity> {
        self.causing_entity.as_ref().or(self.direct_entity.as_ref())
    }

    /// `copy()`.
    pub fn copy(&self) -> Self {
        Self {
            ty: self.ty.clone(),
            causing_entity: None,
            direct_entity: None,
            damage_source_position: self.damage_source_position,
        }
    }

    /// `scalesWithDifficulty()`.
    pub fn scales_with_difficulty(&self) -> bool {
        self.ty.value().map(|dt| dt.scaling).unwrap_or(crate::world::damagesource::damage_scaling::DamageScaling::WhenCausedByLivingNonPlayer) == crate::world::damagesource::damage_scaling::DamageScaling::WhenCausedByLivingNonPlayer
    }

    /// `getFoodExhaustion()`.
    pub fn get_food_exhaustion(&self) -> f32 {
        self.ty.value().map(|dt| dt.exhaustion).unwrap_or(0.0)
    }

    /// `getEffects()`.
    pub fn get_effects(&self) -> crate::world::damagesource::damage_effects::DamageEffects {
        self.ty.value().map(|dt| dt.effects).unwrap_or(crate::world::damagesource::damage_effects::DamageEffects::Hurt)
    }
}

impl std::fmt::Debug for DamageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DamageSource")
            .field("type", &self.ty)
            .field("causing_entity", &self.causing_entity.as_ref().map(|e| e.get_id()))
            .field("direct_entity", &self.direct_entity.as_ref().map(|e| e.get_id()))
            .field("damage_source_position", &self.damage_source_position)
            .finish()
    }
}

impl PartialEq for DamageSource {
    fn eq(&self, other: &Self) -> bool {
        self.ty == other.ty
    }
}

impl Eq for DamageSource {}

impl std::hash::Hash for DamageSource {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ty.hash(state);
    }
}

// Entity needs Clone for DamageSource copy — implement manually
impl Clone for crate::world::entity::entity::Entity {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            uuid: self.uuid,
            entity_type: self.entity_type.clone(),
            level: self.level.clone(),
            pos: self.pos,
            old_pos: self.old_pos,
            rotation: self.rotation,
            old_rotation: self.old_rotation,
            velocity: self.velocity,
            on_ground: self.on_ground,
            horizontal_collision: self.horizontal_collision,
            vertical_collision: self.vertical_collision,
            hurt_time: self.hurt_time,
            hurt_duration: self.hurt_duration,
            death_time: self.death_time,
            tick_count: self.tick_count,
            first_tick: self.first_tick,
            removed: self.removed,
            removal_reason: self.removal_reason,
            name: self.name.clone(),
            custom_name: self.custom_name.clone(),
            bounding_box: self.bounding_box,
            dimensions: self.dimensions,
            eye_height: self.eye_height,
            eye_height_old: self.eye_height_old,
            invisible: self.invisible,
            glowing: self.glowing,
            no_physics: self.no_physics,
            shared_flags: self.shared_flags,
            data: self.data.clone(),
            tags: self.tags.clone(),
            persistence_required: self.persistence_required,
        }
    }
}
