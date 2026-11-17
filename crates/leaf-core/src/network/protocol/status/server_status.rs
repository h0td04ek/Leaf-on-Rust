//! Перенос `net.minecraft.network.protocol.status.ServerStatus`
//! (JSON-кодек через `component_json` + serde_json — аналог Gson/DFU).

use super::name_and_id::NameAndId;
use crate::network::chat::common_components as CommonComponents;
use crate::network::chat::component::Component;
use crate::network::chat::component_json;
use crate::util::uuid::Uuid;
use serde_json::{json, Map, Value};
use std::sync::LazyLock;

/// `ServerStatus` — MOTD-ответ статус-протокола.
#[derive(Clone, Debug)]
pub struct ServerStatus {
    /// `description` — Component (default `CommonComponents.EMPTY`).
    pub description: Component,
    /// `players` — Optional.
    pub players: Option<Players>,
    /// `version` — Optional.
    pub version: Option<Version>,
    /// `favicon` — Optional.
    pub favicon: Option<Favicon>,
    /// `enforcesSecureChat` (Leaf: Configurable chat message signatures).
    pub enforces_secure_chat: bool,
    /// `preventsChatReports` (Leaf).
    pub prevents_chat_reports: bool,
}

impl ServerStatus {
    /// Пустой статус (все опциональные поля отсутствуют).
    pub fn empty() -> ServerStatus {
        ServerStatus {
            description: CommonComponents::empty(),
            players: None,
            version: None,
            favicon: None,
            enforces_secure_chat: false,
            prevents_chat_reports: false,
        }
    }

    /// JSON-кодек `ServerStatus.CODEC` (`lenientOptionalFieldOf` —
    /// отсутствующие поля → дефолты).
    pub fn to_json(&self) -> Value {
        let mut obj = Map::new();
        obj.insert("description".to_string(), component_json::component_to_json(&self.description));
        if let Some(players) = &self.players {
            obj.insert("players".to_string(), players_to_json(players));
        }
        if let Some(version) = &self.version {
            obj.insert("version".to_string(), version_to_json(version));
        }
        if let Some(favicon) = &self.favicon {
            obj.insert("favicon".to_string(), favicon_to_json(favicon));
        }
        obj.insert("enforcesSecureChat".to_string(), json!(self.enforces_secure_chat));
        obj.insert("preventsChatReports".to_string(), json!(self.prevents_chat_reports));
        Value::Object(obj)
    }

    /// `ServerStatus.CODEC.decode`.
    pub fn from_json(value: &Value) -> Result<ServerStatus, String> {
        let obj = value.as_object().ok_or("server status must be an object")?;
        let description = match obj.get("description") {
            Some(v) => component_json::component_from_json(v)?,
            None => CommonComponents::empty(),
        };
        let players = obj.get("players").map(players_from_json).transpose()?;
        let version = obj.get("version").map(version_from_json).transpose()?;
        let favicon = obj.get("favicon").map(favicon_from_json).transpose()?;
        let enforces_secure_chat = obj
            .get("enforcesSecureChat")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let prevents_chat_reports = obj
            .get("preventsChatReports")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        Ok(ServerStatus {
            description,
            players,
            version,
            favicon,
            enforces_secure_chat,
            prevents_chat_reports,
        })
    }
}

fn players_to_json(players: &Players) -> Value {
    json!({
        "max": players.max,
        "online": players.online,
        "sample": players.sample.iter().map(|p| json!({
            "id": p.id.to_string(),
            "name": p.name,
        })).collect::<Vec<_>>(),
    })
}

fn players_from_json(v: &Value) -> Result<Players, String> {
    let obj = v.as_object().ok_or("players must be an object")?;
    let max = obj.get("max").and_then(|v| v.as_i64()).ok_or("players.max required")? as i32;
    let online = obj.get("online").and_then(|v| v.as_i64()).ok_or("players.online required")? as i32;
    let sample = match obj.get("sample") {
        Some(Value::Array(arr)) => arr
            .iter()
            .map(|item| {
                let o = item.as_object().ok_or("sample entry must be object")?;
                let id_str = o.get("id").and_then(|v| v.as_str()).ok_or("sample.id required")?;
                let name = o.get("name").and_then(|v| v.as_str()).ok_or("sample.name required")?;
                let id = Uuid::from_string(id_str).ok_or("invalid uuid")?;
                Ok(NameAndId::new(id, name))
            })
            .collect::<Result<Vec<_>, String>>()?,
        _ => Vec::new(),
    };
    Ok(Players { max, online, sample })
}

