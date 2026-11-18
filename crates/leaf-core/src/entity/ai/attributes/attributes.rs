//! Перенос `net.minecraft.world.entity.ai.attributes.Attributes`.

use super::attribute::Sentiment;
use super::ranged_attribute::RangedAttribute;
use crate::resources::identifier::Identifier;
use std::collections::HashMap;
use std::sync::OnceLock;

fn ranged(
    name: &str,
    desc: &str,
    default: f64,
    min: f64,
    max: f64,
    syncable: bool,
    sentiment: Sentiment,
) -> RangedAttribute {
    let mut a = RangedAttribute::new(
        Identifier::minecraft(name),
        desc,
        default,
        min,
        max,
    );
    a.base.syncable = syncable;
    a.base.sentiment = sentiment;
    a
}

fn plain(name: &str, desc: &str, default: f64) -> RangedAttribute {
    ranged(name, desc, default, f64::NEG_INFINITY, f64::INFINITY, false, Sentiment::Positive)
}

/// Все атрибуты 1.21.11 (`Attributes` константы) — ключ = registry id.
pub fn all_attributes() -> &'static HashMap<Identifier, RangedAttribute> {
    static MAP: OnceLock<HashMap<Identifier, RangedAttribute>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        let mut insert = |a: RangedAttribute| {
            m.insert(a.base.id.clone(), a);
        };

        insert(ranged("armor", "attribute.name.armor", 0.0, 0.0, 30.0, true, Sentiment::Positive));
        insert(ranged("armor_toughness", "attribute.name.armor_toughness", 0.0, 0.0, 20.0, true, Sentiment::Positive));
        insert(ranged("attack_damage", "attribute.name.attack_damage", 2.0, 0.0, 2048.0, false, Sentiment::Positive)); // Spigot max
        insert(ranged("attack_knockback", "attribute.name.attack_knockback", 0.0, 0.0, 5.0, false, Sentiment::Positive));
        insert(ranged("attack_speed", "attribute.name.attack_speed", 4.0, 0.0, 1024.0, true, Sentiment::Positive));
        insert(ranged("block_break_speed", "attribute.name.block_break_speed", 1.0, 0.0, 1024.0, true, Sentiment::Positive));
        insert(ranged("block_interaction_range", "attribute.name.block_interaction_range", 4.5, 0.0, 64.0, true, Sentiment::Positive));
        insert(ranged("burning_time", "attribute.name.burning_time", 1.0, 0.0, 1024.0, true, Sentiment::Negative));
        insert(ranged("camera_distance", "attribute.name.camera_distance", 4.0, 0.0, 32.0, true, Sentiment::Positive));
        insert(ranged("explosion_knockback_resistance", "attribute.name.explosion_knockback_resistance", 0.0, 0.0, 1.0, true, Sentiment::Positive));
        insert(ranged("entity_interaction_range", "attribute.name.entity_interaction_range", 3.0, 0.0, 64.0, true, Sentiment::Positive));
        insert(ranged("fall_damage_multiplier", "attribute.name.fall_damage_multiplier", 1.0, 0.0, 100.0, true, Sentiment::Negative));
        insert(ranged("flying_speed", "attribute.name.flying_speed", 0.4, 0.0, 1024.0, true, Sentiment::Positive));
        insert(ranged("follow_range", "attribute.name.follow_range", 32.0, 0.0, 2048.0, false, Sentiment::Positive));
        insert(ranged("gravity", "attribute.name.gravity", 0.08, -1.0, 1.0, true, Sentiment::Neutral));
        insert(ranged("jump_strength", "attribute.name.jump_strength", 0.42, 0.0, 32.0, true, Sentiment::Positive));
        insert(ranged("knockback_resistance", "attribute.name.knockback_resistance", 0.0, 0.0, 1.0, false, Sentiment::Positive));
        insert(ranged("luck", "attribute.name.luck", 0.0, -1024.0, 1024.0, true, Sentiment::Positive));
        insert(ranged("max_absorption", "attribute.name.max_absorption", 0.0, 0.0, 2048.0, true, Sentiment::Positive)); // Spigot max
        insert(ranged("max_health", "attribute.name.max_health", 20.0, 1.0, 2048.0, true, Sentiment::Positive)); // Spigot max
        insert(ranged("mining_efficiency", "attribute.name.mining_efficiency", 0.0, 0.0, 1024.0, true, Sentiment::Positive));
        insert(ranged("movement_efficiency", "attribute.name.movement_efficiency", 0.0, 0.0, 1.0, true, Sentiment::Positive));
        insert(ranged("movement_speed", "attribute.name.movement_speed", 0.7, 0.0, 1024.0, true, Sentiment::Positive)); // Spigot max
        insert(ranged("oxygen_bonus", "attribute.name.oxygen_bonus", 0.0, 0.0, 1024.0, true, Sentiment::Positive));
        insert(ranged("safe_fall_distance", "attribute.name.safe_fall_distance", 3.0, -1024.0, 1024.0, true, Sentiment::Positive));
        insert(ranged("scale", "attribute.name.scale", 1.0, 0.0625, 16.0, true, Sentiment::Neutral));
        insert(ranged("sneaking_speed", "attribute.name.sneaking_speed", 0.3, 0.0, 1.0, true, Sentiment::Positive));
        insert(ranged("spawn_reinforcements", "attribute.name.spawn_reinforcements", 0.0, 0.0, 1.0, false, Sentiment::Positive));
        insert(ranged("step_height", "attribute.name.step_height", 0.6, 0.0, 10.0, true, Sentiment::Positive));
        insert(ranged("submerged_mining_speed", "attribute.name.submerged_mining_speed", 0.2, 0.0, 20.0, true, Sentiment::Positive));
        insert(ranged("sweeping_damage_ratio", "attribute.name.sweeping_damage_ratio", 0.0, 0.0, 1.0, true, Sentiment::Positive));
        insert(ranged("tempt_range", "attribute.name.tempt_range", 10.0, 0.0, 2048.0, false, Sentiment::Positive));
        insert(ranged("water_movement_efficiency", "attribute.name.water_movement_efficiency", 0.0, 0.0, 1.0, true, Sentiment::Positive));
        insert(ranged("waypoint_transmit_range", "attribute.name.waypoint_transmit_range", 0.0, 0.0, 6.0e7, false, Sentiment::Neutral));
        insert(ranged("waypoint_receive_range", "attribute.name.waypoint_receive_range", 0.0, 0.0, 6.0e7, false, Sentiment::Neutral));
        m
    })
}

