//! Перенос `net.minecraft.tags.DamageTypeTags` — все теги типов урона
//! с ванильным содержимым из `data/minecraft/tags/damage_type/*.json`.

use crate::core::registry::registries;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use crate::tags::tag_key::TagKey;
use crate::world::damagesource::damage_type::DamageType;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Конструкторы `TagKey` для каждого тега (`DamageTypeTags.*`).
pub mod keys {
    use super::*;
    /// `DAMAGES_HELMET`.
    pub fn damages_helmet() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("damages_helmet").unwrap())
    }
    /// `BYPASSES_ARMOR`.
    pub fn bypasses_armor() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("bypasses_armor").unwrap())
    }
    /// `BYPASSES_SHIELD`.
    pub fn bypasses_shield() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("bypasses_shield").unwrap())
    }
    /// `BYPASSES_INVULNERABILITY`.
    pub fn bypasses_invulnerability() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("bypasses_invulnerability").unwrap())
    }
    /// `BYPASSES_COOLDOWN`.
    pub fn bypasses_cooldown() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("bypasses_cooldown").unwrap())
    }
    /// `BYPASSES_EFFECTS`.
    pub fn bypasses_effects() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("bypasses_effects").unwrap())
    }
    /// `BYPASSES_RESISTANCE`.
    pub fn bypasses_resistance() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("bypasses_resistance").unwrap())
    }
    /// `BYPASSES_ENCHANTMENTS`.
    pub fn bypasses_enchantments() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("bypasses_enchantments").unwrap())
    }
    /// `IS_FIRE`.
    pub fn is_fire() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("is_fire").unwrap())
    }
    /// `IS_PROJECTILE`.
    pub fn is_projectile() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("is_projectile").unwrap())
    }
    /// `WITCH_RESISTANT_TO`.
    pub fn witch_resistant_to() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("witch_resistant_to").unwrap())
    }
    /// `IS_EXPLOSION`.
    pub fn is_explosion() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("is_explosion").unwrap())
    }
    /// `IS_FALL`.
    pub fn is_fall() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("is_fall").unwrap())
    }
    /// `IS_DROWNING`.
    pub fn is_drowning() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("is_drowning").unwrap())
    }
    /// `IS_FREEZING`.
    pub fn is_freezing() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("is_freezing").unwrap())
    }
    /// `IS_LIGHTNING`.
    pub fn is_lightning() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("is_lightning").unwrap())
    }
    /// `NO_ANGER`.
    pub fn no_anger() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("no_anger").unwrap())
    }
    /// `NO_IMPACT`.
    pub fn no_impact() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("no_impact").unwrap())
    }
    /// `ALWAYS_MOST_SIGNIFICANT_FALL`.
    pub fn always_most_significant_fall() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("always_most_significant_fall").unwrap())
    }
    /// `WITHER_IMMUNE_TO`.
    pub fn wither_immune_to() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("wither_immune_to").unwrap())
    }
    /// `IGNITES_ARMOR_STANDS`.
    pub fn ignites_armor_stands() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("ignites_armor_stands").unwrap())
    }
    /// `BURNS_ARMOR_STANDS`.
    pub fn burns_armor_stands() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("burns_armor_stands").unwrap())
    }
    /// `AVOIDS_GUARDIAN_THORNS`.
    pub fn avoids_guardian_thorns() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("avoids_guardian_thorns").unwrap())
    }
    /// `ALWAYS_TRIGGERS_SILVERFISH`.
    pub fn always_triggers_silverfish() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("always_triggers_silverfish").unwrap())
    }
    /// `ALWAYS_HURTS_ENDER_DRAGONS`.
    pub fn always_hurts_ender_dragons() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("always_hurts_ender_dragons").unwrap())
    }
    /// `NO_KNOCKBACK`.
    pub fn no_knockback() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("no_knockback").unwrap())
    }
    /// `ALWAYS_KILLS_ARMOR_STANDS`.
    pub fn always_kills_armor_stands() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("always_kills_armor_stands").unwrap())
    }
    /// `CAN_BREAK_ARMOR_STAND`.
    pub fn can_break_armor_stand() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("can_break_armor_stand").unwrap())
    }
    /// `BYPASSES_WOLF_ARMOR`.
    pub fn bypasses_wolf_armor() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("bypasses_wolf_armor").unwrap())
    }
    /// `IS_PLAYER_ATTACK`.
    pub fn is_player_attack() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("is_player_attack").unwrap())
    }
    /// `BURN_FROM_STEPPING`.
    pub fn burn_from_stepping() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("burn_from_stepping").unwrap())
    }
    /// `PANIC_CAUSES`.
    pub fn panic_causes() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("panic_causes").unwrap())
    }
    /// `PANIC_ENVIRONMENTAL_CAUSES`.
    pub fn panic_environmental_causes() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("panic_environmental_causes").unwrap())
    }
    /// `IS_MACE_SMASH`.
    pub fn is_mace_smash() -> TagKey<DamageType> {
        TagKey::create(ResourceKey::create_registry_key(registries::damage_type()), Identifier::with_default_namespace("is_mace_smash").unwrap())
    }
}

