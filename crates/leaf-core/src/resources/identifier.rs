//! Перенос `net.minecraft.resources.Identifier`
//! (в Mojmap — ResourceLocation).
//! Brigaster `StringReader`-методы (`read`, `readNonEmpty`) — TODO:
//! требуют порта brigadier. Codec/DataResult — TODO (serialization).

use std::cmp::Ordering;
use std::fmt;

pub const NAMESPACE_SEPARATOR: char = ':';
pub const DEFAULT_NAMESPACE: &str = "minecraft";
pub const REALMS_NAMESPACE: &str = "realms";
pub const PAPER_NAMESPACE: &str = "paper";

const MAX_LENGTH: usize = i16::MAX as usize; // Short.MAX_VALUE

#[derive(Clone)]
pub struct Identifier {
    pub namespace: String,
    pub path: String,
}

impl Identifier {
    fn new_unchecked(namespace: String, path: String) -> Self {
        let rl = format!("{}:{}", namespace, path);
        // Paper: лимит длины для сети/NBT
        assert!(
            rl.len() <= MAX_LENGTH,
            "Resource location too long: {}",
            rl
        );
        debug_assert!(Self::is_valid_namespace(&namespace));
        debug_assert!(Self::is_valid_path(&path));
        Self { namespace, path }
    }

    pub fn from_namespace_and_path(namespace: &str, path: &str) -> Result<Self, IdentifierException> {
        Ok(Self::new_unchecked(
            Self::assert_valid_namespace(namespace, path)?,
            Self::assert_valid_path(namespace, path)?,
        ))
    }

    pub fn parse(location: &str) -> Result<Self, IdentifierException> {
        Self::by_separator(location, NAMESPACE_SEPARATOR)
    }

    pub fn with_default_namespace(location: &str) -> Result<Self, IdentifierException> {
        Ok(Self::new_unchecked(
            DEFAULT_NAMESPACE.to_string(),
            Self::assert_valid_path(DEFAULT_NAMESPACE, location)?,
        ))
    }

    pub fn try_parse(location: &str) -> Option<Self> {
        Self::try_by_separator(location, NAMESPACE_SEPARATOR)
    }

    pub fn try_build(namespace: &str, path: &str) -> Option<Self> {
        if Self::is_valid_namespace(namespace) && Self::is_valid_path(path) {
            Some(Self::new_unchecked(namespace.to_string(), path.to_string()))
        } else {
            None
        }
    }

    pub fn by_separator(location: &str, separator: char) -> Result<Self, IdentifierException> {
        match location.find(separator) {
            Some(index) => {
                let sub = &location[index + 1..];
                if index != 0 {
                    let sub1 = &location[..index];
                    Self::create_untrusted(sub1, sub)
                } else {
                    Self::with_default_namespace(sub)
                }
            }
            None => Self::with_default_namespace(location),
        }
    }

    pub fn try_by_separator(location: &str, separator: char) -> Option<Self> {
        match location.find(separator) {
            Some(index) => {
                let sub = &location[index + 1..];
                if !Self::is_valid_path(sub) {
                    return None;
                }
                if index != 0 {
                    let sub1 = &location[..index];
                    if Self::is_valid_namespace(sub1) {
                        Some(Self::new_unchecked(sub1.to_string(), sub.to_string()))
                    } else {
                        None
                    }
                } else {
                    Some(Self::new_unchecked(DEFAULT_NAMESPACE.to_string(), sub.to_string()))
                }
            }
            None => {
                if Self::is_valid_path(location) {
                    Some(Self::new_unchecked(DEFAULT_NAMESPACE.to_string(), location.to_string()))
                } else {
                    None
                }
            }
        }
    }