/// `Attributes.get(id)` — получить атрибут по registry id.
pub fn get(id: &Identifier) -> Option<&'static RangedAttribute> {
    all_attributes().get(id)
}

// --- Константы-ключи (Holder<Attribute> в Java → Identifier здесь) ---

macro_rules! attr_id {
    ($name:ident, $key:literal) => {
        pub fn $name() -> Identifier {
            Identifier::minecraft($key)
        }
    };
}

attr_id!(armor, "armor");
attr_id!(armor_toughness, "armor_toughness");
attr_id!(attack_damage, "attack_damage");
attr_id!(attack_knockback, "attack_knockback");
attr_id!(attack_speed, "attack_speed");
attr_id!(block_break_speed, "block_break_speed");
attr_id!(block_interaction_range, "block_interaction_range");
attr_id!(burning_time, "burning_time");
attr_id!(camera_distance, "camera_distance");
attr_id!(explosion_knockback_resistance, "explosion_knockback_resistance");
attr_id!(entity_interaction_range, "entity_interaction_range");
attr_id!(fall_damage_multiplier, "fall_damage_multiplier");
attr_id!(flying_speed, "flying_speed");
attr_id!(follow_range, "follow_range");
attr_id!(gravity, "gravity");
attr_id!(jump_strength, "jump_strength");
attr_id!(knockback_resistance, "knockback_resistance");
attr_id!(luck, "luck");
attr_id!(max_absorption, "max_absorption");
attr_id!(max_health, "max_health");
attr_id!(mining_efficiency, "mining_efficiency");
attr_id!(movement_efficiency, "movement_efficiency");
attr_id!(movement_speed, "movement_speed");
attr_id!(oxygen_bonus, "oxygen_bonus");
attr_id!(safe_fall_distance, "safe_fall_distance");
attr_id!(scale, "scale");
attr_id!(sneaking_speed, "sneaking_speed");
attr_id!(spawn_reinforcements, "spawn_reinforcements");
attr_id!(step_height, "step_height");
attr_id!(submerged_mining_speed, "submerged_mining_speed");
attr_id!(sweeping_damage_ratio, "sweeping_damage_ratio");
attr_id!(tempt_range, "tempt_range");
attr_id!(water_movement_efficiency, "water_movement_efficiency");
attr_id!(waypoint_transmit_range, "waypoint_transmit_range");
attr_id!(waypoint_receive_range, "waypoint_receive_range");

/// Ключи в порядке реестра.
pub fn all_keys() -> Vec<Identifier> {
    all_attributes().keys().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count() {
        // 33 атрибута в 1.21.11
        assert_eq!(all_attributes().len(), 35);
    }

    #[test]
    fn test_max_health() {
        let a = get(&Identifier::minecraft("max_health")).unwrap();
        assert_eq!(a.get_default_value(), 20.0);
        assert!(a.is_client_syncable());
        assert_eq!(a.sanitize_value(5000.0), 2048.0); // Spigot max
        assert_eq!(a.sanitize_value(0.0), 1.0); // min
    }
}
