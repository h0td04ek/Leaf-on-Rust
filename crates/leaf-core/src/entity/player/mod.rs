//! Перенос `net.minecraft.world.entity.player` — типы игрока.

pub mod abilities;
pub mod chat_visibility;
pub mod input;
pub mod player_model_type;
pub mod player_skin;
pub mod profile_key_pair;
pub mod profile_public_key;

pub use player_model_type::PlayerModelType;
pub use player_skin::{PlayerSkinPatch, ResourceTexture};
