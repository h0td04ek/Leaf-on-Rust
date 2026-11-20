//! `net.minecraft.world.ticks` — планировщик тиков блоков/жидкостей.

pub mod blackhole;
pub mod level_chunk_ticks;
pub mod level_ticks;
pub mod proto_chunk_ticks;
pub mod scheduled_tick;
pub mod tick_access;
pub mod tick_priority;

pub use blackhole::{empty_container, empty_level_list, BlackholeContainer, BlackholeLevel, WorldGenTickAccess};
pub use level_chunk_ticks::LevelChunkTicks;
pub use level_ticks::LevelTicks;
pub use proto_chunk_ticks::ProtoChunkTicks;
pub use scheduled_tick::{drain_order_cmp, intra_tick_drain_order_cmp, SavedTick, ScheduledTick, TickKey, TickType};
pub use tick_access::{LevelTickAccess, SerializableTickContainer, TickAccess, TickContainerAccess};
pub use tick_priority::TickPriority;
