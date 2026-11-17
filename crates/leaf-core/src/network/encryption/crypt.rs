//! Перенос `net.minecraft.util.Crypt`.

use base64::Engine;
use rsa::pkcs8::{DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey};
use rsa::{Pkcs1v15Encrypt, Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey};
use sha1::Digest as _;
use sha2::Sha256;

use crate::network::encryption::crypt_error::CryptError;

/// `RSA`/`SHA-1`/`SHA256withRSA` параметры Java-версии.
const RSA_BITS: usize = 1024;

/// `Crypt.SIGNATURE_BYTES`.
pub const SIGNATURE_BYTES: usize = 256;

/// `Crypt.SaltSignaturePair` — соль + RSA-подпись (для signed chat).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SaltSignaturePair {
    /// `salt`.
    pub salt: i64,
    /// `signature` — пустой = невалидная пара.
    pub signature: Vec<u8>,
}

impl SaltSignaturePair {
    /// `EMPTY`.
    pub const EMPTY: Self = Self { salt: 0, signature: Vec::new() };

    /// `new SaltSignaturePair(salt, signature)`.
    pub fn new(salt: i64, signature: Vec<u8>) -> Self {
        Self { salt, signature }
    }

    /// `isValid` — signature.length > 0.
    pub fn is_valid(&self) -> bool {
        !self.signature.is_empty()
    }

    /// `saltAsBytes` — big-endian i64.
    pub fn salt_as_bytes(&self) -> [u8; 8] {
        self.salt.to_be_bytes()
    }
}

/// `Crypt.generateSecretKey` — AES-128 ключ (16 случайных байт).
pub fn generate_secret_key() -> Result<Vec<u8>, CryptError> {
    use rand::RngCore;
    let mut key = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut key);
    Ok(key.to_vec())
}

/// `Crypt.generateKeyPair` — RSA-1024 ключевая пара (PKCS8/X509 DER).
pub fn generate_key_pair() -> Result<(Vec<u8>, Vec<u8>), CryptError> {
    let mut rng = rand::thread_rng();
    let priv_key = RsaPrivateKey::new(&mut rng, RSA_BITS)?;
    let pub_key = RsaPublicKey::from(&priv_key);
    Ok((
        priv_key.to_pkcs8_der()?.as_bytes().to_vec(),
        pub_key.to_public_key_der()?.as_bytes().to_vec(),
    ))
}

/// `Crypt.digestData(serverId, publicKey, secretKey)` — SHA-1 хэш для
/// аутентификации (serverId в ISO-8859-1 → у нас UTF-8 bytes, ASCII-совместимо).
pub fn digest_data(
    server_id: &str,
    public_key: &[u8],
    secret_key: &[u8],
) -> Result<[u8; 20], CryptError> {
    Ok(digest_data_parts(&[server_id.as_bytes(), secret_key, public_key]))
}

/// Внутренний хэш нескольких частей (как `digestData(byte[]...)`).
pub fn digest_data_parts(parts: &[&[u8]]) -> [u8; 20] {
    let mut hasher = sha1::Sha1::new();
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().into()
}

/// `rsaStringToKey` — извлечь base64 между PEM-заголовками и декодировать.
/// Java `Base64.getMimeDecoder` игнорирует символы вне алфавита.
fn pem_decode(key_base64: &str) -> Vec<u8> {
    let cleaned: String = key_base64
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '+' || *c == '/' || *c == '=')
        .collect();
    base64::engine::general_purpose::STANDARD
        .decode(cleaned)
        .unwrap_or_default()
}

fn extract_pem(key_base64: &str, header: &str, footer: &str) -> String {
    match (key_base64.find(header), key_base64.find(footer)) {
        (Some(h), Some(f)) if f > h => key_base64[h + header.len()..f].to_string(),
        _ => key_base64.to_string(),
    }
}

/// `Crypt.stringToPemRsaPrivateKey`.
pub fn string_to_pem_rsa_private_key(key_base64: &str) -> Result<RsaPrivateKey, CryptError> {
    let body = extract_pem(
        key_base64,
        "-----BEGIN RSA PRIVATE KEY-----",
        "-----END RSA PRIVATE KEY-----",
    );
    RsaPrivateKey::from_pkcs8_der(&pem_decode(&body))
        .map_err(|e| CryptError::new(format!("invalid RSA private key: {e}")))
}

