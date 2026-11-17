//! Перенос `CipherBase`/`CipherEncoder`/`CipherDecoder` —
//! `AES/CFB8/NoPadding` потоковый шифр с IV = ключу.

use aes::Aes128;
use aes::cipher::{BlockEncrypt, KeyInit};
use crate::network::encryption::crypt_error::CryptError;

/// `Cipher` — AES/CFB8/NoPadding.
///
/// Java: `Cipher.getInstance("AES/CFB8/NoPadding")` с `IvParameterSpec(key)`.
/// CFB8: `C_i = P_i XOR E_K(C_{i-1})[0]`, регистр сдвигается побайтно.
/// Шифрование и дешифрование — одна и та же операция XOR с keystream.
pub struct AesCfb8Cipher {
    cipher: aes::Aes128,
    /// 16-байтовый регистр обратной связи (IV = ключ).
    register: [u8; 16],
}

impl AesCfb8Cipher {
    /// `Crypt.getCipher(opMode, key)` — создаёт шифр (opMode игнорируется,
    /// CFB8 симметричен).
    pub fn new(key: &[u8]) -> Result<Self, CryptError> {
        if key.len() != 16 {
            return Err(CryptError::new(format!(
                "AES-128 key must be 16 bytes, got {}", key.len()
            )));
        }
        let mut register = [0u8; 16];
        register.copy_from_slice(key);
        Ok(Self { cipher: Aes128::new_from_slice(key)?, register })
    }

    /// Обработка одного байта (encrypt и decrypt идентичны).
    fn crypt_byte(&mut self, byte: u8) -> u8 {
        use aes::cipher::generic_array::GenericArray;
        let block = GenericArray::from_mut_slice(&mut self.register);
        self.cipher.encrypt_block(block);
        let out = self.register[0] ^ byte;
        // CFB8: регистр сдвигается влево на байт, в конец — шифротекст.
        self.register.copy_within(1..16, 0);
        self.register[15] = out;
        out
    }

    /// `doFinal(data)` / потоковая обработка.
    pub fn crypt(&mut self, data: &[u8]) -> Vec<u8> {
        data.iter().map(|&b| self.crypt_byte(b)).collect()
    }
}
