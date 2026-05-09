use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine as _, engine::general_purpose};
use rand::random;
use std::env;

const NONCE_SIZE: usize = 12;

lazy_static::lazy_static! {
    static ref ENCRYPTION_KEY: Key<Aes256Gcm> = {
        let key_env = env::var("CONFIG_ENCRYPTION_KEY").unwrap_or_else(|_| {
            "iNovelConfigEncryptionKey256BitLength!!".to_string()
        });
        let mut key = [0u8; 32];
        for (i, c) in key_env.as_bytes().iter().enumerate().take(32) {
            key[i] = *c;
        }
        Key::<Aes256Gcm>::from_slice(&key).clone()
    };
}

pub fn encrypt(value: &str) -> Result<String, String> {
    let cipher = Aes256Gcm::new(&ENCRYPTION_KEY);
    let nonce_bytes: [u8; NONCE_SIZE] = random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    match cipher.encrypt(nonce, value.as_bytes()) {
        Ok(ciphertext) => {
            let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
            result.extend_from_slice(&nonce_bytes);
            result.extend_from_slice(&ciphertext);
            Ok(general_purpose::STANDARD.encode(&result))
        }
        Err(e) => Err(format!("Encryption failed: {}", e)),
    }
}

pub fn decrypt(value: &str) -> Result<String, String> {
    let cipher = Aes256Gcm::new(&ENCRYPTION_KEY);
    let decoded = general_purpose::STANDARD
        .decode(value)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    if decoded.len() < NONCE_SIZE {
        return Err("Invalid encrypted data".to_string());
    }

    let (nonce_bytes, ciphertext) = decoded.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map(|plaintext| String::from_utf8_lossy(plaintext.as_slice()).to_string())
        .map_err(|e| format!("Decryption failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let original = "sensitive_data_123";
        let encrypted = encrypt(original).unwrap();
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_empty_string() {
        let original = "";
        let encrypted = encrypt(original).unwrap();
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }
}
