//! Перенос `net.minecraft.world.Difficulty`.

use crate::network::chat::component::Component;

/// `Difficulty` — enum PEACEFUL/EASY/NORMAL/HARD.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Difficulty {
    /// `PEACEFUL`.
    Peaceful = 0,
    /// `EASY`.
    Easy = 1,
    /// `NORMAL`.
    Normal = 2,
    /// `HARD`.
    Hard = 3,
}

/// Все значения (порядок как в Java `values()`).
pub const DIFFICULTIES: [Difficulty; 4] = [
    Difficulty::Peaceful,
    Difficulty::Easy,
    Difficulty::Normal,
    Difficulty::Hard,
];

impl Difficulty {
    /// `values()`.
    pub fn values() -> &'static [Difficulty] {
        &DIFFICULTIES
    }

    /// `getId`.
    pub fn get_id(&self) -> i32 {
        *self as i32
    }

    /// `byId` — `ByIdMap.continuous(..., WRAP)`.
    pub fn by_id(id: i32) -> Difficulty {
        let n = DIFFICULTIES.len() as i32;
        DIFFICULTIES[id.rem_euclid(n) as usize]
    }

    /// `byName` — по сериализованному имени.
    pub fn by_name(name: &str) -> Option<Difficulty> {
        DIFFICULTIES.iter().copied().find(|d| d.get_key() == name)
    }

    /// `getKey`/`getSerializedName`.
    pub fn get_key(&self) -> &'static str {
        match self {
            Difficulty::Peaceful => "peaceful",
            Difficulty::Easy => "easy",
            Difficulty::Normal => "normal",
            Difficulty::Hard => "hard",
        }
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        self.get_key()
    }

    /// `getDisplayName` — `options.difficulty.<key>`.
    pub fn get_display_name(&self) -> Component {
        Component::translatable(format!("options.difficulty.{}", self.get_key()), Vec::new())
    }

    /// `getInfo` — `options.difficulty.<key>.info`.
    pub fn get_info(&self) -> Component {
        Component::translatable(format!("options.difficulty.{}.info", self.get_key()), Vec::new())
    }

    /// `STREAM_CODEC` — idMapper (VarInt id).
    pub fn stream_codec() -> impl crate::network::codec::stream_codec::StreamCodec<
        crate::network::friendly_byte_buf::FriendlyByteBuf,
        Difficulty,
    > {
        crate::network::codec::stream_codec::FnCodec::new(
            |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf, d: &Difficulty| {
                buf.write_var_int(d.get_id());
            },
            |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf| {
                Difficulty::by_id(buf.read_var_int())
            },
        )
    }
}
