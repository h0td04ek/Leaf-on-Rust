//! Перенос `net.minecraft.tags.EntityTypeTags` — теги типов сущностей
//! с ванильным содержимым из `data/minecraft/tags/entity_type/*.json`
//! (включая раскрытые вложенные теги `#undead`/`#skeletons`/`#zombies`).

use crate::core::registry::registries;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use crate::tags::tag_key::TagKey;
use crate::world::entity::entity_type::EntityType;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Глобальный реестр тегов типов сущностей:
/// `tag location -> set<entity type id>` (вложенные теги уже
/// раскрыты в плоские списки при регистрации).
fn entity_type_tag_registry() -> &'static Mutex<HashMap<Identifier, Vec<Identifier>>> {
    static REG: OnceLock<Mutex<HashMap<Identifier, Vec<Identifier>>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Зарегистрировать тег типов сущностей (аналог результата
/// `TagLoader` для `entity_type`).
pub fn register_entity_type_tag(location: Identifier, members: Vec<Identifier>) {
    entity_type_tag_registry().lock().unwrap().insert(location, members);
}

/// `EntityType.is(TagKey)` — членство типа сущности в теге
/// по registry-идентификатору.
pub fn is_entity_type_in_tag(entity_type: &EntityType, tag: &TagKey<EntityType>) -> bool {
    let reg = entity_type_tag_registry().lock().unwrap();
    reg.get(&tag.location)
        .map(|members| members.iter().any(|m| m == &entity_type.id))
        .unwrap_or(false)
}

/// `EntityTypeTags` — конструкторы `TagKey<EntityType>`.
pub mod keys {
    use super::*;
    fn tag(name: &str) -> TagKey<EntityType> {
        TagKey::create(
            ResourceKey::create_registry_key(registries::entity_type()),
            Identifier::with_default_namespace(name).unwrap(),
        )
    }
    /// `SKELETONS`.
    pub fn skeletons() -> TagKey<EntityType> { tag("skeletons") }
    /// `ZOMBIES`.
    pub fn zombies() -> TagKey<EntityType> { tag("zombies") }
    /// `UNDEAD`.
    pub fn undead() -> TagKey<EntityType> { tag("undead") }
    /// `INVERTED_HEALING_AND_HARM`.
    pub fn inverted_healing_and_harm() -> TagKey<EntityType> { tag("inverted_healing_and_harm") }
    /// `IGNORES_POISON_AND_REGEN`.
    pub fn ignores_poison_and_regen() -> TagKey<EntityType> { tag("ignores_poison_and_regen") }
    /// `IMMUNE_TO_INFESTED`.
    pub fn immune_to_infested() -> TagKey<EntityType> { tag("immune_to_infested") }
    /// `IMMUNE_TO_OOZING`.
    pub fn immune_to_oozing() -> TagKey<EntityType> { tag("immune_to_oozing") }
    /// `FREEZE_HURTS_EXTRA_TYPES`.
    pub fn freeze_hurts_extra_types() -> TagKey<EntityType> { tag("freeze_hurts_extra_types") }
    /// `ARTHROPOD`.
    pub fn arthropod() -> TagKey<EntityType> { tag("arthropod") }
    /// `ILLAGER`.
    pub fn illager() -> TagKey<EntityType> { tag("illager") }
    /// `RAIDERS`.
    pub fn raiders() -> TagKey<EntityType> { tag("raiders") }
    /// `POWDER_SNOW_WALKABLE_MOBS`.
    pub fn powder_snow_walkable_mobs() -> TagKey<EntityType> {
        tag("powder_snow_walkable_mobs")
    }

    /// `EntityTypeTags.CANDIDATE_FOR_IRON_GOLEM_GIFT`.
    pub fn candidate_for_iron_golem_gift() -> TagKey<EntityType> {
        tag("candidate_for_iron_golem_gift")
    }

    /// `EntityTypeTags.ACCEPTS_IRON_GOLEM_GIFT`.
    pub fn accepts_iron_golem_gift() -> TagKey<EntityType> {
        tag("accepts_iron_golem_gift")
    }

    /// `EntityTypeTags.AXOLOTL_HUNT_TARGETS`.
    pub fn axolotl_hunt_targets() -> TagKey<EntityType> { tag("axolotl_hunt_targets") }
    /// `EntityTypeTags.AXOLOTL_ALWAYS_HOSTILES`.
    pub fn axolotl_always_hostiles() -> TagKey<EntityType> { tag("axolotl_always_hostiles") }
    /// `EntityTypeTags.FOLLOWABLE_FRIENDLY_MOBS`.
    pub fn followable_friendly_mobs() -> TagKey<EntityType> { tag("followable_friendly_mobs") }
}

/// `bootstrap()` — ванильное содержимое используемых тегов
/// (плоские списки; вложенные `#`-ссылки раскрыты вручную).
pub fn bootstrap() {
    use crate::resources::identifier::Identifier as Id;
    let ids = |names: &[&str]| {
        names.iter().map(|n| Id::with_default_namespace(n).unwrap()).collect::<Vec<_>>()
    };
    // skeletons
    register_entity_type_tag(
        Id::with_default_namespace("skeletons").unwrap(),
        ids(&["skeleton", "stray", "wither_skeleton", "skeleton_horse", "bogged", "parched"]),
    );
    // zombies
    register_entity_type_tag(
        Id::with_default_namespace("zombies").unwrap(),
        ids(&[
            "zombie_horse", "camel_husk", "zombie", "zombie_villager",
            "zombified_piglin", "zoglin", "drowned", "husk", "zombie_nautilus",
        ]),
    );
    // undead = skeletons + zombies + wither + phantom
    let mut undead = ids(&[
        "skeleton", "stray", "wither_skeleton", "skeleton_horse", "bogged", "parched",
        "zombie_horse", "camel_husk", "zombie", "zombie_villager",
        "zombified_piglin", "zoglin", "drowned", "husk", "zombie_nautilus",
        "wither", "phantom",
    ]);
    undead.sort();
    undead.dedup();
    register_entity_type_tag(Id::with_default_namespace("undead").unwrap(), undead);
    // inverted_healing_and_harm = #undead
    register_entity_type_tag(
        Id::with_default_namespace("inverted_healing_and_harm").unwrap(),
        entity_type_tag_registry()
            .lock()
            .unwrap()
            .get(&Id::with_default_namespace("undead").unwrap())
            .cloned()
            .unwrap_or_default(),
    );
    // ignores_poison_and_regen = #undead
    register_entity_type_tag(
        Id::with_default_namespace("ignores_poison_and_regen").unwrap(),
        entity_type_tag_registry()
            .lock()
            .unwrap()
            .get(&Id::with_default_namespace("undead").unwrap())
            .cloned()
            .unwrap_or_default(),
    );
    // immune_to_infested = [silverfish]
    register_entity_type_tag(
        Id::with_default_namespace("immune_to_infested").unwrap(),
        ids(&["silverfish"]),
    );
    // immune_to_oozing = [slime]
    register_entity_type_tag(
        Id::with_default_namespace("immune_to_oozing").unwrap(),
        ids(&["slime"]),
    );
    // freeze_hurts_extra_types = [strider, blaze, magma_cube]
    register_entity_type_tag(
        Id::with_default_namespace("freeze_hurts_extra_types").unwrap(),
        ids(&["strider", "blaze", "magma_cube"]),
    );
    // powder_snow_walkable_mobs = [polar_bear, strider]
    register_entity_type_tag(
        Id::with_default_namespace("powder_snow_walkable_mobs").unwrap(),
        ids(&["polar_bear", "strider"]),
    );
    // candidate_for_iron_golem_gift = [villager, wandering_trader]
    register_entity_type_tag(
        Id::with_default_namespace("candidate_for_iron_golem_gift").unwrap(),
        ids(&["villager", "wandering_trader"]),
    );
    // accepts_iron_golem_gift = [copper_golem]
    register_entity_type_tag(
        Id::with_default_namespace("accepts_iron_golem_gift").unwrap(),
        ids(&["copper_golem"]),
    );
    // axolotl_hunt_targets
    register_entity_type_tag(
        Id::with_default_namespace("axolotl_hunt_targets").unwrap(),
        ids(&[
            "tropical_fish", "pufferfish", "salmon", "cod", "squid",
            "glow_squid", "tadpole",
        ]),
    );
    // axolotl_always_hostiles
    register_entity_type_tag(
        Id::with_default_namespace("axolotl_always_hostiles").unwrap(),
        ids(&["drowned", "guardian", "elder_guardian"]),
    );
    // followable_friendly_mobs
    register_entity_type_tag(
        Id::with_default_namespace("followable_friendly_mobs").unwrap(),
        ids(&[
            "armadillo", "bee", "camel", "cat", "chicken", "cow", "donkey",
            "fox", "goat", "happy_ghast", "horse", "skeleton_horse", "llama",
            "mule", "ocelot", "panda", "parrot", "pig", "polar_bear",
            "rabbit", "sheep", "sniffer", "strider", "villager", "wolf",
        ]),
    );
}

/// Удобный хелпер: `entityType.is(EntityTypeTags.X)` по id типа.
pub fn entity_type_is(
    entity_type: &EntityType,
    tag: TagKey<EntityType>,
) -> bool {
    is_entity_type_in_tag(entity_type, &tag)
}

/// `bootstrap_once()` — идемпотентный вызов bootstrap.
pub fn bootstrap_once() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(bootstrap);
}
