//! Перенос `net.minecraft.util.Crypt` + `CipherBase`/`CipherEncoder`/
//! `CipherDecoder` (AES/CFB8 потоковый шифр для логин-шифрования).

pub mod crypt;
pub mod crypt_error;
pub mod cipher;

pub use crypt::{digest_data, encrypt_using_key, decrypt_using_key, byte_to_public_key, byte_to_private_key,
    string_to_rsa_public_key, string_to_pem_rsa_private_key, rsa_public_key_to_string,
    pem_rsa_private_key_to_string, generate_secret_key, generate_key_pair, sign_data, verify_signature};
pub use crypt_error::CryptError;
pub use cipher::AesCfb8Cipher;
