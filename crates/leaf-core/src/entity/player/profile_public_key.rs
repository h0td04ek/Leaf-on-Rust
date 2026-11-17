//! Перенос `net.minecraft.world.entity.player.ProfilePublicKey` (+ `Data`).

use rsa::RsaPublicKey;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::network::friendly_byte_buf::{CodecError, FriendlyByteBuf};
use crate::util::uuid::Uuid;

/// `MAX_KEY_SIGNATURE_SIZE`.
const MAX_KEY_SIGNATURE_SIZE: usize = 4096;

/// `ProfilePublicKey.Data` — expiresAt + RSA public key + подпись ключа.
#[derive(Clone)]
pub struct ProfilePublicKeyData {
    /// `expiresAt` — epoch milli.
    pub expires_at: i64,
    /// `key` — X509 RSA public key.
    pub key: RsaPublicKey,
    /// `keySignature`.
    pub key_signature: Vec<u8>,
}

impl std::fmt::Debug for ProfilePublicKeyData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfilePublicKey.Data")
            .field("expires_at", &self.expires_at)
            .field("key_signature_len", &self.key_signature.len())
            .finish_non_exhaustive()
    }
}

impl PartialEq for ProfilePublicKeyData {
    fn eq(&self, other: &Self) -> bool {
        self.expires_at == other.expires_at
            && self.key == other.key
            && self.key_signature == other.key_signature
    }
}
impl Eq for ProfilePublicKeyData {}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

impl ProfilePublicKeyData {
    pub fn new(expires_at: i64, key: RsaPublicKey, key_signature: Vec<u8>) -> Self {
        Self { expires_at, key, key_signature }
    }

    /// `new Data(FriendlyByteBuf)`.
    pub fn read(buffer: &mut FriendlyByteBuf) -> Result<Self, CodecError> {
        let expires_at = buffer.read_instant();
        let key = buffer.read_public_key()?;
        let key_signature = buffer.read_byte_array_n(MAX_KEY_SIGNATURE_SIZE)?;
        Ok(Self { expires_at, key, key_signature })
    }

    /// `write(FriendlyByteBuf)`.
    pub fn write(&self, buffer: &mut FriendlyByteBuf) -> Result<(), CodecError> {
        buffer.write_instant(self.expires_at);
        buffer.write_public_key(&self.key)?;
        buffer.write_byte_array(&self.key_signature);
        Ok(())
    }

    /// `validateSignature` — RSA PKCS1v15 SHA256 над signedPayload.
    pub fn validate_signature(
        &self,
        signature_validator: &crate::util::signature_validator::SignatureValidator,
        profile_id: &Uuid,
    ) -> bool {
        signature_validator.validate(&self.signed_payload(profile_id), &self.key_signature)
    }

    /// `signedPayload(profileId)` — UUID(16) + expiresAt(8) + X509 DER.
    pub fn signed_payload(&self, profile_id: &Uuid) -> Vec<u8> {
        use rsa::pkcs8::EncodePublicKey;
        let encoded = self.key.to_public_key_der().map(|d| d.as_bytes().to_vec()).unwrap_or_default();
        let mut bytes = Vec::with_capacity(24 + encoded.len());
        bytes.extend_from_slice(&profile_id.most_significant_bits.to_be_bytes());
        bytes.extend_from_slice(&profile_id.least_significant_bits.to_be_bytes());
        bytes.extend_from_slice(&self.expires_at.to_be_bytes());
        bytes.extend_from_slice(&encoded);
        bytes
    }

    /// `hasExpired`.
    pub fn has_expired(&self) -> bool {
        self.expires_at < now_millis()
    }

    /// `hasExpired(Duration gracePeriod)` — grace в миллисекундах.
    pub fn has_expired_with_grace(&self, grace_millis: i64) -> bool {
        self.expires_at + grace_millis < now_millis()
    }
}

/// `ProfilePublicKey`.
#[derive(Clone)]
pub struct ProfilePublicKey {
    /// `data`.
    pub data: ProfilePublicKeyData,
}

impl std::fmt::Debug for ProfilePublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfilePublicKey").field("data", &self.data).finish()
    }
}
impl PartialEq for ProfilePublicKey {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
impl Eq for ProfilePublicKey {}

/// `ProfilePublicKey.EXPIRY_GRACE_PERIOD` = 8 часов.
pub const EXPIRY_GRACE_PERIOD_MILLIS: i64 = 8 * 60 * 60 * 1000;

impl ProfilePublicKey {
    pub fn new(data: ProfilePublicKeyData) -> Self {
        Self { data }
    }

    /// `createSignatureValidator`.
    pub fn create_signature_validator(&self) -> crate::util::signature_validator::SignatureValidator {
        crate::util::signature_validator::SignatureValidator::from_public_key(
            self.data.key.clone(),
        )
    }
}