/// `Crypt.stringToRsaPublicKey`.
pub fn string_to_rsa_public_key(key_base64: &str) -> Result<RsaPublicKey, CryptError> {
    let body = extract_pem(
        key_base64,
        "-----BEGIN RSA PUBLIC KEY-----",
        "-----END RSA PUBLIC KEY-----",
    );
    RsaPublicKey::from_public_key_der(&pem_decode(&body))
        .map_err(|e| CryptError::new(format!("invalid RSA public key: {e}")))
}

/// `Crypt.rsaPublicKeyToString` — X509 SPKI DER → PEM.
pub fn rsa_public_key_to_string(key: &RsaPublicKey) -> Result<String, CryptError> {
    let der = key.to_public_key_der()?;
    Ok(format!(
        "-----BEGIN RSA PUBLIC KEY-----\n{}\n-----END RSA PUBLIC KEY-----\n",
        base64::engine::general_purpose::STANDARD.encode(der.as_bytes())
    ))
}

/// `Crypt.pemRsaPrivateKeyToString` — PKCS8 DER → PEM.
pub fn pem_rsa_private_key_to_string(key: &RsaPrivateKey) -> Result<String, CryptError> {
    let der = key.to_pkcs8_der()?;
    Ok(format!(
        "-----BEGIN RSA PRIVATE KEY-----\n{}\n-----END RSA PRIVATE KEY-----\n",
        base64::engine::general_purpose::STANDARD.encode(der.as_bytes())
    ))
}

/// `Crypt.byteToPrivateKey` — PKCS8 DER → RsaPrivateKey.
pub fn byte_to_private_key(key_bytes: &[u8]) -> Result<RsaPrivateKey, CryptError> {
    RsaPrivateKey::from_pkcs8_der(key_bytes)
        .map_err(|e| CryptError::new(format!("invalid private key: {e}")))
}

/// `Crypt.byteToPublicKey` — X509 SPKI DER → RsaPublicKey.
pub fn byte_to_public_key(encoded_key: &[u8]) -> Result<RsaPublicKey, CryptError> {
    RsaPublicKey::from_public_key_der(encoded_key)
        .map_err(|e| CryptError::new(format!("invalid public key: {e}")))
}

/// `Crypt.encryptUsingKey` — RSA/ECB/PKCS1Padding encrypt.
pub fn encrypt_using_key(key: &[u8], data: &[u8]) -> Result<Vec<u8>, CryptError> {
    let public_key = byte_to_public_key(key)?;
    let mut rng = rand::thread_rng();
    public_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, data)
        .map_err(CryptError::from)
}

/// `Crypt.decryptUsingKey` — RSA/ECB/PKCS1Padding decrypt.
pub fn decrypt_using_key(key: &[u8], data: &[u8]) -> Result<Vec<u8>, CryptError> {
    let private_key = byte_to_private_key(key)?;
    private_key
        .decrypt(Pkcs1v15Encrypt, data)
        .map_err(CryptError::from)
}

/// `Crypt.decryptByteToSecretKey` — расшифровать → AES-128 ключ (16 байт).
pub fn decrypt_byte_to_secret_key(
    private_key: &[u8],
    secret_key_encrypted: &[u8],
) -> Result<Vec<u8>, CryptError> {
    let bytes = decrypt_using_key(private_key, secret_key_encrypted)?;
    if bytes.len() != 16 {
        return Err(CryptError::new(format!(
            "decrypted secret key must be 16 bytes, got {}",
            bytes.len()
        )));
    }
    Ok(bytes)
}

/// `Crypt.getCipher(opMode, key)` — AES/CFB8/NoPadding cipher.
pub fn get_cipher(
    key: &[u8],
) -> Result<crate::network::encryption::cipher::AesCfb8Cipher, CryptError> {
    crate::network::encryption::cipher::AesCfb8Cipher::new(key)
}

/// `SHA256withRSA` подпись (`Signature` с SHA256withRSA).
/// Принимает PKCS8 DER приватного ключа и сообщение, возвращает подпись.
pub fn sign_data(private_key: &[u8], data: &[u8]) -> Result<Vec<u8>, CryptError> {
    let key = byte_to_private_key(private_key)?;
    key.sign(Pkcs1v15Sign::new::<Sha256>(), data)
        .map_err(CryptError::from)
}

/// Проверка `SHA256withRSA` подписи публичным ключом (X509 DER).
pub fn verify_signature(
    public_key: &[u8],
    data: &[u8],
    signature: &[u8],
) -> Result<bool, CryptError> {
    let key = byte_to_public_key(public_key)?;
    key.verify(Pkcs1v15Sign::new::<Sha256>(), data, signature)
        .map(|_| true)
        .or_else(|_| Ok(false))
}
