//! `ClickType` — тип клика в контейнере.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `ClickType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClickType {
    /// `PICKUP`.
    Pickup,
    /// `QUICK_MOVE`.
    QuickMove,
    /// `SWAP`.
    Swap,
    /// `CLONE`.
    Clone,
    /// `THROW`.
    Throw,
    /// `QUICK_CRAFT`.
    QuickCraft,
    /// `PICKUP_ALL`.
    PickupAll,
}

impl ClickType {
    /// `VALUES`.
    pub const VALUES: [ClickType; 7] = [
        ClickType::Pickup,
        ClickType::QuickMove,
        ClickType::Swap,
        ClickType::Clone,
        ClickType::Throw,
        ClickType::QuickCraft,
        ClickType::PickupAll,
    ];

    /// `id()`.
    pub fn id(&self) -> i32 {
        match self {
            ClickType::Pickup => 0,
            ClickType::QuickMove => 1,
            ClickType::Swap => 2,
            ClickType::Clone => 3,
            ClickType::Throw => 4,
            ClickType::QuickCraft => 5,
            ClickType::PickupAll => 6,
        }
    }

    /// `by_id` — `OutOfBoundsStrategy.ZERO`.
    pub fn by_id(id: i32) -> ClickType {
        ClickType::VALUES[id.rem_euclid(7) as usize]
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.idMapper(BY_ID, ClickType::id)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClickType> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, c: &ClickType| { buf.write_var_int(c.id()); },
            |buf: &mut FriendlyByteBuf| ClickType::by_id(buf.read_var_int()),
        )
    }
}
