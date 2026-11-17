//! Перенос `net.minecraft.world.attribute.BedRule`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::chat::component::Component;

/// `BedRule` — правила сна/спавна (биом-атрибут `gameplay/bed_rule`).
#[derive(Clone, Debug, PartialEq)]
pub struct BedRule {
    /// `canSleep`.
    pub can_sleep: Rule,
    /// `canSetSpawn`.
    pub can_set_spawn: Rule,
    /// `explodes`.
    pub explodes: bool,
    /// `errorMessage`.
    pub error_message: Option<Component>,
}

impl BedRule {
    /// `CAN_SLEEP_WHEN_DARK`.
    pub fn can_sleep_when_dark() -> Self {
        Self {
            can_sleep: Rule::WhenDark,
            can_set_spawn: Rule::Always,
            explodes: false,
            error_message: Some(Component::translatable("block.minecraft.bed.no_sleep", Vec::new())),
        }
    }

    /// `EXPLODES`.
    pub fn explodes_rule() -> Self {
        Self {
            can_sleep: Rule::Never,
            can_set_spawn: Rule::Never,
            explodes: true,
            error_message: None,
        }
    }
}

/// `BedRule.Rule`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rule {
    /// `always`.
    Always,
    /// `when_dark`.
    WhenDark,
    /// `never`.
    Never,
}

impl Rule {
    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            Rule::Always => "always",
            Rule::WhenDark => "when_dark",
            Rule::Never => "never",
        }
    }

    /// `byName`-эквивалент.
    pub fn by_name(name: &str) -> Option<Rule> {
        match name {
            "always" => Some(Rule::Always),
            "when_dark" => Some(Rule::WhenDark),
            "never" => Some(Rule::Never),
            _ => None,
        }
    }

    /// StreamCodec — UTF8 name.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Rule> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, v: &Rule| { buf.write_utf_n(v.get_serialized_name(), 16).unwrap(); },
            |buf: &mut FriendlyByteBuf| {
                let s = buf.read_utf_n(16).unwrap();
                Rule::by_name(&s).expect("unknown bed rule")
            },
        )
    }
}
