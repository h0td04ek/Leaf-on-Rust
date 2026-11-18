//! Перенос `net.minecraft.world.entity.EntityType` — тип сущности.

use std::sync::Arc;

use crate::core::holder::Holder;
use crate::resources::identifier::Identifier;
use crate::world::entity::mob_category::MobCategory;

/// `EntityType` — тип сущности.
#[derive(Clone)]
pub struct EntityType {
    /// `id` — идентификатор.
    pub id: Identifier,
    /// `category` — категория.
    pub category: MobCategory,
    /// `width` — ширина.
    pub width: f32,
    /// `height` — высота.
    pub height: f32,
    /// `eye_height` — высота глаз.
    pub eye_height: f32,
    /// `max_health` — максимальное здоровье.
    pub max_health: f32,
    /// `armor` — броня.
    pub armor: f32,
    /// `armor_toughness` — прочность брони.
    pub armor_toughness: f32,
    /// `attack_damage` — урон атаки.
    pub attack_damage: f32,
    /// `attack_knockback` — отбрасывание атаки.
    pub attack_knockback: f32,
    /// `speed` — скорость.
    pub speed: f32,
    /// `fly_speed` — скорость полёта.
    pub fly_speed: f32,
    /// `follow_range` — дальность следования.
    pub follow_range: f32,
    /// `knockback_resistance` — сопротивление отбрасыванию.
    pub knockback_resistance: f32,
    /// `jump_strength` — сила прыжка.
    pub jump_strength: f32,
    /// `spawn_egg` — яйцо призыва.
    pub spawn_egg: Option<Identifier>,
    /// `experience_reward` — награда опыта.
    pub experience_reward: i32,
    /// `fire_immune` — иммунитет к огню.
    pub fire_immune: bool,
    /// `can_spawn_far_from_player` — может ли спавниться далеко.
    pub can_spawn_far_from_player: bool,
    /// `can_breathe_underwater` — может ли дышать под водой.
    pub can_breathe_underwater: bool,
    /// `client_tracking_range` — дальность отслеживания клиента.
    pub client_tracking_range: i32,
    /// `update_interval` — интервал обновления.
    pub update_interval: i32,
    /// `tracking_tick_interval` — интервал тиков отслеживания.
    pub tracking_tick_interval: i32,
    /// `required_features` — требуемые функции.
    pub required_features: Vec<Identifier>,
    /// `dimensions` — размеры.
    pub dimensions: EntityDimensions,
}

/// `EntityDimensions` — размеры сущности.
#[derive(Clone, Copy, Debug)]
pub struct EntityDimensions {
    /// `width` — ширина.
    pub width: f32,
    /// `height` — высота.
    pub height: f32,
    /// `eye_height` — высота глаз.
    pub eye_height: f32,
    /// `fixed` — фиксированные размеры.
    pub fixed: bool,
}

impl EntityDimensions {
    /// `new(width, height, eyeHeight, fixed)`.
    pub fn new(width: f32, height: f32, eye_height: f32, fixed: bool) -> Self {
        Self { width, height, eye_height, fixed }
    }

    /// `scalable(width, height)`.
    pub fn scalable(width: f32, height: f32) -> Self {
        Self::new(width, height, height * 0.85, false)
    }

    /// `fixed(width, height)`.
    pub fn fixed(width: f32, height: f32) -> Self {
        Self::new(width, height, height * 0.85, true)
    }
}

