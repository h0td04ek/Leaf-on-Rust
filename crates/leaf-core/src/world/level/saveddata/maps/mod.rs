//! Перенос `net.minecraft.world.level.saveddata.maps`.

pub mod map_decoration;
pub mod map_decoration_type;
pub mod map_id;
pub mod map_patch;

pub use map_decoration::MapDecoration;
pub use map_decoration_type::MapDecorationType;
pub use map_id::MapId;
pub use map_patch::MapPatch;
