//! leaf-core — Rust-перенос ядра Leaf (Minecraft server, 1.21.11).
//!
//! Структура модулей зеркалит пакет `net.minecraft.server` / `net.minecraft.world`
//! декомпилированных исходников (`leaf-server/src/minecraft`), чтобы перенос
//! шёл пакет-за-пакетом, как HotSpot→Rust в java-25-on-rust.
//!
//! Сервер запускается на JVM, переписанной на Rust
//! (`java-25-on-rust`, бинарь `jvm25-rust`), через `leaf-runner`.
//! Постепенно горячие подсистемы будут переписываться на Rust и
//! подключаться к JVM через JNI или заменять Java-классы.

pub mod core;
pub mod crash_report;
pub mod joml;
pub mod nbt;
pub mod random;
pub mod reported_exception;
pub mod server;
pub mod util;
pub mod world;
pub mod level;
pub mod entity;
pub mod player;
pub mod chunk;
pub mod locale;
pub mod network;
pub mod resources;
pub mod command;
pub mod plugin;
