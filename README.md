# Leaf-on-Rust

Форк Leaf (Minecraft server, 1.21.11) — ядро переписывается на Rust.

## Как это устроено

- **`crates/leaf-core`** — Rust-порт `net.minecraft.*` (nbt, random, util/mth...)
- **`crates/leaf-runner`** — лаунчер сервера на Rust-JVM
- **JVM**: [levin1337/java-25-on-rust](https://github.com/levin1337/java-25-on-rust) — порт OpenJDK 25 HotSpot на Rust (клонируется отдельно в `java-25-on-rust/`)

Java-исходники Leaf остаются локально (не в репо) — это референс для порта и исходник jar'а.

## Сборка и запуск

```bash
# 1. JVM
git clone https://github.com/levin1337/java-25-on-rust java-25-on-rust
cd java-25-on-rust && cargo build --release && cd ..

# 2. Лаунчер
cargo build -p leaf-runner

# 3. Сервер (нужны Java-исходники Leaf + Gradle):
#    ./gradlew applyAllPatches && ./gradlew :leaf-server:build
#   classpath для dev-режима: leaf-server/build/runtime-classpath.txt

# 4. Запуск
cd run && ../target/debug/leaf-runner -- nogui
# или с classpath напрямую:
../target/debug/leaf-runner --cp @../leaf-server/build/runtime-classpath.txt \
    --main org.bukkit.craftbukkit.Main -- nogui
```

Тесты: `cargo test -p leaf-core`

См. `AGENTS.md` для деталей сборки.
