//! Перенос `net.minecraft.world.level.block.state.properties.TestBlockMode`.

use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `TestBlockMode` — id + serialized name.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TestBlockMode {
    /// `START`.
    START,
    /// `LOG`.
    LOG,
    /// `FAIL`.
    FAIL,
    /// `ACCEPT`.
    ACCEPT,
}

impl TestBlockMode {
    /// `values()` — порядок = id.
    pub const VALUES: [TestBlockMode; 4] = [
        TestBlockMode::START,
        TestBlockMode::LOG,
        TestBlockMode::FAIL,
        TestBlockMode::ACCEPT,
    ];

    /// `id`.
    pub fn id(&self) -> i32 {
        *self as i32
    }

    /// `ByIdMap.continuous(..., OutOfBoundsStrategy.ZERO)` — clamp в [0,3].
    pub fn by_id(id: i32) -> TestBlockMode {
        Self::VALUES[id.clamp(0, 3) as usize]
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            TestBlockMode::START => "start",
            TestBlockMode::LOG => "log",
            TestBlockMode::FAIL => "fail",
            TestBlockMode::ACCEPT => "accept",
        }
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.idMapper(BY_ID, id)` (VarInt id).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, TestBlockMode> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, v: &TestBlockMode| {
                buf.write_var_int(v.id());
            },
            |buf: &mut FriendlyByteBuf| TestBlockMode::by_id(buf.read_var_int()),
        )
    }
}
