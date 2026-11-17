//! Перенос `net.minecraft.util.SignatureValidator` (+ `SignatureUpdater`).

use rsa::pkcs1v15::{Signature, VerifyingKey};
use rsa::signature::Verifier;
use rsa::RsaPublicKey;
use sha2::Sha256;

/// `SignatureUpdater` — обновление подписываемых данных чанками.
pub trait SignatureUpdater {
    /// `update(Output)` — скармливает данные подписывающему коду.
    /// Бросает `Err` при ошибке подписи/проверки.
    fn update(&mut self, output: &mut dyn FnMut(&[u8]) -> Result<(), SignatureException>) -> Result<(), SignatureException>;
}

/// Ошибка подписывания/проверки (аналог `java.security.SignatureException`).
#[derive(Debug)]
pub struct SignatureException(pub String);

impl std::fmt::Display for SignatureException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for SignatureException {}

/// `SignatureValidator`.
pub trait SignatureValidatorTrait {
    /// `validate(updater, signature)`.
    fn validate(&self, updater: &mut dyn SignatureUpdater, signature: &[u8]) -> bool;

    /// `validate(digest, signature)` — convenience.
    fn validate_bytes(&self, digest: &[u8], signature: &[u8]) -> bool {
        struct Fixed<'a> {
            data: &'a [u8],
        }
        impl<'a> SignatureUpdater for Fixed<'a> {
            fn update(
                &mut self,
                output: &mut dyn FnMut(&[u8]) -> Result<(), SignatureException>,
            ) -> Result<(), SignatureException> {
                output(self.data)
            }
        }
        self.validate(&mut Fixed { data: digest }, signature)
    }
}

/// Реализация `SignatureValidator.from(PublicKey, "SHA256withRSA")`.
/// PKCS1v15 + SHA-256.
#[derive(Clone)]
pub struct RsaSha256SignatureValidator {
    verifying_key: VerifyingKey<Sha256>,
}

impl RsaSha256SignatureValidator {
    /// `SignatureValidator.from(publicKey, "SHA256withRSA")`.
    pub fn from_public_key(key: RsaPublicKey) -> Self {
        Self { verifying_key: VerifyingKey::new(key) }
    }
}

impl SignatureValidatorTrait for RsaSha256SignatureValidator {
    fn validate(&self, updater: &mut dyn SignatureUpdater, signature: &[u8]) -> bool {
        // Собираем digest: updater скармливает чанки в hasher.
        let mut hasher = Sha256::new();
        use sha2::Digest;
        let mut ok = true;
        let _ = updater.update(&mut |chunk: &[u8]| {
            if ok {
                hasher.update(chunk);
            }
            Ok(())
        });
        let digest = hasher.finalize();
        Signature::try_from(signature)
            .map(|sig| self.verifying_key.verify(&digest, &sig).is_ok())
            .unwrap_or(false)
    }
}

/// `SignatureValidator.NO_VALIDATION` — всегда true.
pub struct NoValidation;

impl SignatureValidatorTrait for NoValidation {
    fn validate(&self, _updater: &mut dyn SignatureUpdater, _signature: &[u8]) -> bool {
        true
    }
}

/// Удобный owned-вариант для хранения в структурах.
#[derive(Clone)]
pub enum SignatureValidator {
    /// `NO_VALIDATION`.
    NoValidation,
    /// RSA SHA256.
    RsaSha256(RsaPublicKey),
}

impl SignatureValidator {
    /// `SignatureValidator.from(publicKey, algorithm)` для RSA/SHA256withRSA.
    pub fn from_public_key(key: RsaPublicKey) -> Self {
        Self::RsaSha256(key)
    }

    /// `validate(digest, signature)` — byte[]-вариант.
    pub fn validate(&self, payload: &[u8], signature: &[u8]) -> bool {
        match self {
            SignatureValidator::NoValidation => true,
            SignatureValidator::RsaSha256(key) => {
                let vk = RsaSha256SignatureValidator::from_public_key(key.clone());
                vk.validate_bytes(payload, signature)
            }
        }
    }
}