use keys::*;

/// Значение тега: конкретный тип или ссылка на другой тег (`#...`).
#[derive(Clone, Debug)]
pub enum TagValue {
    Direct(Identifier),
    Nested(TagKey<DamageType>),
}

fn tag_registry() -> &'static Mutex<HashMap<Identifier, Vec<TagValue>>> {
    static REG: OnceLock<Mutex<HashMap<Identifier, Vec<TagValue>>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Регистрация содержимого тега (аналог TagLoader).
pub fn register_tag(location: Identifier, values: Vec<TagValue>) {
    tag_registry().lock().unwrap().insert(location, values);
}

fn tag_key_id(key: &TagKey<DamageType>) -> Identifier {
    key.location().clone()
}

/// `Holder.is(tag)` / `source.is(DamageTypeTags.X)` — членство типа в теге.
pub fn is_in_tag(damage_type_id: &Identifier, tag: &TagKey<DamageType>) -> bool {
    is_in_tag_inner(damage_type_id, &tag_key_id(tag), 0)
}

/// Проверка членства произвольного registry-id в теге по его location.
/// Используется для item/entity/других тегов, разделяющих тот же
/// формат (id -> [members | #nested]).
pub fn is_member_of_tag(tag_location: &Identifier, value_id: &Identifier) -> bool {
    is_in_tag_inner(value_id, tag_location, 0)
}

fn is_in_tag_inner(damage_type_id: &Identifier, tag_location: &Identifier, depth: u32) -> bool {
    if depth > 16 {
        return false;
    }
    let (direct, nested) = {
        let reg = tag_registry().lock().unwrap();
        let Some(values) = reg.get(tag_location) else { return false; };
        let mut direct = Vec::new();
        let mut nested = Vec::new();
        for v in values {
            match v {
                TagValue::Direct(id) => direct.push(id.clone()),
                TagValue::Nested(n) => nested.push(tag_key_id(n)),
            }
        }
        (direct, nested)
    };
    if direct.iter().any(|id| id == damage_type_id) {
        return true;
    }
    nested
        .iter()
        .any(|nid| is_in_tag_inner(damage_type_id, nid, depth + 1))
}

