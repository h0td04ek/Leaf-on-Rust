//! Перенос `com.mojang.authlib.properties.Property`.

/// `Property` — (name, value, signature?).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Property {
    pub name: String,
    pub value: String,
    /// `signature` — RSA-подпись (optional).
    pub signature: Option<String>,
}

impl Property {
    pub fn new(name: impl Into<String>, value: impl Into<String>, signature: Option<String>) -> Self {
        Self { name: name.into(), value: value.into(), signature }
    }

    /// `hasSignature`.
    pub fn has_signature(&self) -> bool {
        self.signature.is_some()
    }
}
