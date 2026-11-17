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
│   │               game — пакеты 1.21.11 (~70 из ~190 портировано)
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

Актуально на 2026-11-17. **~50k строк Rust** в `crates/leaf-core`, 75 тестов зелёные.

### ✅ Работает и проверено тестами

| Подсистема | Что сделано |
|---|---|
| **NBT** | Binary IO: все теги, MUTF-8, Compound/List, **SNBT-парсер** (`TagParser`), accounter — round-trip идентичен Java |
| **RNG** | Legacy/Single/ThreadLocalRandom/Xoroshiro — **bit-exact** против `java.util.Random` |
| **util/** | ~60 модулей: `mth`, bit_storage, easing, valueproviders, weighted random, debugchart, directories и др. |
| **core/** | `BlockPos`, `SectionPos`, `QuartPos`, `GlobalPos`, `Vec3i`, `Direction`, `Registry`, `SharedConstants`/`WorldVersion` |
| **network/codec** | ByteBuf-codec'и, `id_dispatch_codec`, stream codec, bundle codec'и |
| **network/protocol** | Инфраструктура (ProtocolInfoBuilder, codec dispatch, bundler) + пакеты: **handshake, status, ping, cookie, common, login, configuration** ✅ + **game ~70 пакетов** (entity-движение/телепорты, respawn, position-sync, projectile, experience, spawn-position, tag-query, move/minecart/vehicle, take-item, entity-motion/passengers/link, bundle, border×5, ticking state/step, boss event, sound×2, damage event, mob-effect add/remove/update, attributes, objectives/teams/scores, waypoint, player-info update/remove, recipe-book settings/remove, serverbound: structure/jigsaw/test-block/beacon/command-blocks/recipe-book/pick-item/use-item-on...) |
| **network/chat** | Компоненты + стили + click/hover events + **JSON-сериализация** (`ComponentSerialization.CODEC`-аналог), `CommonComponents`, `ComponentUtils`, NumberFormat |
| **core/component** | **Data components полные**: `DataComponentType`/`Map`/`Patch`/`PatchedDataComponentMap`/`TypedDataComponent`, все **104 типа** vanilla-порядка; Send+Sync кодеки (`SyncFnCodec`); payload'ы: UseEffects, UseCooldown, CustomModelData, OminousBottleAmplifier, DyedItemColor, ItemLore, TooltipDisplay, DamageResistant, MapId |
| **world/item** | `Item` (registry + holderRegistry-кодек), **`ItemStack` полная модель** — `Holder<Item>` + `PatchedDataComponentMap`, все 4 wire-кодека (optional/untrusted-delimited/strict/list); `Rarity`, `DyeColor` |
| **joml/** | **Полный порт JOML**: Vector2f/3f/3i/4f, Quaternionf, AxisAngle4f, Matrix2f/3f/3x2f/4f, Math — конвенции и формулы как в JOML (тесты на трансформации/кватернионы) |
| **tags/** | `TagKey` (registry+location, stream-кодек) |
| **world/** | `Rotation`, `Mirror`, `StructureMode`, `TestBlockMode`, block entities (command/jigsaw/structure/test-instance), `GameTestInstance`, `RecipeBookType`/`Settings`, `RecipeDisplayId`, `DisplaySlot`, `MobEffect`, `DamageType`, `SoundEvent`, `BossEvent` enums, `Team`, `ObjectiveCriteria`, `Waypoint`/`TrackedWaypoint`, `MinecartStep`, `RespawnData`, `InteractionHand`, `Difficulty` |
| **resources/** | `Identifier`, `ResourceKey` |

### 🟡 В работе

| Подсистема | Состояние |
|---|---|
| **Запуск сервера** | JVM исполняет Java (~0.8s bootstrap), сервер доходит до `PaperBootstrap`, дальше `FileNotFoundException` в natives — баг jvm25-rust |
| **network/protocol/game** | Осталось ~120 пакетов: chunks (`LevelChunkWithLight`, `ChunksBiomes`, `LightUpdate`, heightmaps/palettes), entity data (`SetEntityData`, `AddEntity`), containers (content/slot/click/merchant/cursor/inventory), `LoginPacket`, map item, particles, explode, recipe book add/update-recipes, advancements, command tree (`CommandNode`), debug subscriptions |
| **core/component payloads** | ~55 из 104 типов ждут свои payload-классы (`FoodProperties`, `Consumable`, `PotionContents`, `ArmorTrim`, `Equippable`, `Tool`, `AdventureModePredicate`, `ItemEnchantments`, books, fireworks, bees, entity-variant реестры, `EitherHolder`...) — wire-id порядок уже корректен |
| **registries** | Нужны `HolderSet`/`EitherHolder` и bootstrap реестров (ITEM, DAMAGE_TYPE, ENTITY_TYPE, SOUND_EVENT...) для полной резолва holder'ов |
| **world/level** | Чанки, lighting, level-логика — в портировании |
| **entity/player/command** | Command-дерево и entity-логика — следующие этапы |

### 🔴 Осталось

1. **Нативные методы JVM** — сервер упирается в natives (`FileNotFoundException`); часть нативов java.base не реализована в jvm25-rust.
2. **Протоколы login/configuration/game** — регистрация пакетов в ProtocolInfo.
3. **World/level** — генерация чанков, lighting, entities.
4. **Plugin API** — Bukkit/Paper события и менеджеры.
5. **Полный прогон** — `nogui` сервер должен дойти до "Done".

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
