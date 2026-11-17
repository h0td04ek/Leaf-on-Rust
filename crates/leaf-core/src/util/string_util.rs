//! Перенос `net.minecraft.util.StringUtil`.
//! Leaf-специфичные проверки (VanillaUsernameCheck, playedPlayers,
//! useUsernameRegex) — конфигурация Leaf-сервера; здесь vanilla-логика
//! и чистые функции. Paper `isReasonablePlayerName` портирован без
//! обращения к серверу (bypassCheck=false по умолчанию).

use crate::util::mth;

/// `formatTickDuration(ticks, ticksPerSecond)`.
pub fn format_tick_duration(ticks: i32, ticks_per_second: f32) -> String {
    let mut floor = mth::floor_f32(ticks as f32 / ticks_per_second);
    let i = floor / 60;
    floor %= 60;
    let i1 = i / 60;
    let i = i % 60;
    if i1 > 0 {
        format!("{:02}:{:02}:{:02}", i1, i, floor)
    } else {
        format!("{:02}:{:02}", i, floor)
    }
}

/// `stripColor` — удаляет §-коды цвета/формата.
pub fn strip_color(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{a7}' {
            // §[0-9A-FK-OR] — один символ после §
            if let Some(&next) = chars.peek() {
                if next.is_ascii_hexdigit()
                    || ('K'..='O').contains(&next.to_ascii_uppercase())
                    || next == 'r' || next == 'R'
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

/// `isNullOrEmpty`.
pub fn is_null_or_empty(string: Option<&str>) -> bool {
    string.is_none_or(|s| s.is_empty())
}

/// `truncateStringIfNecessary`.
pub fn truncate_string_if_necessary(string: &str, max_size: usize, add_ellipsis: bool) -> String {
    // Java считает по UTF-16 code units; Rust — по bytes. Используем
    // chars() как приближение (для ASCII идентично).
    let len = string.chars().count();
    if len <= max_size {
        return string.to_string();
    }
    if add_ellipsis && max_size > 3 {
        let truncated: String = string.chars().take(max_size - 3).collect();
        format!("{}...", truncated)
    } else {
        string.chars().take(max_size).collect()
    }
}

/// `lineCount` — число строк по \r\n и \v.
pub fn line_count(string: &str) -> usize {
    if string.is_empty() {
        return 0;
    }
    let mut count = 1;
    let mut chars = string.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            count += 1;
        } else if c == '\u{b}' {
            count += 1;
        }
    }
    count
}

/// `endsWithNewLine`.
pub fn ends_with_new_line(string: &str) -> bool {
    string.ends_with("\r\n") || string.ends_with('\u{b}')
}

/// `trimChatMessage`.
pub fn trim_chat_message(string: &str) -> String {
    truncate_string_if_necessary(string, 256, false)
}

/// `isAllowedChatCharacter(codePoint)` — != 167, >= 32, != 127.
pub fn is_allowed_chat_character(code_point: u32) -> bool {
    code_point != 167 && code_point >= 32 && code_point != 127
}

/// `isValidPlayerNameVanilla` — <=16, без управляющих/пробельных в начале?
/// Java: length <= 16 && chars().filter(i -> i <= 32 || i >= 127).findAny().isEmpty()
pub fn is_valid_player_name_vanilla(player_name: &str) -> bool {
    player_name.chars().count() <= 16
        && player_name.chars().all(|c| {
            let cp = c as u32;
            !(cp <= 32 || cp >= 127)
        })
}

/// `isValidPlayerName(playerName, bypassCheck)`.
pub fn is_valid_player_name(player_name: &str, bypass_check: bool) -> bool {
    if bypass_check {
        player_name.chars().count() <= 16
    } else {
        is_valid_player_name_vanilla(player_name)
    }
}

/// `filterText` — оставляет только разрешённые chat-символы.
pub fn filter_text(text: &str, allow_line_breaks: bool) -> String {
    text.chars()
        .filter(|&c| is_allowed_chat_character(c as u32) || (allow_line_breaks && c == '\n'))
        .collect()
}

/// Paper `isReasonablePlayerName` — [a-zA-Z0-9_.], <=16, непустое.
/// (Без интеграции с сервером — отдельный параметр `played_before`.)
pub fn is_reasonable_player_name(name: &str, played_before: bool) -> bool {
    if played_before {
        return true;
    }
    if name.is_empty() || name.chars().count() > 16 {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
}

/// `isWhitespace`.
pub fn is_whitespace(character: char) -> bool {
    character.is_whitespace()
}

/// `isBlank`.
pub fn is_blank(string: Option<&str>) -> bool {
    match string {
        None => true,
        Some(s) => s.is_empty() || s.chars().all(is_whitespace),
    }
}
