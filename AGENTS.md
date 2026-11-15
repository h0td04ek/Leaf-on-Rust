# Leaf-on-Rust

Форк Leaf (Minecraft server 1.21.11). Ядро переписывается на Rust,
работает поверх Rust-JVM [java-25-on-rust](https://github.com/levin1337/java-25-on-rust).

## Репозиторий содержит только Rust-часть

- `crates/leaf-core` — порт `net.minecraft.*` (nbt, random, util/mth...)
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

## Тесты

```bash
cargo test -p leaf-core
```

## Статус

- jvm25-rust собран и исполняет Java (bootstrap ~0.8с), сервер доходит
  до `PaperBootstrap`, дальше FileNotFoundException в natives (баг JVM).
- Портировано: `nbt` (binary IO + SNBT print), `random` (Legacy/Single/
  Xoroshiro, bit-exact vs java.util.Random), `util/mth` (getSeed).
