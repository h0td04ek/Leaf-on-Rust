<div align="center">

# 🍃 Leaf on Rust

**Полный порт сервера Leaf (Minecraft 1.21.11, PaperAPI/Paper) на Rust**

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Minecraft](https://img.shields.io/badge/Minecraft-1.21.11-green?style=for-the-badge&logo=minetest&logoColor=white)](https://www.minecraft.net)
[![JVM](https://img.shields.io/badge/runtime-JVM%2025%20on%20Rust-orange?style=for-the-badge)](https://github.com/levin1337/java-25-on-rust)
[![Status](https://img.shields.io/badge/status-work%20in%20progress-yellow?style=for-the-badge)]()

<p>
  <img src="https://img.shields.io/badge/Java-classes%20%26%20plugins-blueviolet?style=for-the-badge" alt="hybrid">
  <img src="https://img.shields.io/badge/goal-Leaf%20server-green?style=for-the-badge" alt="goal">
</p>

> 🚧 **Проект в активной разработке.** Ядро сервера переписывается на Rust
> **параллельно с работой над Rust-JVM** [java-25-on-rust](https://github.com/levin1337/java-25-on-rust) —
> нативные методы, которые JVM ещё не умеет, исполняются в JVM, пока код портируется в Rust.

</div>

---

## 🎯 Цель проекта

Переписать **Leaf server 1.21.11** (форк Paper) на **Rust**, сохранив:

- 🏗️ **Архитектуру оригинала** — пакетная структура `net.minecraft.*` / `org.bukkit.*` переносится 1:1 в `crates/leaf-core`
- ⚡ **Совместимость с плагинами** — цель — запуск Leaf-jar со всеми плагинами без ошибок
- 📡 **Сетевой стек** — протокол Minecraft 1.21.11, пакеты, кодеки, VarInt-заголовки
- 🧪 **Проверяемый порт** — критичные алгоритмы (RNG, NBT, math) побайтово идентичны Java-версии

Java-исходники Leaf (`leaf-server/`, `leaf-api/`, `paper-*/`) **намеренно не коммитятся** — они остаются локально как референс для порта.

---

## 🗺️ Структура репозитория

```
Leaf-ver-1.21.11/
├── crates/
│   ├── leaf-core/        🧬 Порт net.minecraft.* — ядро сервера
│   └── leaf-runner/      🚀 Лаунчер: ищет JVM, modules, leaf-jar и запускает
├── java-25-on-rust/      🦀 Rust-JVM (gitignored, см. AGENTS.md)
├── leaf-server/          📦 Java-исходники Leaf + собранные jar (gitignored)
└── run/                  ⚙️  Рабочая директория сервера
```

### `leaf-core` — карта портирования

```
crates/leaf-core/src/
├── nbt/            📦 NBT: binary IO, SNBT, Compound/Tag, MUTF-8
├── random/         🎲 RNG: Legacy/Single/Xoroshiro — bit-exact vs java.util.Random
├── util/           🛠️  ~54 модуля: mth, bit_storage, easing, debugchart,
│                   valueproviders, random, directories, value providers...
├── network/        🌐 Netty-стек: codec, connection, protocols
│   ├── codec/      💬 ByteBuf codecs, id_dispatch_codec, stream codec
│   ├── protocol/   📜 handshake ✅, status ✅, ping ✅, cookie ✅,
│   │               common ✅, login ✅, configuration ✅,
│   │               game — пакеты 1.21.11 (~175 из ~204 зарегистрировано в game_protocols)
│   └── chat/       💬 Компоненты чата, стили, JSON-сериализация
├── core/           🧱 Базовые типы: BlockPos, SectionPos, QuartPos,
│                   GlobalPos, Vec3i, Direction, Registry, Holder,
│                   SharedConstants, **component/** (data components)
├── joml/           📐 Полный порт JOML: Vector2f/3f/3i/4f, Quaternionf,
│                   AxisAngle4f, Matrix2f/3f/3x2f/4f — конвенции JOML
├── world/          🌍 Level, chunk, item (Item/ItemStack + components),
│                   level/block (Rotation/Mirror/StructureMode +
│                   block entities), scores, stats, waypoints,
│                   gametest, damagesource, effect, inventory
├── entity/         🧟 Entity-типы, ai/attributes (Attribute/Modifier)
├── sounds/         🔊 SoundEvent (holder-кодек)
├── tags/           🏷️  TagKey
├── player/         🧍 Игроки, input
├── level/          🗺️  Серверный уровень
├── server/         🖥️  ClientInformation, ParticleStatus, dialog, packs
├── chunk.rs         🧱 Чанки
├── command.rs       ⌨️  Система команд
├── locale/          🌐 Локализация / lang
├── resources/       📂 Identifier, ResourceKey
├── crash_report.rs  💥 Отчёты о падениях
└── plugin.rs        🔌 Plugin API-слой
```

Референс Java-исходников: `leaf-server/src/` и сгенерированный Mojmap-код Paper.

---

## 🚀 Быстрый старт

### Требования

- Rust (stable)
- **java-25-on-rust** собранный (`jvm25-rust`), JDK 25 с образом `lib/modules`
- Собранный `leaf-*.jar` (или dev-classpath)

### Сборка

```bash
# Rust-JVM (один раз)
cd java-25-on-rust && cargo build --release

# Лаунчер Leaf
cd .. && cargo build -p leaf-runner

# Leaf jar (если нужно пересобрать Java-часть):
./gradlew applyAllPatches                    # один раз
./gradlew :leaf-server:build
./gradlew :leaf-server:createMojmapBundlerJar
```

Gradle требует git identity через env:
`GIT_AUTHOR_NAME=... GIT_AUTHOR_EMAIL=... GIT_COMMITTER_NAME=... GIT_COMMITTER_EMAIL=...`

### Запуск сервера

```bash
cd run
../target/debug/leaf-runner -- nogui                                   # -jar режим
../target/debug/leaf-runner --cp @../leaf-server/build/runtime-classpath.txt \
    --main org.bukkit.craftbukkit.Main -- nogui                        # dev-режим
```

`leaf-runner` сам находит:

| Что | Откуда |
|---|---|
| `jvm25-rust` | env `JVM25_RUST` или `java-25-on-rust/target/{debug,release}/` |
| `lib/modules` JDK 25 | env `JAVA_HOME` или `java -XshowSettings` |
| `leaf-*.jar` | новейший `leaf-server/build/libs/leaf-*.jar` |

Опции: `--jvm`, `--modules`, `--jar`, `--java-home`, `--cp`, `--main`.
Всё после `--` передаётся серверу как есть.

### Тесты

```bash
cargo test -p leaf-core
```

Ключевые тесты: RNG bit-exact против `java.util.Random`, NBT round-trip, SNBT.

---

## 📈 Статус разработки

Актуально на 2026-11-17. **~25.5k строк Rust** в `crates/leaf-core/src`, 84 теста зелёные.

### ✅ Работает и проверено тестами

| Подсистема | Что сделано |
|---|---|
| **NBT** | Binary IO: все теги, MUTF-8, Compound/List, **SNBT-парсер** (`TagParser`), accounter — round-trip идентичен Java |
| **RNG** | Legacy/Single/ThreadLocalRandom/Xoroshiro — **bit-exact** против `java.util.Random` |
| **util/** | ~60 модулей: `mth`, bit_storage, easing, valueproviders, weighted random, debugchart, directories и др. |
| **core/** | `BlockPos`, `SectionPos`, `QuartPos`, `GlobalPos`, `Vec3i`, `Direction`, `Registry`, `SharedConstants`/`WorldVersion` |
| **network/codec** | ByteBuf-codec'и, `id_dispatch_codec`, stream codec, bundle codec'и |
| **network/protocol** | **ВСЕ протоколы дописаны** ✅ | Инфраструктура (ProtocolInfoBuilder, codec dispatch, bundler) + **все фазы**: handshake, status, ping, cookie, common, login, configuration ✅ + **game — все 180+ пакетов** (entity-движение/телепорты, respawn, position-sync, projectile, experience, spawn-position, tag-query, move/minecart/vehicle, take-item, entity-motion/passengers/link, bundle, border×5, ticking state/step, boss event, sound×2, damage event, mob-effect add/remove/update, attributes, objectives/teams/scores, waypoint, player-info update/remove, recipe-book, containers, chunks (`LevelChunkWithLight`/`LevelChunkPacketData`/`LightUpdate`/`ChunksBiomes`/heightmaps/palettes), entity data (`SetEntityData`/`AddEntity`/synched-data serializers), dialogs (`ShowDialog`/`CommonDialogData`/`Dialog`/`ActionButton`), debug subscriptions, commands...) |
| **network/chat** | Компоненты + стили + click/hover events + **JSON-сериализация** (`ComponentSerialization.CODEC`-аналог), `CommonComponents`, `ComponentUtils`, NumberFormat |
| **core/component** | **Data components полные**: `DataComponentType`/`Map`/`Patch`/`PatchedDataComponentMap`/`TypedDataComponent`, все **104 типа** vanilla-порядка; Send+Sync кодеки (`SyncFnCodec`); payload'ы: UseEffects, UseCooldown, CustomModelData, OminousBottleAmplifier, DyedItemColor, ItemLore, TooltipDisplay, DamageResistant, MapId |
| **world/item** | `Item` (registry + holderRegistry-кодек), **`ItemStack` полная модель** — `Holder<Item>` + `PatchedDataComponentMap`, все 4 wire-кодека (optional/untrusted-delimited/strict/list); `Rarity`, `DyeColor` |
| **joml/** | **Полный порт JOML**: Vector2f/3f/3i/4f, Quaternionf, AxisAngle4f, Matrix2f/3f/3x2f/4f, Math — конвенции и формулы как в JOML (тесты на трансформации/кватернионы) |
| **tags/** | `TagKey` (registry+location, stream-кодек) |
| **world/** | `Rotation`, `Mirror`, `StructureMode`, `TestBlockMode`, block entities, `GameTestInstance`, `RecipeBookType`/`Settings`, `DisplaySlot`, `MobEffect`, `DamageType`, `SoundEvent`, `BossEvent` enums, `Team`, `ObjectiveCriteria`, `Waypoint`/`TrackedWaypoint`, `MinecartStep`, `RespawnData`, `InteractionHand`, `Difficulty`, `LevelChunk`/`ProtoChunk`/palettes/`LevelChunkSection`, levelgen (noise/improved/perlin/normal, `DensityFunction`-система, `Climate` Parameter/RTree/SpawnFinder, biomes, `ChunkGenerator`/`NoiseBasedChunkGenerator`, heightmap, blending) |
| **entity/AI** | **`Mob`** (goal/target selectors, navigation, controls, sensing, leash/home/restriction), **`LivingEntity`** полный порт (~2600 строк: hurt-цепочка, эффекты, equipment, attributes), **AI-фреймворк**: `Goal`/`WrappedGoal`/`GoalSelector`/`GoalFlagSet`, `LookControl`/`MoveControl`/`FlyingMoveControl`/`JumpControl`/`BodyRotationControl`, `Sensing`, **pathfinding полный**: `Node`/`Target`/`Path`/`BinaryHeap`/`PathType`(+cache)/`PathfindingContext`/`NodeEvaluator`/`WalkNodeEvaluator`/`FlyNodeEvaluator`/`AmphibiousNodeEvaluator`/`SwimNodeEvaluator`/`PathFinder` (A*), **навигация**: `MobNavigation`/`PathNavigation` (~1000 строк: moveTo/tick/stuck-detection/trim/timeout) + `NavKind` Ground/Flying/Amphibious/WaterBound/WallClimber, `PathNavigationRegion`, `LevelReader`-интерфейс (Send+Sync), **AI-утилиты**: `RandomPos`/`LandRandomPos`/`HoverRandomPos`/`AirAndWaterRandomPos`/`DefaultRandomPos`/`GoalUtils`/`BehaviorUtils`, **конкретные цели**: `FloatGoal`/`PanicGoal`/`RandomStrollGoal`/`RandomSwimmingGoal`/`WaterAvoidingRandomStrollGoal`/`WaterAvoidingRandomFlyingGoal`/`RandomLookAroundGoal`/`LookAtPlayerGoal`/`JumpGoal`/`MoveTowardsTargetGoal` + все 63 goal-класса (`combat`/`movement`/`look`/`target`/`util`/`social`/`door`/`breed`/`special`/`village`), `BlockState` collision/pathfind API, `Entity`/`DamageSource`/`DamageTypes`/`DamageSources`/`CombatTracker`, `EntityType`+`MobCategory`, `SynchedEntityData` |
| **entity/AI Brain** | **Полный Brain-AI порт** ✅: `Brain` (activities/mensors/sensors, schedule), `MemoryModuleType` (~100 ключей) + `ExpirableValue`/`MemoryStatus`, `SensorType` + все сенсоры, `GossipContainer`, **behavior-фреймворк**: `BehaviorControl`/`BehaviorBase`/`PointBehavior`/`GateBehavior`(OrderPolicy/RunningPolicy)/`TriggerGate`/`ShufflingList`, **все 104 behavior-класса Java** (`behavior/`: simple/sinks/animal/walk/combat/long_jump/poi/villager_extra/villager_final/villager_goal_packages), `VillagerGoalPackages` — все 9 activity-пакетов (core/work/meet/rest/idle/panic/pre_raid/raid/hide/play) с ванильными приоритетами, `PoiManager`/`PoiType`/`PoiRecord`, `VillagerData`/`VillagerProfession`/`MerchantOffers`, `FireworkRocketEntity`, `Raid`/`Raids`/`RaidStatus`, `SimpleContainer`/`Container` |
| **world/level — взрывы** | **`Explosion`/`ServerExplosion` полный порт** ✅: `BlockInteraction`, `ExplosionDamageCalculator` (+Default/Simple/EntityBased), `calculateExplodedPositions` (16³-perimeter rays + power decay), `getSeenPercent` (bounding-box sampling + clip), `hurtEntities` (урон + blast-protection knockback + hitPlayers), `interactWithBlocks` (shuffle/TNT-prime/StackCollector merge≤16), `createFire`, `Explosion`-хуки в `LevelReader` (8 шт.) |
| **world/level — спавнеры** | **`BaseSpawner`/`SpawnData`/`EquipmentTable` полный порт** ✅: `serverTick`/`clientTick`/`delay`/NBT save-load, spawnPotentials `WeightedList`, `CustomSpawnRules` (light limits), `Spawner`/`CustomSpawner` трейты, `entity_type_registry` (аналог BuiltInRegistries.ENTITY_TYPE), 17 `LevelReader`-хуков |
| **world/level — NaturalSpawner** | **`NaturalSpawner` полный порт** ✅: `SpawnState` (energy budget через `PotentialCalculator`/`LocalMobCapCalculator`, per-player `chunkCap`), `createState`/`createStateAsync`, `getFilteredSpawningCategories`, `spawnForChunk`/`spawnCategoryForPosition` (random-bit packed loop, group size, caps), `spawnMobsForChunkGeneration`, `isValidEmptySpawnBlock`, fortress enemies, `PreSpawnStatus`; **`SpawnPlacements`** — полная таблица ~70 типов + placement types + семейные spawn-правила; `EntitySpawnReason`, `Mob::finalizeSpawn`/`checkSpawnObstruction`/cluster/group-size, `Entity::snapTo`; ~15 `LevelReader`-хуков |
| **world/level — structure-фреймворк** | **Полный порт базового слоя** ✅: `BoundingBox` (intersection/encapsulate/move/inflate/chunk-intersection/corners), `StructurePiece` (trait + `StructurePieceAccessor`, `makeBoundingBox`, `PieceTransform` world-pos трансформации, `placeBlock`/`generateBox`/`generateAirBox`/`generateMaybeBox`/`generateUpperHalfSphere`/`fillColumnDown`, `createChest`/`createDispenser`/`reorient`, `BlockSelector`, mirror/rotation-state трансформации), `PiecesContainer` + `StructurePieceType`-реестр (NBT save/load), `StructureStart` (references/boundingBox-cache/placeInChunk/createTag/loadStaticStart), `Structure` (settings/spawnOverrides/step/terrainAdaptation, `GenerationContext`/`GenerationStub`, `generate`/`findValidGenerationPoint`/`adjustBoundingBox`/corner-height helpers), `StructureManager` + `StructureManagerAccess`, `StructureCheck` (storage-scan/placement-restrictions/feature-check кэш через трейты), `SinglePieceStructure` + `ScatteredFeaturePieceData`, `TerrainAdjustment`/`StructureSpawnOverride`/`StructureType`/`StructureCheckResult`; 6 `LevelReader`-хуков |
| **resources/** | `Identifier`, `ResourceKey` |

### 🟡 В работе

| Подсистема | Состояние |
|---|---|
| **world/level — structure-генерация** | `templatesystem` ✅, `placement` ✅ (StructurePlacement/FrequencyReductionMethod/ExclusionZone, RandomSpread, ConcentricRings, StructureSet+Placement dispatch), `pools` ✅ **полностью**: StructureTemplatePool/Projection/JigsawJunction, все pool elements (Single/LegacySingle/List/Empty/Feature), StructureProcessorList, **`JigsawPlacement`** ✅ (weighted pool selection, PoolAliasLookup, projection Rigid/terrain, heightmap fallback, dimension padding, max-distance, collision через `VoxelShape::deflate_boxes`, max-depth recursion), `PoolElementStructurePiece` + `FrontAndTop`/`JigsawBlock`. **`structures/*` частично** ✅: buried treasure, desert pyramid, igloo, jungle temple, swamp hut, shipwreck, ocean ruin, ruined portal, nether fossil, jigsaw wrapper, end city (полные куски + piece-реализации), **mineshaft** ✅ (4 куска room/corridor/crossing/stairs, рекурсивная генерация, NORMAL/MESA типы, childEntranceBoxes, minecart-chest/spawner). Осталось: `structures/*` — nether fortress, stronghold, ocean monument, woodland mansion, `StructureFeatureIndexSavedData` |

### 🔴 Осталось

1. **Нативные методы JVM** — сервер упирается в natives (`FileNotFoundException`); часть нативов java.base не реализована в jvm25-rust.
2. **World/level runtime** — `Level`/`ServerLevel` (chunk loading, generation pipeline, lighting engine, block-entity ticking), конкретные структуры (`structure/structures/*`, `templatesystem`, `placement`, `pools`).
3. **Commands** — brigadier-база → `CommandNode`-дерево, dispatcher, argument types, synchronization (пакеты уже есть).
4. **Entity-иерархия** — `Player` полный порт, projectile/item-entity Mob-операции через LevelReader-хуки.
5. **Component payloads** — ~55 типов (`FoodProperties`, `Consumable`, `PotionContents`, `Equippable`, `Tool`...) + registry bootstrap (`HolderSet`/`EitherHolder`).
6. **Plugin API** — Bukkit/Paper события и менеджеры.
7. **Полный прогон** — `nogui` сервер должен дойти до "Done".

### 🏃 Текущий запуск

```bash
cd run
../target/debug/leaf-runner -- nogui
# → bootstrap ~0.8s → PaperBootstrap → FileNotFoundException (нативы JVM)
```

---

## 🤝 Contributing

Порт идёт пакетно: берёшь Java-класс из `leaf-server/`, пишешь Rust-аналог в
`crates/leaf-core` с тем же API-именем (`snake_case`), добавляешь тесты
против Java-поведения.

1. 🍴 Сделай fork
2. 🌿 Создай ветку `feature/<module>`
3. 🔀 Открой Pull Request

Подробности окружения и сборки — в [`AGENTS.md`](AGENTS.md).