    fn create_untrusted(namespace: &str, path: &str) -> Result<Self, IdentifierException> {
        Ok(Self::new_unchecked(
            Self::assert_valid_namespace(namespace, path)?,
            Self::assert_valid_path(namespace, path)?,
        ))
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn get_namespace(&self) -> &str {
        &self.namespace
    }

    pub fn with_path(&self, path: &str) -> Result<Self, IdentifierException> {
        Ok(Self::new_unchecked(
            self.namespace.clone(),
            Self::assert_valid_path(&self.namespace, path)?,
        ))
    }

    pub fn with_path_fn(&self, f: impl FnOnce(&str) -> String) -> Result<Self, IdentifierException> {
        self.with_path(&f(&self.path))
    }

    pub fn with_prefix(&self, path_prefix: &str) -> Result<Self, IdentifierException> {
        self.with_path(&format!("{}{}", path_prefix, self.path))
    }

    pub fn with_suffix(&self, path_suffix: &str) -> Result<Self, IdentifierException> {
        self.with_path(&format!("{}{}", self.path, path_suffix))
    }

    pub fn to_debug_file_name(&self) -> String {
        self.to_string().replace('/', "_").replace(':', "_")
    }

    pub fn to_language_key(&self) -> String {
        format!("{}.{}", self.namespace, self.path)
    }

    pub fn to_short_language_key(&self) -> String {
        if self.namespace == DEFAULT_NAMESPACE {
            self.path.clone()
        } else {
            self.to_language_key()
        }
    }

    pub fn to_short_string(&self) -> String {
        if self.namespace == DEFAULT_NAMESPACE {
            self.path.clone()
        } else {
            self.to_string()
        }
    }

    pub fn to_language_key_of(&self, type_: &str) -> String {
        format!("{}.{}", type_, self.to_language_key())
    }

    pub fn to_language_key_of_key(&self, type_: &str, key: &str) -> String {
        format!("{}.{}.{}", type_, self.to_language_key(), key)
    }

    // ---------------------------------------------------------- validation

    pub fn is_allowed_in_identifier(c: char) -> bool {
        c.is_ascii_digit()
            || c.is_ascii_lowercase()
            || c == '_'
            || c == ':'
            || c == '/'
            || c == '.'
            || c == '-'
    }

    // Java-версия: пустая строка валидна (цикл по символам не выполняется)
    pub fn is_valid_path(path: &str) -> bool {
        path.chars().all(Self::valid_path_char)
    }

    pub fn is_valid_namespace(namespace: &str) -> bool {
        namespace.chars().all(Self::valid_namespace_char)
    }

    pub fn valid_path_char(c: char) -> bool {
        c == '_'
            || c == '-'
            || c.is_ascii_lowercase()
            || c.is_ascii_digit()
            || c == '/'
            || c == '.'
    }

    fn valid_namespace_char(c: char) -> bool {
        c == '_'
            || c == '-'
            || c.is_ascii_lowercase()
            || c.is_ascii_digit()
            || c == '.'
    }

    fn assert_valid_namespace(namespace: &str, path: &str) -> Result<String, IdentifierException> {
        if !Self::is_valid_namespace(namespace) {
            Err(IdentifierException::new(format!(
                "Non [a-z0-9_.-] character in namespace of location: {}:{}",
                namespace, path
            )))
        } else {
            Ok(namespace.to_string())
        }
    }

    fn assert_valid_path(namespace: &str, path: &str) -> Result<String, IdentifierException> {
        if !Self::is_valid_path(path) {
            Err(IdentifierException::new(format!(
                "Non [a-z0-9/._-] character in path of location: {}:{}",
                namespace, path
            )))
        } else {
            Ok(path.to_string())
        }
    }

    /// Известные константы.
    pub fn minecraft(path: &str) -> Self {
        Self::new_unchecked(DEFAULT_NAMESPACE.to_string(), path.to_string())
    }
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.namespace == other.namespace && self.path == other.path
    }
}
impl Eq for Identifier {}

impl std::hash::Hash for Identifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.namespace.hash(state);
        self.path.hash(state);
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Identifier({})", self)
    }
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> Ordering {
        // Java compareTo: сначала path, потом namespace
        self.path
            .cmp(&other.path)
            .then_with(|| self.namespace.cmp(&other.namespace))
    }
}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<&str> for Identifier {
    /// `Identifier.parse` — паникует при невалидном (как Java-версия кидает exception).
    fn from(s: &str) -> Self {
        Self::parse(s).expect("invalid resource location")
    }
}

/// `net.minecraft.IdentifierException`.
#[derive(Debug)]
pub struct IdentifierException {
    message: String,
}

impl IdentifierException {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl fmt::Display for IdentifierException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for IdentifierException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_default_namespace() {
        let id = Identifier::parse("stone").unwrap();
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.path, "stone");
        assert_eq!(id.to_string(), "minecraft:stone");
        assert_eq!(id.to_short_string(), "stone");
    }

    #[test]
    fn parse_with_namespace() {
        let id = Identifier::parse("mymod:block/thing").unwrap();
        assert_eq!(id.namespace, "mymod");
        assert_eq!(id.path, "block/thing");
    }

    #[test]
    fn parse_colon_only() {
        let id = Identifier::parse(":path").unwrap();
        assert_eq!(id.namespace, "minecraft");
        assert_eq!(id.path, "path");
    }

    #[test]
    fn invalid_paths() {
        assert!(Identifier::parse("BAD").is_err());
        assert!(Identifier::parse("foo:has space").is_err());
        assert!(Identifier::try_parse("bad path").is_none());
        assert!(Identifier::is_valid_path("a/b.c-d_e"));
    }

    #[test]
    fn ordering_is_path_first() {
        let a = Identifier::parse("zzz:a").unwrap();
        let b = Identifier::parse("aaa:z").unwrap();
        assert!(a < b); // path "a" < "z"
    }
}
