//! Перенос `com.mojang.authlib` — GameProfile, Property, PropertyMap.

pub mod game_profile;
pub mod property;
pub mod property_map;

pub use game_profile::GameProfile;
pub use property::Property;
pub use property_map::PropertyMap;
