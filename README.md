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
│   ├── protocol/   📜 handshake, status, ping, cookie, login,
│   │               configuration, game — пакеты 1.21.11
│   └── chat/       💬 Компоненты чата, JSON-сериализация
├── core/           🧱 Базовые типы: BlockPos, SectionPos, QuartPos,
│                   GlobalPos, Vec3i, Direction, Registry
├── world/          🌍 Level, chunk, entity/item, phys
├── level/          🗺️  Серверный уровень
├── entity.rs        🧟 Сущности
├── player.rs        🧍 Игроки
├── chunk.rs         🧱 Чанки
├── command.rs       ⌨️  Система команд
├── server.rs        🖥️  Серверный entry point
├── locale/          🌐 Локализация / lang
├── resources/       📂 Resource packs, datapacks
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

Актуально на 2026-11-17. **~22k строк Rust** в `crates/leaf-core`.

### ✅ Работает и проверено тестами

| Подсистема | Что сделано |
|---|---|
| **NBT** | Полный binary IO: все теги, MUTF-8, Compound/List, SNBT-вывод, accounter — round-trip идентичен Java |
| **RNG** | Legacy/Single/ThreadLocalRandom/Xoroshiro — **bit-exact** против `java.util.Random` (автоматический тест) |
| **util/mth** | Seed-функции, математика уровня `net.minecraft.util.Mth` |
| **core/** | `BlockPos`, `SectionPos`, `QuartPos`, `GlobalPos`, `Vec3i`, `Direction` — портed со всеми граничными случаями |
| **network/codec** | ByteBuf-codec'и, `id_dispatch_codec`, stream codec, исключения кодирования |
| **network/protocol** | Структуры пакетов всех фаз 1.21.11: handshake, status, ping, cookie, login, configuration, game |
| **network/chat** | Компоненты чата, сериализация, стили |
| **util/** | ~54 модуля: easing, bit_storage, valueproviders, debugchart, memory_reserve и др. |
| **leaf-runner** | Авто-поиск JVM/modules/jar, `-jar` и `--cp` dev-режимы |

### 🟡 В работе

| Подсистема | Состояние |
|---|---|
| **Запуск сервера** | JVM исполняет Java (~0.8s bootstrap), сервер доходит до `PaperBootstrap`, дальше `FileNotFoundException` в natives — баг jvm25-rust |
| **world/level** | Чанки, level, phys — в портировании |
| **Сетевой цикл** | Кодеки и пакеты готовы; полный Connection→Pipeline ещё собирается |
| **entity/player/command** | Заготовки модулей |

### 🔴 Осталось

1. **Нативные методы JVM** — сервер упирается в natives (`FileNotFoundException`); часть из ~77 нативов java.base не реализована в jvm25-rust.
2. **Packet → ProtocolInfo wiring** — привязка пакетов к protocol info builder.
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
