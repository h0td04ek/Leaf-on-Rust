//! Перенос `net.minecraft.util.TriState`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `TriState` — TRUE / FALSE / DEFAULT.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TriState {
    /// `TRUE`.
    True,
    /// `FALSE`.
    False,
    /// `DEFAULT`.
    Default,
}

impl TriState {
    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            TriState::True => "true",
            TriState::False => "false",
            TriState::Default => "default",
        }
    }

    /// `byName`-эквивалент.
    pub fn by_name(name: &str) -> Option<TriState> {
        match name {
            "true" => Some(TriState::True),
            "false" => Some(TriState::False),
            "default" => Some(TriState::Default),
            _ => None,
        }
    }

    /// `from(boolean)`.
    pub fn from(value: bool) -> TriState {
        if value { TriState::True } else { TriState::False }
    }

    /// `toBoolean(defaultValue)`.
    pub fn to_boolean(&self, default_value: bool) -> bool {
        match self {
            TriState::True => true,
            TriState::False => false,
            TriState::Default => default_value,
        }
    }

    /// StreamCodec — VarInt id (порядок объявления).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, TriState> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, v: &TriState| {
                buf.write_var_int(match v {
                    TriState::True => 0,
                    TriState::False => 1,
                    TriState::Default => 2,
                });
            },
            |buf: &mut FriendlyByteBuf| match buf.read_var_int() {
                0 => TriState::True,
                1 => TriState::False,
                _ => TriState::Default,
            },
        )
    }
}
