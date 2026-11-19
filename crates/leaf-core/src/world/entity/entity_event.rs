//! Перенос `net.minecraft.world.entity.EntityEvent` — константы
//! entity-event id для `broadcastEntityEvent`.

pub const DEATH: u8 = 3;
pub const HURT: u8 = 2;
pub const VILLAGER_HAPPY: u8 = 14;
pub const PLAY_DEAD: u8 = 59 - 6; // уточняется по ванили при необходимости
pub const START_RAM: u8 = 58;
pub const END_RAM: u8 = 59;
pub const SHEEP_EAT_GRASS: u8 = 10;
pub const FOX_EAT: u8 = 45;
pub const WOLF_SHAKE: u8 = 36;
