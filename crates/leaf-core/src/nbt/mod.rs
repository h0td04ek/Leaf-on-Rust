//! Перенос `net.minecraft.nbt` — бинарный NBT (Named Binary Tag).
//!
//! Пока портировано: модель тегов (`Tag`, `CompoundTag`), `NbtAccounter`
//! и бинарное чтение/запись (`NbtIo`). Visitor API, SNBT (`TagParser`,
//! `Snbt*`) и `NbtOps` — следующими итерациями.

pub mod accounter;
pub mod compound;
pub mod io;
pub mod mutf8;
pub mod snbt;
pub mod snbt_parse;
pub mod tag;

pub use accounter::NbtAccounter;
pub use compound::CompoundTag;
pub use tag::Tag;
