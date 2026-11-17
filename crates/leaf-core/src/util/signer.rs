//! Перенос `net.minecraft.util.Signer`.

use rsa::pkcs1v15::{Signature, SigningKey};
use rsa::signature::Signer as _;
use rsa::signature::SignatureEncoding;
use rsa::{RsaPrivateKey, RsaPublicKey};
use sha2::{Digest, Sha256};

use crate::util::signature_validator::{SignatureException, SignatureUpdater};

/// `Signer` — подписывает данные через `SignatureUpdater`.
pub trait Signer {
    /// `sign(updater)` → signature bytes.
    fn sign(&self, updater: &mut dyn SignatureUpdater) -> Result<Vec<u8>, SignatureException>;

    /// `sign(byte[])` — convenience.
    fn sign_bytes(&self, data: &[u8]) -> Result<Vec<u8>, SignatureException> {
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
        self.sign(&mut Fixed { data })
    }
}

/// `Signer.from(privateKey, "SHA256withRSA")` — PKCS1v15 + SHA-256.
#[derive(Clone)]
pub struct RsaSha256Signer {
    signing_key: SigningKey<Sha256>,
    private_key: RsaPrivateKey,
}

impl RsaSha256Signer {
    /// Создаёт signer из приватного ключа.
    pub fn from_private_key(private_key: RsaPrivateKey) -> Self {
        let signing_key = SigningKey::new(private_key.clone());
        Self { signing_key, private_key }
    }

    /// `privateKey`.
    pub fn private_key(&self) -> &RsaPrivateKey {
        &self.private_key
    }

    /// Публичный ключ, соответствующий приватному.
    pub fn public_key(&self) -> RsaPublicKey {
        RsaPublicKey::from(&self.private_key)
    }
}

impl Signer for RsaSha256Signer {
    fn sign(&self, updater: &mut dyn SignatureUpdater) -> Result<Vec<u8>, SignatureException> {
        let mut hasher = Sha256::new();
        updater
            .update(&mut |chunk: &[u8]| {
                hasher.update(chunk);
                Ok(())
            })
            .map_err(|e| SignatureException(e.to_string()))?;
        let digest = hasher.finalize();
        let sig: Signature = self
            .signing_key
            .try_sign(&digest)
            .map_err(|e| SignatureException(e.to_string()))?;
        Ok(sig.to_vec())
    }
}
