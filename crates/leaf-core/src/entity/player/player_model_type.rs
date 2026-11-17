//! Перенос `net.minecraft.world.entity.player.PlayerModelType`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `PlayerModelType` — модель скина игрока.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlayerModelType {
    /// `SLIM` ("slim"/"slim").
    Slim,
    /// `WIDE` ("wide"/"default").
    Wide,
}

impl PlayerModelType {
    /// `STREAM_CODEC` — `ByteBufCodecs.BOOL.map(SLIM/WIDE)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, PlayerModelType> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &PlayerModelType| {
                buf.write_bool(*v == PlayerModelType::Slim);
            },
            |buf: &mut FriendlyByteBuf| {
                if buf.read_bool() { PlayerModelType::Slim } else { PlayerModelType::Wide }
            },
        )
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            PlayerModelType::Slim => "slim",
            PlayerModelType::Wide => "wide",
        }
    }

    /// `legacyServicesId`.
    pub fn legacy_services_id(&self) -> &'static str {
        match self {
            PlayerModelType::Slim => "slim",
            PlayerModelType::Wide => "default",
        }
    }
}