impl EntityType {
    /// `new(...)` — создаёт новый EntityType.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Identifier,
        category: MobCategory,
        width: f32,
        height: f32,
        eye_height: f32,
        max_health: f32,
        armor: f32,
        armor_toughness: f32,
        attack_damage: f32,
        attack_knockback: f32,
        speed: f32,
        fly_speed: f32,
        follow_range: f32,
        knockback_resistance: f32,
        jump_strength: f32,
        spawn_egg: Option<Identifier>,
        experience_reward: i32,
        fire_immune: bool,
        can_spawn_far_from_player: bool,
        can_breathe_underwater: bool,
        client_tracking_range: i32,
        update_interval: i32,
        tracking_tick_interval: i32,
        required_features: Vec<Identifier>,
    ) -> Self {
        let dimensions = EntityDimensions::new(width, height, eye_height, false);
        Self {
            id,
            category,
            width,
            height,
            eye_height,
            max_health,
            armor,
            armor_toughness,
            attack_damage,
            attack_knockback,
            speed,
            fly_speed,
            follow_range,
            knockback_resistance,
            jump_strength,
            spawn_egg,
            experience_reward,
            fire_immune,
            can_spawn_far_from_player,
            can_breathe_underwater,
            client_tracking_range,
            update_interval,
            tracking_tick_interval,
            required_features,
            dimensions,
        }
    }

    /// `getId()`.
    pub fn get_id(&self) -> &Identifier {
        &self.id
    }

    /// `getCategory()`.
    pub fn get_category(&self) -> MobCategory {
        self.category
    }

    /// `getDimensions()`.
    pub fn get_dimensions(&self) -> EntityDimensions {
        self.dimensions
    }

    /// `getMaxHealth()`.
    pub fn get_max_health(&self) -> f32 {
        self.max_health
    }

    /// `getArmor()`.
    pub fn get_armor(&self) -> f32 {
        self.armor
    }

    /// `getArmorToughness()`.
    pub fn get_armor_toughness(&self) -> f32 {
        self.armor_toughness
    }

    /// `getAttackDamage()`.
    pub fn get_attack_damage(&self) -> f32 {
        self.attack_damage
    }

    /// `getAttackKnockback()`.
    pub fn get_attack_knockback(&self) -> f32 {
        self.attack_knockback
    }

    /// `getSpeed()`.
    pub fn get_speed(&self) -> f32 {
        self.speed
    }

    /// `getFlySpeed()`.
    pub fn get_fly_speed(&self) -> f32 {
        self.fly_speed
    }

    /// `getFollowRange()`.
    pub fn get_follow_range(&self) -> f32 {
        self.follow_range
    }

    /// `getKnockbackResistance()`.
    pub fn get_knockback_resistance(&self) -> f32 {
        self.knockback_resistance
    }

    /// `getJumpStrength()`.
    pub fn get_jump_strength(&self) -> f32 {
        self.jump_strength
    }

    /// `getSpawnEgg()`.
    pub fn get_spawn_egg(&self) -> Option<&Identifier> {
        self.spawn_egg.as_ref()
    }

    /// `getExperienceReward()`.
    pub fn get_experience_reward(&self) -> i32 {
        self.experience_reward
    }

    /// `isFireImmune()`.
    pub fn is_fire_immune(&self) -> bool {
        self.fire_immune
    }

    /// `canSpawnFarFromPlayer()`.
    pub fn can_spawn_far_from_player(&self) -> bool {
        self.can_spawn_far_from_player
    }

    /// `canBreatheUnderwater()`.
    pub fn can_breathe_underwater(&self) -> bool {
        self.can_breathe_underwater
    }

    /// `getClientTrackingRange()`.
    pub fn get_client_tracking_range(&self) -> i32 {
        self.client_tracking_range
    }

    /// `getUpdateInterval()`.
    pub fn get_update_interval(&self) -> i32 {
        self.update_interval
    }

    /// `getTrackingTickInterval()`.
    pub fn get_tracking_tick_interval(&self) -> i32 {
        self.tracking_tick_interval
    }

    /// `getRequiredFeatures()`.
    pub fn get_required_features(&self) -> &[Identifier] {
        &self.required_features
    }

    /// `getDescriptionId()`.
    pub fn get_description_id(&self) -> String {
        format!("entity.{}.{}", self.id.get_namespace(), self.id.get_path())
    }

    /// `toString()`.
    pub fn to_string(&self) -> String {
        self.id.to_string()
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl PartialEq for EntityType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for EntityType {}
impl std::fmt::Debug for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EntityType").field(&self.id).finish()
    }
}

/// Хелпер для типов без боевых характеристик (item, xp orb и т.п.).
#[allow(clippy::too_many_arguments)]
fn misc_entity_type(
    id: &str,
    width: f32,
    height: f32,
    eye_height: f32,
    client_tracking_range: i32,
    update_interval: i32,
) -> EntityType {
    EntityType::new(
        Identifier::with_default_namespace(id).unwrap(),
        MobCategory::Misc,
        width,
        height,
        eye_height,
        0.0, // max_health — не-живые сущности
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        None,
        0,
        true,  // fire_immune — предметы/сферы не горят
        true,  // can_spawn_far_from_player
        false, // can_breathe_underwater
        client_tracking_range,
        update_interval,
        update_interval,
        Vec::new(),
    )
}

/// `EntityType.ITEM` — `sized(0.25, 0.25).eyeHeight(0.2125)`.
pub fn item_entity_type() -> EntityType {
    misc_entity_type("item", 0.25, 0.25, 0.2125, 6, 20)
}

/// `EntityType.EXPERIENCE_ORB` — `sized(0.5, 0.5).eyeHeight(0.425)`.
pub fn experience_orb_entity_type() -> EntityType {
    misc_entity_type("experience_orb", 0.5, 0.5, 0.425, 6, 20)
}

/// `EntityType.SLIME` — `sized(0.52, 0.52).eyeHeight(0.442)`.
pub fn slime_entity_type() -> EntityType {
    misc_entity_type("slime", 0.52, 0.52, 0.442, 10, 10)
}

/// `EntityType.SILVERFISH` — `sized(0.4, 0.3).eyeHeight(0.255)`.
pub fn silverfish_entity_type() -> EntityType {
    misc_entity_type("silverfish", 0.4, 0.3, 0.255, 8, 5)
}
