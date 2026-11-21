//! Перенос `net.minecraft.world.item.component.LodestoneTracker`.

use crate::core::block_pos::BlockPos;
use crate::core::global_pos::GlobalPos;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use crate::world::level::LevelKey;

/// `LodestoneTracker` — (target?, tracked).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LodestoneTracker {
    /// `target`.
    pub target: Option<GlobalPos>,
    /// `tracked`.
    pub tracked: bool,
}

impl LodestoneTracker {
    /// `new`.
    pub fn new(target: Option<GlobalPos>, tracked: bool) -> Self {
        Self { target, tracked }
    }

    /// `GlobalPos.STREAM_CODEC` — dimension ResourceKey (только
    /// Identifier на проводе) + BLOCK_POS.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, LodestoneTracker> + Send + Sync + 'static {
        let target_enc = crate::network::codec::byte_buf_codecs::optional(Self::global_pos_codec());
        let target_dec = crate::network::codec::byte_buf_codecs::optional(Self::global_pos_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &LodestoneTracker| {
                target_enc.encode(buf, &v.target);
                buf.write_bool(v.tracked);
            },
            move |buf: &mut FriendlyByteBuf| {
                let target = target_dec.decode(buf);
                let tracked = buf.read_bool();
                LodestoneTracker::new(target, tracked)
            },
        )
    }

    /// `GlobalPos` stream codec — `ResourceKey.streamCodec` пишет
    /// только Identifier (registry `minecraft:dimension`).
    fn global_pos_codec() -> impl StreamCodec<FriendlyByteBuf, GlobalPos> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &GlobalPos| {
                buf.write_identifier(&v.dimension.identifier).expect("dimension id");
                buf.write_block_pos(&v.pos);
            },
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_identifier().expect("dimension id");
                let dimension = ResourceKey::create_with_registry_name(
                    Identifier::parse("minecraft:dimension").expect("dimension registry"),
                    id,
                );
                let pos = buf.read_block_pos();
                GlobalPos::of(dimension, pos)
            },
        )
    }
}