/// Ванильное содержимое всех тегов (из jar `data/minecraft/tags/damage_type/*.json`).
/// Вызывать один раз при bootstrap реестров.
pub fn bootstrap() {
    let mut reg = tag_registry().lock().unwrap();
    reg.insert(Identifier::with_default_namespace("damages_helmet").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("falling_anvil").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("falling_block").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("falling_stalactite").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("bypasses_armor").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("on_fire").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("in_wall").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("cramming").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("drown").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("fly_into_wall").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("generic").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("wither").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("dragon_breath").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("starve").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("fall").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("ender_pearl").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("freeze").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("stalagmite").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("magic").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("indirect_magic").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("out_of_world").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("generic_kill").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("sonic_boom").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("outside_border").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("bypasses_shield").unwrap(), vec![
        TagValue::Nested(bypasses_armor()),
        TagValue::Direct(Identifier::with_default_namespace("cactus").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("campfire").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("dry_out").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("falling_anvil").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("falling_stalactite").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("hot_floor").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("in_fire").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("lava").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("lightning_bolt").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("sweet_berry_bush").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("bypasses_invulnerability").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("out_of_world").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("generic_kill").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("bypasses_cooldown").unwrap(), vec![
    ]);
    reg.insert(Identifier::with_default_namespace("bypasses_effects").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("starve").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("bypasses_resistance").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("out_of_world").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("generic_kill").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("bypasses_enchantments").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("sonic_boom").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("is_fire").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("in_fire").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("campfire").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("on_fire").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("lava").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("hot_floor").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("unattributed_fireball").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("fireball").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("is_projectile").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("arrow").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("trident").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("mob_projectile").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("unattributed_fireball").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("fireball").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("wither_skull").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("thrown").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("wind_charge").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("witch_resistant_to").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("magic").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("indirect_magic").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("sonic_boom").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("thorns").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("is_explosion").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("fireworks").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("explosion").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("player_explosion").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("bad_respawn_point").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("is_fall").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("fall").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("ender_pearl").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("stalagmite").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("is_drowning").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("drown").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("is_freezing").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("freeze").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("is_lightning").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("lightning_bolt").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("no_anger").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("mob_attack_no_aggro").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("no_impact").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("drown").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("always_most_significant_fall").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("out_of_world").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("wither_immune_to").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("drown").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("ignites_armor_stands").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("in_fire").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("campfire").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("burns_armor_stands").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("on_fire").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("avoids_guardian_thorns").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("magic").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("thorns").unwrap()),
        TagValue::Nested(is_explosion()),
    ]);
    reg.insert(Identifier::with_default_namespace("always_triggers_silverfish").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("magic").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("always_hurts_ender_dragons").unwrap(), vec![
        TagValue::Nested(is_explosion()),
    ]);
    reg.insert(Identifier::with_default_namespace("no_knockback").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("explosion").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("player_explosion").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("bad_respawn_point").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("in_fire").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("lightning_bolt").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("on_fire").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("lava").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("hot_floor").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("in_wall").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("cramming").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("drown").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("starve").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("cactus").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("fall").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("ender_pearl").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("fly_into_wall").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("out_of_world").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("generic").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("magic").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("wither").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("dragon_breath").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("dry_out").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("sweet_berry_bush").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("freeze").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("stalagmite").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("outside_border").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("generic_kill").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("campfire").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("spear").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("always_kills_armor_stands").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("arrow").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("trident").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("fireball").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("wither_skull").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("wind_charge").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("can_break_armor_stand").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("player_explosion").unwrap()),
        TagValue::Nested(is_player_attack()),
    ]);
    reg.insert(Identifier::with_default_namespace("bypasses_wolf_armor").unwrap(), vec![
        TagValue::Nested(bypasses_invulnerability()),
        TagValue::Direct(Identifier::with_default_namespace("cramming").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("drown").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("dry_out").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("freeze").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("in_wall").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("indirect_magic").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("magic").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("outside_border").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("starve").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("thorns").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("wither").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("is_player_attack").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("player_attack").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("spear").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("mace_smash").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("burn_from_stepping").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("campfire").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("hot_floor").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("panic_causes").unwrap(), vec![
        TagValue::Nested(panic_environmental_causes()),
        TagValue::Direct(Identifier::with_default_namespace("arrow").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("dragon_breath").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("explosion").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("fireball").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("fireworks").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("indirect_magic").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("magic").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("mob_attack").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("mob_projectile").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("player_explosion").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("sonic_boom").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("sting").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("thrown").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("trident").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("unattributed_fireball").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("wind_charge").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("wither").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("wither_skull").unwrap()),
        TagValue::Nested(is_player_attack()),
    ]);
    reg.insert(Identifier::with_default_namespace("panic_environmental_causes").unwrap(), vec![
        TagValue::Direct(Identifier::with_default_namespace("cactus").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("freeze").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("hot_floor").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("in_fire").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("lava").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("lightning_bolt").unwrap()),
        TagValue::Direct(Identifier::with_default_namespace("on_fire").unwrap()),
    ]);
    reg.insert(Identifier::with_default_namespace("is_mace_smash").unwrap(), vec![
    ]);
}

/// Удобные bool-проверки (`source.is(DamageTypeTags.X)`).
pub mod checks {
    use super::*;
    /// `DamageTypeTags.DAMAGES_HELMET`.
    pub fn damages_helmet(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::damages_helmet())
    }
    /// `DamageTypeTags.BYPASSES_ARMOR`.
    pub fn bypasses_armor(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::bypasses_armor())
    }
    /// `DamageTypeTags.BYPASSES_SHIELD`.
    pub fn bypasses_shield(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::bypasses_shield())
    }
    /// `DamageTypeTags.BYPASSES_INVULNERABILITY`.
    pub fn bypasses_invulnerability(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::bypasses_invulnerability())
    }
    /// `DamageTypeTags.BYPASSES_COOLDOWN`.
    pub fn bypasses_cooldown(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::bypasses_cooldown())
    }
    /// `DamageTypeTags.BYPASSES_EFFECTS`.
    pub fn bypasses_effects(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::bypasses_effects())
    }
    /// `DamageTypeTags.BYPASSES_RESISTANCE`.
    pub fn bypasses_resistance(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::bypasses_resistance())
    }
    /// `DamageTypeTags.BYPASSES_ENCHANTMENTS`.
    pub fn bypasses_enchantments(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::bypasses_enchantments())
    }
    /// `DamageTypeTags.IS_FIRE`.
    pub fn is_fire(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::is_fire())
    }
    /// `DamageTypeTags.IS_PROJECTILE`.
    pub fn is_projectile(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::is_projectile())
    }
    /// `DamageTypeTags.WITCH_RESISTANT_TO`.
    pub fn witch_resistant_to(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::witch_resistant_to())
    }
    /// `DamageTypeTags.IS_EXPLOSION`.
    pub fn is_explosion(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::is_explosion())
    }
    /// `DamageTypeTags.IS_FALL`.
    pub fn is_fall(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::is_fall())
    }
    /// `DamageTypeTags.IS_DROWNING`.
    pub fn is_drowning(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::is_drowning())
    }
    /// `DamageTypeTags.IS_FREEZING`.
    pub fn is_freezing(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::is_freezing())
    }
    /// `DamageTypeTags.IS_LIGHTNING`.
    pub fn is_lightning(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::is_lightning())
    }
    /// `DamageTypeTags.NO_ANGER`.
    pub fn no_anger(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::no_anger())
    }
    /// `DamageTypeTags.NO_IMPACT`.
    pub fn no_impact(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::no_impact())
    }
    /// `DamageTypeTags.ALWAYS_MOST_SIGNIFICANT_FALL`.
    pub fn always_most_significant_fall(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::always_most_significant_fall())
    }
    /// `DamageTypeTags.WITHER_IMMUNE_TO`.
    pub fn wither_immune_to(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::wither_immune_to())
    }
    /// `DamageTypeTags.IGNITES_ARMOR_STANDS`.
    pub fn ignites_armor_stands(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::ignites_armor_stands())
    }
    /// `DamageTypeTags.BURNS_ARMOR_STANDS`.
    pub fn burns_armor_stands(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::burns_armor_stands())
    }
    /// `DamageTypeTags.AVOIDS_GUARDIAN_THORNS`.
    pub fn avoids_guardian_thorns(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::avoids_guardian_thorns())
    }
    /// `DamageTypeTags.ALWAYS_TRIGGERS_SILVERFISH`.
    pub fn always_triggers_silverfish(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::always_triggers_silverfish())
    }
    /// `DamageTypeTags.ALWAYS_HURTS_ENDER_DRAGONS`.
    pub fn always_hurts_ender_dragons(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::always_hurts_ender_dragons())
    }
    /// `DamageTypeTags.NO_KNOCKBACK`.
    pub fn no_knockback(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::no_knockback())
    }
    /// `DamageTypeTags.ALWAYS_KILLS_ARMOR_STANDS`.
    pub fn always_kills_armor_stands(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::always_kills_armor_stands())
    }
    /// `DamageTypeTags.CAN_BREAK_ARMOR_STAND`.
    pub fn can_break_armor_stand(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::can_break_armor_stand())
    }
    /// `DamageTypeTags.BYPASSES_WOLF_ARMOR`.
    pub fn bypasses_wolf_armor(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::bypasses_wolf_armor())
    }
    /// `DamageTypeTags.IS_PLAYER_ATTACK`.
    pub fn is_player_attack(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::is_player_attack())
    }
    /// `DamageTypeTags.BURN_FROM_STEPPING`.
    pub fn burn_from_stepping(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::burn_from_stepping())
    }
    /// `DamageTypeTags.PANIC_CAUSES`.
    pub fn panic_causes(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::panic_causes())
    }
    /// `DamageTypeTags.PANIC_ENVIRONMENTAL_CAUSES`.
    pub fn panic_environmental_causes(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::panic_environmental_causes())
    }
    /// `DamageTypeTags.IS_MACE_SMASH`.
    pub fn is_mace_smash(damage_type_id: &Identifier) -> bool {
        is_in_tag(damage_type_id, &keys::is_mace_smash())
    }
}
