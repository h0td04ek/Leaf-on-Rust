//! `net.minecraft.world.timeline` — таймлайны окружения.

pub mod attribute_track;
pub mod attribute_track_sampler;
pub mod timeline;
pub mod timelines;

pub use attribute_track::AttributeTrack;
pub use attribute_track_sampler::AttributeTrackSampler;
pub use timeline::{Timeline, TimelineBuilder};
