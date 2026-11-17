//! Перенос `net.minecraft.network.chat.Style` (без DFU-кодека).

use crate::network::chat::chat_formatting::ChatFormatting;
use crate::network::chat::click_event::ClickEvent;
use crate::network::chat::font_description::FontDescription;
use crate::network::chat::hover_event::HoverEvent;
use crate::network::chat::text_color::TextColor;
use std::fmt;

/// `Style` — иммутабельный стиль текста.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Style {
    pub color: Option<TextColor>,
    pub shadow_color: Option<u32>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub obfuscated: Option<bool>,
    pub click_event: Option<ClickEvent>,
    pub hover_event: Option<HoverEvent>,
    pub insertion: Option<String>,
    pub font: Option<FontDescription>,
}

impl Style {
    /// `Style.EMPTY`.
    pub fn empty() -> Style {
        Style::default()
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self == &Style::empty()
    }

    /// `getColor`.
    pub fn get_color(&self) -> Option<&TextColor> {
        self.color.as_ref()
    }

    /// `getShadowColor`.
    pub fn get_shadow_color(&self) -> Option<u32> {
        self.shadow_color
    }

    /// `getClickEvent` / `getHoverEvent` / `getInsertion` / `getFont`.
    pub fn get_click_event(&self) -> Option<&ClickEvent> {
        self.click_event.as_ref()
    }
    pub fn get_hover_event(&self) -> Option<&HoverEvent> {
        self.hover_event.as_ref()
    }
    pub fn get_insertion(&self) -> Option<&str> {
        self.insertion.as_deref()
    }
    pub fn get_font(&self) -> Option<&FontDescription> {
        self.font.as_ref()
    }

    /// Булевы геттеры.
    pub fn is_bold(&self) -> Option<bool> {
        self.bold
    }
    pub fn is_italic(&self) -> Option<bool> {
        self.italic
    }
    pub fn is_underlined(&self) -> Option<bool> {
        self.underlined
    }
    pub fn is_strikethrough(&self) -> Option<bool> {
        self.strikethrough
    }
    pub fn is_obfuscated(&self) -> Option<bool> {
        self.obfuscated
    }

    fn with<F>(base: &Style, f: F) -> Style
    where
        F: FnOnce(Style) -> Style,
    {
        let mut s = base.clone();
        f(std::mem::take(&mut s));
        // checkEmptyAfterChange: если всё обнулилось — вернуть EMPTY
        if s.is_empty() { Style::empty() } else { s }
    }

    /// `withColor(TextColor)`.
    pub fn with_color(&self, color: Option<TextColor>) -> Style {
        Self::with(self, |mut s| {
            s.color = color;
            s
        })
    }

    /// `withColor(ChatFormatting)`.
    pub fn with_color_formatting(&self, formatting: ChatFormatting) -> Style {
        self.with_color(TextColor::from_legacy_format(formatting))
    }

    /// `withColor(int)` — RGB.
    pub fn with_color_rgb(&self, color: u32) -> Style {
        self.with_color(Some(TextColor::from_rgb(color)))
    }

    /// `withShadowColor(int)`.
    pub fn with_shadow_color(&self, color: u32) -> Style {
        Self::with(self, |mut s| {
            s.shadow_color = Some(color);
            s
        })
    }

    /// `withoutShadow`.
    pub fn without_shadow(&self) -> Style {
        Self::with(self, |mut s| {
            s.shadow_color = None;
            s
        })
    }

    /// `withBold(Optional<Boolean>)` и т.п.
    pub fn with_bold(&self, bold: Option<bool>) -> Style {
        Self::with(self, |mut s| {
            s.bold = bold;
            s
        })
    }
    pub fn with_italic(&self, italic: Option<bool>) -> Style {
        Self::with(self, |mut s| {
            s.italic = italic;
            s
        })
    }
    pub fn with_underlined(&self, underlined: Option<bool>) -> Style {
        Self::with(self, |mut s| {
            s.underlined = underlined;
            s
        })
    }
    pub fn with_strikethrough(&self, strikethrough: Option<bool>) -> Style {
        Self::with(self, |mut s| {
            s.strikethrough = strikethrough;
            s
        })
    }
    pub fn with_obfuscated(&self, obfuscated: Option<bool>) -> Style {
        Self::with(self, |mut s| {
            s.obfuscated = obfuscated;
            s
        })
    }
    pub fn with_click_event(&self, click_event: Option<ClickEvent>) -> Style {
        Self::with(self, |mut s| {
            s.click_event = click_event;
            s
        })
    }
    pub fn with_hover_event(&self, hover_event: Option<HoverEvent>) -> Style {
        Self::with(self, |mut s| {
            s.hover_event = hover_event;
            s
        })
    }
    pub fn with_insertion(&self, insertion: Option<String>) -> Style {
        Self::with(self, |mut s| {
            s.insertion = insertion;
            s
        })
    }
    pub fn with_font(&self, font: Option<FontDescription>) -> Style {
        Self::with(self, |mut s| {
            s.font = font;
            s
        })
    }

