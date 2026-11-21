//! Перенос `net.minecraft.world.level.dimension.LevelStem` —
//! ствол измерения (тип + генератор). Пока — только ключи,
//! используемые `Registries.levelToLevelStem`.

use crate::resources::resource_key::ResourceKey;
use crate::resources::Identifier;
use std::sync::LazyLock;

/// `LevelStem` — маркер типа для `ResourceKey<LevelStem>`.
#[derive(Debug)]
pub struct LevelStem;

/// `LevelStem.OVERWORLD` — `minecraft:overworld` в реестре `dimension`.
pub static OVERWORLD: LazyLock<ResourceKey<LevelStem>> = LazyLock::new(|| {
    ResourceKey::create_with_registry_name(
        Identifier::minecraft("dimension"),
        Identifier::minecraft("overworld"),
    )
});

/// `LevelStem.NETHER` — `minecraft:the_nether`.
pub static NETHER: LazyLock<ResourceKey<LevelStem>> = LazyLock::new(|| {
    ResourceKey::create_with_registry_name(
        Identifier::minecraft("dimension"),
        Identifier::minecraft("the_nether"),
    )
});

/// `LevelStem.END` — `minecraft:the_end`.
pub static END: LazyLock<ResourceKey<LevelStem>> = LazyLock::new(|| {
    ResourceKey::create_with_registry_name(
        Identifier::minecraft("dimension"),
        Identifier::minecraft("the_end"),
    )
});

/// `Registries.LEVEL_STEM` — ключ реестра `minecraft:dimension`.
pub static LEVEL_STEM_REGISTRY: LazyLock<ResourceKey<crate::core::registry::Registry<LevelStem>>> =
    LazyLock::new(|| {
        ResourceKey::create_registry_key(Identifier::minecraft("dimension"))
    });

/// `Registries.levelToLevelStem(level)` — ключ измерения → ключ ствола.
/// В Java: `ResourceKey.create(LEVEL_STEM, level.identifier())`.
pub fn level_to_level_stem(
    level: &ResourceKey<crate::world::level::LevelKey>,
) -> ResourceKey<LevelStem> {
    ResourceKey::create(&LEVEL_STEM_REGISTRY, level.id().clone())
}
