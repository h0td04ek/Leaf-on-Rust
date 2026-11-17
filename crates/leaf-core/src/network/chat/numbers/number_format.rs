//! Перенос `net.minecraft.network.chat.numbers.NumberFormat` +
//! `NumberFormatTypes` — форматы отображения чисел (очки, bossbar, таб).
//!
//! Wire-формат `NumberFormatTypes.STREAM_CODEC`:
//! `ByteBufCodecs.registry(NUMBER_FORMAT_TYPE).dispatch(...)` —
//! ResourceLocation типа (`minecraft:blank|styled|fixed`) + payload.

use crate::network::chat::component::Component;
use crate::network::chat::component_json::component_stream_codec;
use crate::network::chat::style::style_stream_codec;
use crate::network::chat::style::Style;
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// Идентификаторы типов `NumberFormatType` в реестре `NUMBER_FORMAT_TYPE`.
pub mod number_format_type_ids {
    /// `blank`.
    pub const BLANK: &str = "blank";
    /// `styled`.
    pub const STYLED: &str = "styled";
    /// `fixed`.
    pub const FIXED: &str = "fixed";
}

/// `NumberFormat` — формат числа. В Java — интерфейс с тремя
/// реализациями (`BlankFormat`, `StyledFormat`, `FixedFormat`);
/// здесь — enum, диспетчеризация по типу сохранена в кодеке.
#[derive(Clone, Debug, PartialEq)]
pub enum NumberFormat {
    /// `BlankFormat` — пустой компонент.
    Blank,
    /// `StyledFormat` — число со стилем.
    Styled(Style),
    /// `FixedFormat` — фиксированный компонент вместо числа.
    Fixed(Component),
}

impl NumberFormat {
    /// `BlankFormat.INSTANCE`.
    pub const BLANK: NumberFormat = NumberFormat::Blank;

    /// `StyledFormat.NO_STYLE`.
    pub fn styled_no_style() -> NumberFormat {
        NumberFormat::Styled(Style::empty())
    }

    /// `StyledFormat.SIDEBAR_DEFAULT` — красный стиль.
    pub fn styled_sidebar_default() -> NumberFormat {
        NumberFormat::Styled(Style::empty().with_color_formatting(
            crate::network::chat::chat_formatting::ChatFormatting::Red,
        ))
    }

    /// `StyledFormat.PLAYER_LIST_DEFAULT` — жёлтый стиль.
    pub fn styled_player_list_default() -> NumberFormat {
        NumberFormat::Styled(Style::empty().with_color_formatting(
            crate::network::chat::chat_formatting::ChatFormatting::Yellow,
        ))
    }

    /// `type()` — идентификатор типа для dispatch-кодека.
    pub fn type_id(&self) -> &'static str {
        match self {
            NumberFormat::Blank => number_format_type_ids::BLANK,
            NumberFormat::Styled(_) => number_format_type_ids::STYLED,
            NumberFormat::Fixed(_) => number_format_type_ids::FIXED,
        }
    }

    /// `format(int)` — отформатировать число в компонент.
    pub fn format(&self, number: i32) -> Component {
        match self {
            NumberFormat::Blank => Component::empty(),
            NumberFormat::Styled(style) => {
                let mut c = Component::literal(number.to_string());
                c.with_style(style.clone());
                c
            }
            NumberFormat::Fixed(value) => value.copy(),
        }
    }
}

fn write_type_id(buf: &mut FriendlyByteBuf, type_id: &str) {
    buf.write_identifier(&Identifier::with_default_namespace(type_id).expect("number format type id"))
        .expect("number format type id");
}

fn read_type_id(buf: &mut FriendlyByteBuf) -> String {
    let id = buf.read_identifier().expect("number format type id");
    id.to_string()
}

/// `NumberFormatTypes.STREAM_CODEC`.
pub fn number_format_stream_codec() -> impl StreamCodec<FriendlyByteBuf, NumberFormat> {
    FnCodec::new(
        |buf: &mut FriendlyByteBuf, value: &NumberFormat| match value {
            NumberFormat::Blank => write_type_id(buf, number_format_type_ids::BLANK),
            NumberFormat::Styled(style) => {
                write_type_id(buf, number_format_type_ids::STYLED);
                style_stream_codec().encode(buf, style);
            }
            NumberFormat::Fixed(component) => {
                write_type_id(buf, number_format_type_ids::FIXED);
                component_stream_codec().encode(buf, component);
            }
        },
        |buf: &mut FriendlyByteBuf| match read_type_id(buf).as_str() {
            number_format_type_ids::BLANK => NumberFormat::Blank,
            number_format_type_ids::STYLED => {
                NumberFormat::Styled(style_stream_codec().decode(buf))
            }
            number_format_type_ids::FIXED => {
                NumberFormat::Fixed(component_stream_codec().decode(buf))
            }
            other => panic!("Unknown number format type: {other}"),
        },
    )
}

/// `NumberFormatTypes.OPTIONAL_STREAM_CODEC`.
pub fn optional_number_format_stream_codec()
-> impl StreamCodec<FriendlyByteBuf, Option<NumberFormat>> {
    FnCodec::new(
        |buf: &mut FriendlyByteBuf, value: &Option<NumberFormat>| {
            match value {
                Some(v) => {
                    buf.write_bool(true);
                    number_format_stream_codec().encode(buf, v);
                }
                None => {
                    buf.write_bool(false);
                }
            }
        },
        |buf: &mut FriendlyByteBuf| {
            if buf.read_bool() {
                Some(number_format_stream_codec().decode(buf))
            } else {
                None
            }
        },
    )
}