    /// `applyFormat(ChatFormatting)`.
    pub fn apply_format(&self, formatting: ChatFormatting) -> Style {
        let mut color = self.color.clone();
        let mut bold = self.bold;
        let mut italic = self.italic;
        let mut strikethrough = self.strikethrough;
        let mut underlined = self.underlined;
        let mut obfuscated = self.obfuscated;
        match formatting {
            ChatFormatting::Obfuscated => obfuscated = Some(true),
            ChatFormatting::Bold => bold = Some(true),
            ChatFormatting::Strikethrough => strikethrough = Some(true),
            ChatFormatting::Underline => underlined = Some(true),
            ChatFormatting::Italic => italic = Some(true),
            ChatFormatting::Reset => return Style::empty(),
            _ => color = TextColor::from_legacy_format(formatting),
        }
        Style {
            color,
            shadow_color: self.shadow_color,
            bold,
            italic,
            underlined,
            strikethrough,
            obfuscated,
            click_event: self.click_event.clone(),
            hover_event: self.hover_event.clone(),
            insertion: self.insertion.clone(),
            font: self.font.clone(),
        }
    }

    /// `applyLegacyFormat` — как applyFormat, но цвет также сбрасывает форматы.
    pub fn apply_legacy_format(&self, formatting: ChatFormatting) -> Style {
        let mut color = self.color.clone();
        let mut bold = self.bold;
        let mut italic = self.italic;
        let mut strikethrough = self.strikethrough;
        let mut underlined = self.underlined;
        let mut obfuscated = self.obfuscated;
        match formatting {
            ChatFormatting::Obfuscated => obfuscated = Some(true),
            ChatFormatting::Bold => bold = Some(true),
            ChatFormatting::Strikethrough => strikethrough = Some(true),
            ChatFormatting::Underline => underlined = Some(true),
            ChatFormatting::Italic => italic = Some(true),
            ChatFormatting::Reset => return Style::empty(),
            _ => {
                obfuscated = Some(false);
                bold = Some(false);
                strikethrough = Some(false);
                underlined = Some(false);
                italic = Some(false);
                color = TextColor::from_legacy_format(formatting);
            }
        }
        Style {
            color,
            shadow_color: self.shadow_color,
            bold,
            italic,
            underlined,
            strikethrough,
            obfuscated,
            click_event: self.click_event.clone(),
            hover_event: self.hover_event.clone(),
            insertion: self.insertion.clone(),
            font: self.font.clone(),
        }
    }

    /// `applyFormats`.
    pub fn apply_formats(&self, formats: &[ChatFormatting]) -> Style {
        let mut style = self.clone();
        for &f in formats {
            style = style.apply_format(f);
        }
        style
    }

    /// `applyTo(Style)` — наложить self поверх style (self приоритетнее).
    pub fn apply_to(&self, style: &Style) -> Style {
        if self.is_empty() {
            return style.clone();
        }
        if style.is_empty() {
            return self.clone();
        }
        Style {
            color: self.color.clone().or_else(|| style.color.clone()),
            shadow_color: self.shadow_color.or(style.shadow_color),
            bold: self.bold.or(style.bold),
            italic: self.italic.or(style.italic),
            underlined: self.underlined.or(style.underlined),
            strikethrough: self.strikethrough.or(style.strikethrough),
            obfuscated: self.obfuscated.or(style.obfuscated),
            click_event: self.click_event.clone().or_else(|| style.click_event.clone()),
            hover_event: self.hover_event.clone().or_else(|| style.hover_event.clone()),
            insertion: self.insertion.clone().or_else(|| style.insertion.clone()),
            font: self.font.clone().or_else(|| style.font.clone()),
        }
    }
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        let mut first = true;
        let mut sep = |f: &mut fmt::Formatter<'_>| -> fmt::Result {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            Ok(())
        };
        if self.color.is_some() {
            sep(f)?;
            write!(f, "color={}", self.color.as_ref().unwrap())?;
        }
        if self.shadow_color.is_some() {
            sep(f)?;
            write!(f, "shadow_color={:#x}", self.shadow_color.unwrap())?;
        }
        for (name, v) in [
            ("bold", self.bold),
            ("italic", self.italic),
            ("underlined", self.underlined),
            ("strikethrough", self.strikethrough),
            ("obfuscated", self.obfuscated),
        ] {
            if let Some(v) = v {
                sep(f)?;
                write!(f, "{}={}", name, v)?;
            }
        }
        if self.click_event.is_some() {
            sep(f)?;
            write!(f, "click_event={:?}", self.click_event.as_ref().unwrap())?;
        }
        if self.hover_event.is_some() {
            sep(f)?;
            write!(f, "hover_event={:?}", self.hover_event.as_ref().unwrap())?;
        }
        if self.insertion.is_some() {
            sep(f)?;
            write!(f, "insertion={:?}", self.insertion.as_ref().unwrap())?;
        }
        if self.font.is_some() {
            sep(f)?;
            write!(f, "font={:?}", self.font.as_ref().unwrap())?;
        }
        write!(f, "}}")
    }
}

// ------------------------------------------------------------ stream codec

/// `Style.Serializer.TRUSTED_STREAM_CODEC` — Style как NBT tag
/// (codec → NbtOps; JSON-поля конвертируются через `nbt::json`).
pub fn style_stream_codec()
-> impl crate::network::codec::stream_codec::StreamCodec<crate::network::friendly_byte_buf::FriendlyByteBuf, Style> {
    crate::network::codec::stream_codec::FnCodec::new(
        |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf, style: &Style| {
            let json = crate::network::chat::component_json::style_to_json(style);
            let tag = crate::nbt::json::tag_from_json(&json).expect("style nbt encode");
            buf.write_nbt(Some(&tag)).expect("style nbt write");
        },
        |buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf| {
            let tag = buf.read_nbt().expect("invalid style nbt").unwrap_or(crate::nbt::tag::Tag::End);
            let json = crate::nbt::json::json_from_tag(&tag).expect("style nbt decode");
            crate::network::chat::component_json::style_from_json(&json).expect("invalid style")
        },
    )
}
