//! Перенос `net.minecraft.util.CryptException`.

use std::fmt;

/// `CryptException`.
#[derive(Debug)]
pub struct CryptError {
    pub message: String,
}

impl CryptError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl fmt::Display for CryptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CryptException: {}", self.message)
    }
}

impl std::error::Error for CryptError {}

impl From<rsa::Error> for CryptError {
    fn from(e: rsa::Error) -> Self {
        Self::new(e.to_string())
    }
}

impl From<aes::cipher::InvalidLength> for CryptError {
    fn from(e: aes::cipher::InvalidLength) -> Self {
        Self::new(e.to_string())
    }
}

impl From<rsa::pkcs8::Error> for CryptError {
    fn from(e: rsa::pkcs8::Error) -> Self {
        Self::new(e.to_string())
    }
}

impl From<rsa::pkcs8::spki::Error> for CryptError {
    fn from(e: rsa::pkcs8::spki::Error) -> Self {
        Self::new(e.to_string())
    }
}
