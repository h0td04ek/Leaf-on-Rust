//! Перенос `net.minecraft.world.item.component.Bees`.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::level::block::entity::beehive_block_entity::BeehiveOccupant;

/// `Bees` — список `BeehiveBlockEntity.Occupant`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bees {
    /// `bees`.
    pub bees: Vec<BeehiveOccupant>,
}

impl Bees {
    /// `new`.
    pub fn new(bees: Vec<BeehiveOccupant>) -> Self {
        Self { bees }
    }

    /// `Bees.STREAM_CODEC` — list(Occupant).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Bees> + Send + Sync + 'static {
        let list_enc = byte_buf_codecs::list(BeehiveOccupant::stream_codec());
        let list_dec = byte_buf_codecs::list(BeehiveOccupant::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &Bees| {
                list_enc.encode(buf, &v.bees);
            },
            move |buf: &mut FriendlyByteBuf| Bees { bees: list_dec.decode(buf) },
        )
    }
}
