//! Перенос `net.minecraft.util.datafix` — `DataFixTypes` и `DataFixer`-трейт.
//!
//! DFU (`com.mojang.datafixers`) в Rust не портирован: `Dynamic<T>`
//! отображается на `CompoundTag` (NbtOps), `TypeReference` — на
//! [`DataFixTypes`]. Сам fixer подключается через [`DataFixer`].

pub mod data_fix_types;
pub mod data_fixer;

pub use data_fix_types::DataFixTypes;
pub use data_fixer::DataFixer;
