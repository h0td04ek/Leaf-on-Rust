//! Перенос variant-типов entity для data components
//! (`fox/variant`, `salmon/size`, ..., `horse/variant`).
//!
//! Два вида:
//! - **enum-варианты** (`FoxVariant`, `SalmonSize`, ...) — idMapper
//!   stream-кодеки, как `ByteBufCodecs.idMapper(BY_ID, getId)`.
//! - **registry-варианты** (`VillagerType`, `WolfVariant`, ...) —
//!   `holderRegistry` кодеки (VarInt id); record-поля
//!   (assets/spawn conditions) портируются с entity-стадией,
//!   wire-идентичность — registry id.

use crate::core::holder::{holder_registry_stream_codec, Holder};
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

macro_rules! id_enum {
    ($(#[$m:meta])* $name:ident { $($v:ident => ($id:expr, $ser:literal)),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $name { $($v),+ }

        impl $name {
            /// `values()`.
            pub const VALUES: &'static [$name] = &[$($name::$v),+];
            /// `getId`.
            pub fn get_id(&self) -> i32 {
                match self { $( $name::$v => $id ),+ }
            }
            /// `byId` — continuous ZERO: вне диапазона → первый.
            pub fn by_id(id: i32) -> $name {
                Self::VALUES.iter().copied().find(|v| v.get_id() == id)
                    .unwrap_or(Self::VALUES[0])
            }
            /// `getSerializedName`.
            pub fn get_serialized_name(&self) -> &'static str {
                match self { $( $name::$v => $ser ),+ }
            }
            /// `byName`.
            pub fn by_name(name: &str) -> Option<$name> {
                Self::VALUES.iter().copied().find(|v| v.get_serialized_name() == name)
            }
            /// `STREAM_CODEC` — `ByteBufCodecs.idMapper(BY_ID, getId)`.
            pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, $name> + Send + Sync + 'static {
                sync_fn(
                    |buf: &mut FriendlyByteBuf, v: &$name| {
                        buf.write_var_int(v.get_id());
                    },
                    |buf: &mut FriendlyByteBuf| $name::by_id(buf.read_var_int()),
                )
            }
        }
    };
}

id_enum! {
    /// `Fox.Variant` (`fox/variant`).
    FoxVariant { RED => (0, "red"), SNOW => (1, "snow") }
}
id_enum! {
    /// `Salmon.Variant` (`salmon/size`).
    SalmonSize { SMALL => (0, "small"), MEDIUM => (1, "medium"), LARGE => (2, "large") }
}
id_enum! {
    /// `Parrot.Variant` (`parrot/variant`).
    ParrotVariant {
        RED_BLUE => (0, "red_blue"), BLUE => (1, "blue"), GREEN => (2, "green"),
        YELLOW_BLUE => (3, "yellow_blue"), GRAY => (4, "gray")
    }
}
id_enum! {
    /// `TropicalFish.Pattern` (`tropical_fish/pattern`).
    TropicalFishPattern {
        KOB => (0, "kob"), SUNSTREAK => (1, "sunstreak"), SNOOPER => (2, "snooper"),
        DASHER => (3, "dasher"), BRINELY => (4, "brinely"), SPOTTY => (5, "spotty"),
        FLOPPER => (6, "flopper"), STRIPEY => (7, "stripey"), GLITTER => (8, "glitter"),
        BLOCKFISH => (9, "blockfish"), BETTY => (10, "betty"), CLAYFISH => (11, "clayfish")
    }
}
id_enum! {
    /// `MushroomCow.Variant` (`mooshroom/variant`).
    MooshroomVariant { RED => (0, "red"), BROWN => (1, "brown") }
}
id_enum! {
    /// `Rabbit.Variant` (`rabbit/variant`).
    RabbitVariant {
        BROWN => (0, "brown"), WHITE => (1, "white"), BLACK => (2, "black"),
        WHITE_SPLOTCHED => (3, "white_splashed"), GOLD => (4, "gold"),
        SALT => (5, "salt"), EVIL => (6, "evil")
    }
}
id_enum! {
    /// `Axolotl.Variant` (`axolotl/variant`).
    AxolotlVariant {
        LUCY => (0, "lucy"), WILD => (1, "wild"), GOLD => (2, "gold"),
        CYAN => (3, "cyan"), BLUE => (4, "blue")
    }
}
id_enum! {
    /// `equine.Variant` (`horse/variant`).
    HorseVariant {
        WHITE => (0, "white"), CREAMY => (1, "creamy"), CHESTNUT => (2, "chestnut"),
        BROWN => (3, "brown"), BLACK => (4, "black"), GRAY => (5, "gray"),
        DARK_BROWN => (6, "dark_brown")
    }
}
id_enum! {
    /// `Llama.Variant` (`llama/variant`).
    LlamaVariant {
        CREAMY => (0, "creamy"), WHITE => (1, "white"), BROWN => (2, "brown"),
        GRAY => (3, "gray")
    }
}

// ---- registry-варианты (holderRegistry кодек = VarInt id) ----

macro_rules! registry_variant {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        /// Registry-запись варианта: record-поля (assets,
        /// spawn conditions, model) портируются с entity-стадией;
        /// wire-идентичность — registry id (`holderRegistry`).
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name {
            /// Registry-ключ.
            pub id: Identifier,
        }

        impl $name {
            /// `new`.
            pub fn new(id: Identifier) -> Self {
                Self { id }
            }

            /// `STREAM_CODEC` — `ByteBufCodecs.holderRegistry`.
            pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<$name>> + Send + Sync + 'static {
                holder_registry_stream_codec()
            }
        }
    };
}

registry_variant!(VillagerType, "`VillagerType` (`villager/variant`).");
registry_variant!(WolfVariant, "`WolfVariant` (`wolf/variant`).");
registry_variant!(WolfSoundVariant, "`WolfSoundVariant` (`wolf/sound_variant`).");
registry_variant!(PigVariant, "`PigVariant` (`pig/variant`).");
registry_variant!(CowVariant, "`CowVariant` (`cow/variant`).");
registry_variant!(ChickenVariant, "`ChickenVariant` (`chicken/variant`).");
registry_variant!(ZombieNautilusVariant, "`ZombieNautilusVariant` (`zombie_nautilus/variant`).");
registry_variant!(FrogVariant, "`FrogVariant` (`frog/variant`).");
registry_variant!(PaintingVariant, "`PaintingVariant` (`painting/variant`).");
registry_variant!(CatVariant, "`CatVariant` (`cat/variant`).");
