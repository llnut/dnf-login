// dnf-login: a launcher for Dungeon & Fighter written in Rust.
// Copyright (C) 2026 llnut
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

use crate::{DnfError, Result};

/// AES-256-GCM cipher for encrypting/decrypting client-server communication
#[derive(Clone)]
pub struct AesGcmCipher {
    cipher: Aes256Gcm,
}

impl AesGcmCipher {
    /// Create new cipher from 32-byte key
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new(key.into());
        Self { cipher }
    }

    /// Create from hex-encoded key string
    pub fn from_hex_key(hex_key: &str) -> Result<Self> {
        if hex_key.len() != 64 {
            return Err(DnfError::Encryption(
                "Key must be 64 hex characters (32 bytes)".to_string(),
            ));
        }

        let key_bytes = hex::decode(hex_key)?;
        if key_bytes.len() != 32 {
            return Err(DnfError::Encryption("Invalid key length".to_string()));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        Ok(Self::new(&key))
    }

    /// Encrypt plaintext and return base64-encoded ciphertext (with prepended nonce)
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<String> {
        // Generate random 96-bit nonce via AeadCore (uses rand_core internally)
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // Encrypt
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| DnfError::Encryption(e.to_string()))?;

        // Prepend nonce (12 bytes) to ciphertext, then base64-encode
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(BASE64.encode(&result))
    }

    /// Decrypt base64-encoded ciphertext (with prepended nonce)
    pub fn decrypt(&self, base64_data: &str) -> Result<Vec<u8>> {
        // Base64 decode
        let data = BASE64
            .decode(base64_data)
            .map_err(|e| DnfError::Decryption(e.to_string()))?;

        if data.len() < 12 {
            return Err(DnfError::Decryption("Data too short".to_string()));
        }

        // Extract nonce (first 12 bytes)
        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        // Decrypt
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| DnfError::Decryption(e.to_string()))?;

        Ok(plaintext)
    }

    /// Encrypt and return as base64 string
    pub fn encrypt_string(&self, plaintext: &str) -> Result<String> {
        self.encrypt(plaintext.as_bytes())
    }

    /// Decrypt from base64 string and return as UTF-8 string
    pub fn decrypt_string(&self, base64_data: &str) -> Result<String> {
        let bytes = self.decrypt(base64_data)?;
        String::from_utf8(bytes).map_err(|e| DnfError::Decryption(format!("Invalid UTF-8: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = [0u8; 32]; // Test key
        let cipher = AesGcmCipher::new(&key);

        let plaintext = b"Hello, DNF!";
        let encrypted = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_string() {
        let key = [1u8; 32];
        let cipher = AesGcmCipher::new(&key);

        let plaintext = "login|testuser|password|mac";
        let encrypted = cipher.encrypt_string(plaintext).unwrap();
        let decrypted = cipher.decrypt_string(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_from_hex_key() {
        let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let cipher = AesGcmCipher::from_hex_key(hex_key).unwrap();

        let plaintext = b"test";
        let encrypted = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_different_nonces() {
        let key = [2u8; 32];
        let cipher = AesGcmCipher::new(&key);

        let plaintext = b"same data";
        let encrypted1 = cipher.encrypt(plaintext).unwrap();
        let encrypted2 = cipher.encrypt(plaintext).unwrap();

        // Different ciphertexts due to different nonces
        assert_ne!(encrypted1, encrypted2);

        // But both decrypt to same plaintext
        assert_eq!(cipher.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(cipher.decrypt(&encrypted2).unwrap(), plaintext);
    }
}
