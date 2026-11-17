# Leaf-on-Rust

Форк Leaf (Minecraft server 1.21.11). Ядро переписывается на Rust,
работает поверх Rust-JVM [java-25-on-rust](https://github.com/levin1337/java-25-on-rust).

## Репозиторий содержит только Rust-часть

- `crates/leaf-core` — порт `net.minecraft.*` / `org.bukkit.*` (~22k строк):
  `nbt`, `random`, `util/` (~54 модуля), `network/` (codec + protocols
  1.21.11: handshake/status/ping/cookie/login/configuration/game/chat),
  `core/` (BlockPos/SectionPos/QuartPos/GlobalPos/Vec3i/Direction/Registry),
  `world/`, `level/`, `entity.rs`, `player.rs`, `chunk.rs`, `command.rs`,
  `server.rs`, `locale/`, `resources/`, `plugin.rs`, `crash_report.rs`
- `crates/leaf-runner` — лаунчер сервера на jvm25-rust

Java-исходники Leaf (`leaf-server/`, `leaf-api/`, `paper-*/`) намеренно
не коммитятся — они остаются локально как референс для порта.

## Окружение (локально)

- `java-25-on-rust/` — клон https://github.com/levin1337/java-25-on-rust
  (gitignored). ВНИМАНИЕ: на его HEAD есть локальные фиксы сборки
  (`src/share/c1/c1_Compiler.rs`, `src/share/c1/c1_LIR.rs`) — без них не компилится.
- `leaf-server/` — Java-исходники Leaf + собранные jar'ы.
- `run/` — рабочая директория сервера.

## Сборка

```bash
# Rust-JVM (внутри java-25-on-rust/)
cd java-25-on-rust && cargo build --release

# Лаунчер
cd .. && cargo build -p leaf-runner

# Leaf jar (если нужно пересобрать):
./gradlew applyAllPatches   # один раз
./gradlew :leaf-server:build
./gradlew :leaf-server:createMojmapBundlerJar
# dev-classpath:
./gradlew -I <init> :leaf-server:printRuntimeClasspath --no-configuration-cache
```

Gradle требует git identity через env:
`GIT_AUTHOR_NAME=... GIT_AUTHOR_EMAIL=... GIT_COMMITTER_NAME=... GIT_COMMITTER_EMAIL=...`

## Запуск сервера на Rust-JVM

```bash
cd run
../target/debug/leaf-runner -- nogui                                   # -jar режим
../target/debug/leaf-runner --cp @../leaf-server/build/runtime-classpath.txt \
    --main org.bukkit.craftbukkit.Main -- nogui                        # dev-режим
```

`leaf-runner` сам находит `jvm25-rust` (env `JVM25_RUST`), `lib/modules`
JDK (env `JAVA_HOME` или `java -XshowSettings`) и новейший `leaf-*.jar`.
Опции: `--jvm`, `--modules`, `--jar`, `--java-home`, `--cp`, `--main`.
Всё после `--` передаётся серверу как есть.

## Тесты

```bash
cargo test -p leaf-core
```

Ключевые: RNG bit-exact против `java.util.Random`, NBT round-trip,
SNBT. Подробности — в `README.md`.

## Статус

- jvm25-rust собран и исполняет Java (bootstrap ~0.8с), сервер доходит
  до `PaperBootstrap`, дальше FileNotFoundException в natives (баг JVM).
- Проверено тестами: `nbt` (binary IO + SNBT), `random` (Legacy/Single/
  Xoroshiro, bit-exact), `util/mth`, `core/` (позиции),
  `network/codec` + протоколы пакетов 1.21.11, `network/chat`, ~60 тестов.
- В работе: `world/`/`level/` (чанки, lighting), wiring пакетов в
  ProtocolInfo, entity/player/command, plugin API.
