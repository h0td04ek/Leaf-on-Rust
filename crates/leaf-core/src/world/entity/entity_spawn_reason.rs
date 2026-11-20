//! Перенос `net.minecraft.world.entity.EntitySpawnReason` —
//! причина спавна сущности.

/// `EntitySpawnReason` — причина появления сущности в мире.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntitySpawnReason {
    /// NATURAL — естественный спавн.
    Natural,
    /// CHUNK_GENERATION — спавн при генерации чанка.
    ChunkGeneration,
    /// SPAWNER — mob spawner / trial spawner блок.
    Spawner,
    /// STRUCTURE — структура.
    Structure,
    /// BREEDING — разведение.
    Breeding,
    /// MOB_SUMMONED — призыв мобом (зомби-лошадь и т.п.).
    MobSummoned,
    /// JOCKEY — наездник.
    Jockey,
    /// EVENT — событие (зелье, призыв).
    Event,
    /// CONVERSION — конверсия (зомбификация и т.п.).
    Conversion,
    /// REINFORCEMENT — подкрепление (зомби-звон призывает).
    Reinforcement,
    /// TRIGGERED — триггер (крипер-спавн).
    Triggered,
    /// BUCKET — ведро.
    Bucket,
    /// SPAWN_ITEM_USE — спавн-яйцо.
    SpawnItemUse,
    /// COMMAND — команда.
    Command,
    /// DISPENSER — раздатчик.
    Dispenser,
    /// PATROL — патруль разбойников.
    Patrol,
    /// TRIAL_SPAWNER — trial spawner.
    TrialSpawner,
    /// LOAD — загрузка из NBT.
    Load,
    /// DIMENSION_TRAVEL — переход между измерениями.
    DimensionTravel,
}

impl EntitySpawnReason {
    /// `EntitySpawnReason.isSpawner(reason)` — SPAWNER или
    /// TRIAL_SPAWNER.
    pub fn is_spawner(reason: EntitySpawnReason) -> bool {
        matches!(reason, EntitySpawnReason::Spawner | EntitySpawnReason::TrialSpawner)
    }

    /// `EntitySpawnReason.ignoresLightRequirements(reason)` —
    /// только TRIAL_SPAWNER игнорирует требования освещения.
    pub fn ignores_light_requirements(reason: EntitySpawnReason) -> bool {
        matches!(reason, EntitySpawnReason::TrialSpawner)
    }
}