fn version_to_json(version: &Version) -> Value {
    json!({ "name": version.name, "protocol": version.protocol })
}

fn version_from_json(v: &Value) -> Result<Version, String> {
    let obj = v.as_object().ok_or("version must be an object")?;
    Ok(Version {
        name: obj
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("version.name required")?
            .to_string(),
        protocol: obj
            .get("protocol")
            .and_then(|v| v.as_i64())
            .ok_or("version.protocol required")? as i32,
    })
}

const FAVICON_PREFIX: &str = "data:image/png;base64,";

fn favicon_to_json(favicon: &Favicon) -> Value {
    // base64 без паддинга переносов — как в Java (encode + UTF-8).
    use base64_encode::encode;
    let mut s = String::from(FAVICON_PREFIX);
    s.push_str(&encode(&favicon.icon_bytes));
    Value::String(s)
}

fn favicon_from_json(v: &Value) -> Result<Favicon, String> {
    let s = v.as_str().ok_or("favicon must be a string")?;
    let b64 = s
        .strip_prefix(FAVICON_PREFIX)
        .ok_or("Unknown format")?;
    let cleaned: String = b64.chars().filter(|c| *c != '\n').collect();
    let bytes = base64_decode(&cleaned).map_err(|_| "Malformed base64 server icon".to_string())?;
    Ok(Favicon { icon_bytes: bytes })
}

/// Минимальный base64 (RFC 4648) — encode/decode без внешних зависимостей.
mod base64_encode {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    pub fn encode(data: &[u8]) -> String {
        let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
        for chunk in data.chunks(3) {
            let b0 = chunk[0] as u32;
            let b1 = *chunk.get(1).unwrap_or(&0) as u32;
            let b2 = *chunk.get(2).unwrap_or(&0) as u32;
            let n = (b0 << 16) | (b1 << 8) | b2;
            out.push(ALPHABET[(n >> 18) as usize & 63] as char);
            out.push(ALPHABET[(n >> 12) as usize & 63] as char);
            out.push(if chunk.len() > 1 { ALPHABET[(n >> 6) as usize & 63] as char } else { '=' });
            out.push(if chunk.len() > 2 { ALPHABET[n as usize & 63] as char } else { '=' });
        }
        out
    }
}

fn base64_decode(s: &str) -> Result<Vec<u8>, ()> {
    fn val(c: u8) -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a' + 26) as u32),
            b'0'..=b'9' => Some((c - b'0' + 52) as u32),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let bytes = s.as_bytes();
    if bytes.len() % 4 != 0 {
        return Err(());
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let padding = chunk.iter().filter(|&&c| c == b'=').count();
        let mut n: u32 = 0;
        for (i, &c) in chunk.iter().enumerate() {
            if c == b'=' {
                continue;
            }
            n |= val(c).ok_or(())? << (18 - 6 * i);
        }
        out.push((n >> 16) as u8);
        if padding < 2 {
            out.push((n >> 8) as u8);
        }
        if padding < 1 {
            out.push(n as u8);
        }
    }
    Ok(out)
}

/// `ServerStatus.Favicon` — `data:image/png;base64,...`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Favicon {
    /// Сырые байты PNG.
    pub icon_bytes: Vec<u8>,
}

impl Favicon {
    /// `new Favicon(byte[])` — проверка PNG-сигнатуры не выполняется в Java.
    pub fn new(icon_bytes: Vec<u8>) -> Self {
        Self { icon_bytes }
    }
}

/// `ServerStatus.Players`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Players {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<NameAndId>,
}

impl Players {
    pub fn new(max: i32, online: i32, sample: Vec<NameAndId>) -> Self {
        Self { max, online, sample }
    }
}

/// `ServerStatus.Version`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Version {
    pub name: String,
    pub protocol: i32,
}

impl Version {
    pub fn new(name: impl Into<String>, protocol: i32) -> Self {
        Self { name: name.into(), protocol }
    }

    /// `Version.current()` — `SharedConstants.getCurrentVersion()`;
    /// SharedConstants портируется отдельно, здесь текущая версия Leaf.
    pub fn current() -> Version {
        Version::new(crate::core::shared_constants::CURRENT_VERSION_NAME, crate::core::shared_constants::CURRENT_PROTOCOL_VERSION)
    }
}
