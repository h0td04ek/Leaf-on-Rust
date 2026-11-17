//! Перенос `net.minecraft.ChatFormatting`.

use crate::util::string_representable::StringRepresentable;
use std::fmt;

/// `ChatFormatting` — цвет/формат с кодом `§`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChatFormatting {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
    Obfuscated,
    Bold,
    Strikethrough,
    Underline,
    Italic,
    Reset,
}

impl ChatFormatting {
    pub const PREFIX_CODE: char = '§';

    /// Все значения в порядке объявления.
    pub const VALUES: [ChatFormatting; 22] = [
        ChatFormatting::Black,
        ChatFormatting::DarkBlue,
        ChatFormatting::DarkGreen,
        ChatFormatting::DarkAqua,
        ChatFormatting::DarkRed,
        ChatFormatting::DarkPurple,
        ChatFormatting::Gold,
        ChatFormatting::Gray,
        ChatFormatting::DarkGray,
        ChatFormatting::Blue,
        ChatFormatting::Green,
        ChatFormatting::Aqua,
        ChatFormatting::Red,
        ChatFormatting::LightPurple,
        ChatFormatting::Yellow,
        ChatFormatting::White,
        ChatFormatting::Obfuscated,
        ChatFormatting::Bold,
        ChatFormatting::Strikethrough,
        ChatFormatting::Underline,
        ChatFormatting::Italic,
        ChatFormatting::Reset,
    ];

    /// `name` — UPPER_SNAKE имя.
    pub fn name(&self) -> &'static str {
        match self {
            ChatFormatting::Black => "BLACK",
            ChatFormatting::DarkBlue => "DARK_BLUE",
            ChatFormatting::DarkGreen => "DARK_GREEN",
            ChatFormatting::DarkAqua => "DARK_AQUA",
            ChatFormatting::DarkRed => "DARK_RED",
            ChatFormatting::DarkPurple => "DARK_PURPLE",
            ChatFormatting::Gold => "GOLD",
            ChatFormatting::Gray => "GRAY",
            ChatFormatting::DarkGray => "DARK_GRAY",
            ChatFormatting::Blue => "BLUE",
            ChatFormatting::Green => "GREEN",
            ChatFormatting::Aqua => "AQUA",
            ChatFormatting::Red => "RED",
            ChatFormatting::LightPurple => "LIGHT_PURPLE",
            ChatFormatting::Yellow => "YELLOW",
            ChatFormatting::White => "WHITE",
            ChatFormatting::Obfuscated => "OBFUSCATED",
            ChatFormatting::Bold => "BOLD",
            ChatFormatting::Strikethrough => "STRIKETHROUGH",
            ChatFormatting::Underline => "UNDERLINE",
            ChatFormatting::Italic => "ITALIC",
            ChatFormatting::Reset => "RESET",
        }
    }

    /// `getSerializedName` — имя в нижнем регистре.
    pub fn get_serialized_name(&self) -> String {
        self.name().to_lowercase()
    }

    /// `code` — символ после §.
    pub fn code(&self) -> char {
        match self {
            ChatFormatting::Black => '0',
            ChatFormatting::DarkBlue => '1',
            ChatFormatting::DarkGreen => '2',
            ChatFormatting::DarkAqua => '3',
            ChatFormatting::DarkRed => '4',
            ChatFormatting::DarkPurple => '5',
            ChatFormatting::Gold => '6',
            ChatFormatting::Gray => '7',
            ChatFormatting::DarkGray => '8',
            ChatFormatting::Blue => '9',
            ChatFormatting::Green => 'a',
            ChatFormatting::Aqua => 'b',
            ChatFormatting::Red => 'c',
            ChatFormatting::LightPurple => 'd',
            ChatFormatting::Yellow => 'e',
            ChatFormatting::White => 'f',
            ChatFormatting::Obfuscated => 'k',
            ChatFormatting::Bold => 'l',
            ChatFormatting::Strikethrough => 'm',
            ChatFormatting::Underline => 'n',
            ChatFormatting::Italic => 'o',
            ChatFormatting::Reset => 'r',
        }
    }

    /// `isFormat` — true для не-цветовых форматов.
    pub fn is_format(&self) -> bool {
        self.is_color() == false && *self != ChatFormatting::Reset
    }

    /// `isColor`.
    pub fn is_color(&self) -> bool {
        self.get_color().is_some()
    }

    /// `getColor` — RGB-значение для цветовых форматов.
    pub fn get_color(&self) -> Option<u32> {
        match self {
            ChatFormatting::Black => Some(0),
            ChatFormatting::DarkBlue => Some(170),
            ChatFormatting::DarkGreen => Some(43520),
            ChatFormatting::DarkAqua => Some(43690),
            ChatFormatting::DarkRed => Some(11141120),
            ChatFormatting::DarkPurple => Some(11141290),
            ChatFormatting::Gold => Some(16755200),
            ChatFormatting::Gray => Some(11184810),
            ChatFormatting::DarkGray => Some(5592405),
            ChatFormatting::Blue => Some(5592575),
            ChatFormatting::Green => Some(5635925),
            ChatFormatting::Aqua => Some(5636095),
            ChatFormatting::Red => Some(16733525),
            ChatFormatting::LightPurple => Some(16733695),
            ChatFormatting::Yellow => Some(16777045),
            ChatFormatting::White => Some(16777215),
            _ => None,
        }
    }

    /// `getByCode(char)`.
    pub fn get_by_code(code: char) -> Option<ChatFormatting> {
        Self::VALUES.iter().copied().find(|f| f.code() == code)
    }

    /// `getByName(String)` — case-insensitive, без не-букв.
    pub fn get_by_name(name: &str) -> Option<ChatFormatting> {
        let cleaned: String = name
            .to_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
            .collect();
        Self::VALUES.iter().copied().find(|f| {
            f.name()
                .to_lowercase()
                .chars()
                .filter(|c| c.is_ascii_alphabetic())
                .eq(cleaned.chars())
        })
    }

    /// `stripFormatting(String)` — убрать §-коды.
    pub fn strip_formatting(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();
        while let Some(c) = chars.next() {
            if c == Self::PREFIX_CODE {
                if let Some(&next) = chars.peek() {
                    if next.is_ascii_alphanumeric() || next == 'k' || next == 'l' || next == 'm'
                        || next == 'n' || next == 'o' || next == 'r'
                    {
                        chars.next();
                        continue;
                    }
                }
            }
            out.push(c);
        }
        out
    }
}

impl StringRepresentable for ChatFormatting {
    fn get_serialized_name(&self) -> String {
        ChatFormatting::get_serialized_name(self)
    }
}

impl fmt::Display for ChatFormatting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name().to_lowercase())
    }
}
