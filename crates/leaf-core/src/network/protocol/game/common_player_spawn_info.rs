//! Перенос `net.minecraft.network.protocol.game.CommonPlayerSpawnInfo`.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::level::game_type::GameType;
use crate::world::level::Level;
use crate::resources::ResourceKey;
use crate::core::global_pos::GlobalPos;

/// Маркер `DimensionType` — полная запись портируется в
/// `world::level::dimension_type`; по сети передаётся только
/// registry id (`ByteBufCodecs.holderRegistry`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DimensionType;

/// `CommonPlayerSpawnInfo`.
#[derive(Clone, Debug, PartialEq)]
pub struct CommonPlayerSpawnInfo {
    /// `dimensionType` — `Holder<DimensionType>`.
    pub dimension_type: Holder<DimensionType>,
    /// `dimension`.
    pub dimension: ResourceKey<Level>,
    /// `seed`.
    pub seed: i64,
    /// `gameType`.
    pub game_type: GameType,
    /// `previousGameType`.
    pub previous_game_type: Option<GameType>,
    /// `isDebug`.
    pub is_debug: bool,
    /// `isFlat`.
    pub is_flat: bool,
    /// `lastDeathLocation`.
    pub last_death_location: Option<GlobalPos>,
    /// `portalCooldown`.
    pub portal_cooldown: i32,
    /// `seaLevel`.
    pub sea_level: i32,
}

impl CommonPlayerSpawnInfo {
    /// Декодирование (как конструктор из `RegistryFriendlyByteBuf`).
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        // DimensionType.STREAM_CODEC = holderRegistry → VarInt id
        let dimension_type = Holder::Reference(buf.read_var_int());
        let dimension = buf.read_resource_key_for::<Level>(
            crate::core::registry::registries::dimension(),
        );
        let seed = buf.read_long();
        let game_type = GameType::by_id(buf.read_byte() as i32);
        let previous_game_type = GameType::by_nullable_id(buf.read_byte());
        let is_debug = buf.read_bool();
        let is_flat = buf.read_bool();
        let last_death_location = buf.read_optional(|b| b.read_global_pos());
        let portal_cooldown = buf.read_var_int();
        let sea_level = buf.read_var_int();
        Self {
            dimension_type,
            dimension,
            seed,
            game_type,
            previous_game_type,
            is_debug,
            is_flat,
            last_death_location,
            portal_cooldown,
            sea_level,
        }
    }

    /// Кодирование.
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        match self.dimension_type {
            Holder::Reference(id) => {
                buf.write_var_int(id);
            }
            Holder::Direct(_) => panic!("DimensionType direct holder cannot be encoded without registry"),
            Holder::Key(_) => panic!("DimensionType unbound key cannot be encoded without registry"),
        }
        let _ = buf.write_resource_key(&self.dimension);
        buf.write_long(self.seed);
        buf.write_byte(self.game_type.get_id());
        buf.write_byte(GameType::get_nullable_id(self.previous_game_type));
        buf.write_bool(self.is_debug);
        buf.write_bool(self.is_flat);
        buf.write_optional(&self.last_death_location, |b, g| {
            b.write_global_pos(g);
        });
        buf.write_var_int(self.portal_cooldown);
        buf.write_var_int(self.sea_level);
    }

    /// `STREAM_CODEC`-обёртка.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, CommonPlayerSpawnInfo> {
        crate::network::codec::stream_codec::FnCodec::new(
            |buf: &mut FriendlyByteBuf, v: &CommonPlayerSpawnInfo| v.write(buf),
            |buf: &mut FriendlyByteBuf| CommonPlayerSpawnInfo::read(buf),
        )
    }
}
