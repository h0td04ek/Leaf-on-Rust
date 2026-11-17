//! Перенос `net.minecraft.network.chat.TextColor`.

use crate::network::chat::chat_formatting::ChatFormatting;
use std::fmt;
use std::sync::OnceLock;

/// `TextColor` — цвет текста: именованный (из ChatFormatting) или RGB `#RRGGBB`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextColor {
    value: u32,
    pub name: Option<String>,
    pub format: Option<ChatFormatting>,
}

impl TextColor {
    /// `NAMED_COLORS` / `LEGACY_FORMAT_TO_COLOR`.
    fn legacy_map() -> &'static std::collections::HashMap<ChatFormatting, TextColor> {
        static MAP: OnceLock<std::collections::HashMap<ChatFormatting, TextColor>> = OnceLock::new();
        MAP.get_or_init(|| {
            let mut m = std::collections::HashMap::new();
            for f in ChatFormatting::VALUES {
                if f.is_color() {
                    m.insert(f, TextColor::from_legacy_format_unchecked(f));
                }
            }
            m
        })
    }

    fn named_map() -> &'static std::collections::HashMap<String, TextColor> {
        static MAP: OnceLock<std::collections::HashMap<String, TextColor>> = OnceLock::new();
        MAP.get_or_init(|| {
            Self::legacy_map()
                .values()
                .filter_map(|c| c.name.clone().map(|n| (n, c.clone())))
                .collect()
        })
    }

    fn from_legacy_format_unchecked(format: ChatFormatting) -> TextColor {
        TextColor {
            value: format.get_color().unwrap_or(0) & 0xFFFFFF,
            name: Some(format.get_serialized_name()),
            format: Some(format),
        }
    }

    /// `fromRgb(int)`.
    pub fn from_rgb(color: u32) -> TextColor {
        TextColor {
            value: color & 0xFFFFFF,
            name: None,
            format: None,
        }
    }

    /// `fromLegacyFormat(ChatFormatting)`.
    pub fn from_legacy_format(formatting: ChatFormatting) -> Option<TextColor> {
        Self::legacy_map().get(&formatting).cloned()
    }

    /// `getValue()`.
    pub fn get_value(&self) -> u32 {
        self.value
    }

    /// `serialize()`.
    pub fn serialize(&self) -> String {
        match &self.name {
            Some(n) => n.clone(),
            None => self.format_value(),
        }
    }

    fn format_value(&self) -> String {
        const HEX: &[u8; 16] = b"0123456789ABCDEF";
        let v = self.value as usize;
        let mut s = String::with_capacity(7);
        s.push('#');
        for shift in [20usize, 16, 12, 8, 4, 0] {
            s.push(HEX[(v >> shift) & 0xF] as char);
        }
        s
    }

    /// `parseColor(String)` → Result<TextColor, String>.
    /// Принимает `#RRGGBB` или именованный цвет.
    pub fn parse_color(color: &str) -> Result<TextColor, String> {
        if !color.is_empty() && color.starts_with('#') {
            let parsed = u32::from_str_radix(&color[1..], 16)
                .map_err(|_| format!("Invalid color value: {}", color))?;
            if parsed <= 0xFFFFFF {
                Ok(TextColor::from_rgb(parsed))
            } else {
                Err(format!("Color value out of range: {}", color))
            }
        } else {
            Self::named_map()
                .get(color)
                .cloned()
                .ok_or_else(|| format!("Invalid color name: {}", color))
        }
    }
}

impl fmt::Display for TextColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.serialize())
    }
}
