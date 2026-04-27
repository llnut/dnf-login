use aes_gcm::aead::OsRng;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::traits::{PublicKeyParts, SignatureScheme};
use rsa::{Pkcs1v15Sign, RsaPrivateKey};

use crate::types::UserId;
use crate::{DnfError, Result};

/// Token generator for DNF login tokens.
///
/// Uses RSA PKCS#1 v1.5 private-key signing (type-1 block). Channel
/// verifies the token with the matching public key embedded in its binary.
///
/// Token plaintext format: `{uid:08x}010101...0155914510010403030101`.
pub struct TokenGenerator {
    private_key: RsaPrivateKey,
}

impl TokenGenerator {
    /// Create from PEM-encoded private key
    pub fn from_pem(pem_data: &str) -> Result<Self> {
        let private_key = RsaPrivateKey::from_pkcs8_pem(pem_data)
            .or_else(|_| RsaPrivateKey::from_pkcs1_pem(pem_data))
            .map_err(|e| DnfError::TokenGeneration(format!("Invalid PEM key: {}", e)))?;

        Ok(Self { private_key })
    }

    /// Generates a login token for the given user ID.
    ///
    /// Uses PKCS#1 v1.5 type-1 block signing; the result is base64-encoded.
    /// The game channel server verifies the token with its embedded public key.
    pub fn generate_token(&self, user_id: UserId) -> Result<String> {
        let token_hex = format!(
            "{:08x}010101010101010101010101010101010101010101010101010101010101010155914510010403030101",
            user_id
        );
        debug_assert_eq!(token_hex.len(), 92);

        let token_bytes = hex::decode(&token_hex)
            .map_err(|e| DnfError::TokenGeneration(format!("Hex decode failed: {}", e)))?;
        debug_assert_eq!(token_bytes.len(), 46);

        // PKCS#1 v1.5 private-key sign (type-1 block, no hash prefix).
        // Deterministic.
        let encrypted = Pkcs1v15Sign::new_unprefixed()
            .sign(None::<&mut OsRng>, &self.private_key, &token_bytes)
            .map_err(|e| DnfError::TokenGeneration(format!("RSA sign failed: {}", e)))?;

        Ok(BASE64.encode(&encrypted))
    }

    /// Get the key modulus size in bits.
    pub fn key_size(&self) -> usize {
        self.private_key.size() * 8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2048-bit RSA test key
    const TEST_PRIVATE_KEY_PEM: &str = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEpQIBAAKCAQEAu8OiyQMqG7JsliG9W/dOI6hDZuHmgF7BO/gsJqwaFhQd2mhS
fXNVGY2ONnprWvA65itx7qcLUNZT9i9gkwyogn0YFyCXRS+Le34FLuCJ6ur2EBQ/
4mHTO3wylqmv5OiyJKv/w3nALrH+1IxXdxEAM0+TSzpoBCaizD94lJRl/hck/KvC
LhOn8D3K+fLryRhwgGCRXekS4F4p/WIbsjBQnTY6crcEQByudbHRuX9QWqtu9Yu8
qhhchJcqocirXWsBCK9AkMO0d0BzcdTcejswO9fa2/dqJyqOcrrmmoWOb0GOhLG2
OICto8qJ/6zHiNGhStK1yzmxcXm95zXGhWI52wIDAQABAoIBAAx3NZSA2EfUda8V
+FtltNNbNXZcIxB8ufmARXYf0O+MUFsSt/9KK+kxY7KsN/pmnpJvafX9Mxwfzp02
kgPRQFLBeVr3t/NI78q4GCH/mEh3ZvS0U3V1Jy/40+b6xwm8hS84GBfjOmYfPRrh
YmEuSMQfUVkaPJOh+Qb0Y84BeDABPjxtJ82ly/1PxetFTvcuei6wCKWeombN2oiQ
2ih40cnWrxhzabNw/Bo709ArM/mpfXbOs9ib0tFWIVmTT0B3Ddc8EGCZvPXmji0S
8+5p5X6zBMA5iyG8s2NvRg3TuBw1u0l0A5k5aFQA2+2AvSzRlQhpjfGFjXkVknk/
JZy1fTkCgYEA4fivcJYUqKiK2RtHLyh2E4zyxwsZu2yYVuwwFSW6qY6z/m/P0ot+
MAlZ235ZWCxOp7bPXWnsRirhBBb3w+Y8WVmCHLTNS0xkaCHorZPOnoQa4RM126Vo
51k/8EoKDUiJ4ULLoAxrHMRk9i0qP4V0p8/MOlsZsrGWFFmf0g3dBE0CgYEA1Lcu
I+OQ/kYBtst6AXAgXuIAGS99u75c9P3QubA72/inAu507HaBdIaWzAuMVmMco3Ri
qnwliAOiz8ZhEKotDGV1iFBV3s3OzSSrdk6EWEH5nDgO9xpFnem5eimLsDmdDZ8j
RitRqjUNcY7O3KWXWYDBvVS8j5GkBtIJG3v8ascCgYEAgWO6YUcucRyA1Kvv6KrM
YYl1gk9y3oTh/fOj3JgL+AbEPc6cOzywdqUEFNCWLAzCxPnCZwS9y7fFvGfCWyO8
LpU4EWPdoV4OqCmyZ6GYz99o3LP5RNnD5aSPHfHnK4/7k0aB/hTeSEyUWvmllVW/
ZE9x64A6iL1y6BghkU9q3IkCgYEAhUKQ/FjXgASZlEvbDkWRcf/BsgWHjnOOxsiv
13Spu4AGGRcMVwtSxI6AsCnX7FLBIUGLgmSuGoy0ldgg/RCvkiGJxTEW6rMiiHAd
nstHrAcA+jZAYduqm2hOE1MtuOQPGPaGYbJHwgrkdizSOXbf32mDdjo8uvCxwrgY
johZNQcCgYEAkA1WXxaIMbaa0VDIGH48VXzmHxPWnoEgXnA5wR34bxf3XUYqRh2/
0bCcd7UNCV2ZmjlkCvoHLvzfGQy0Fe/usmllO+jTKkqDn+6+Pdmlvggq8D/nBPU8
6fELbAaAY7s5V4mRI9T7p82CO17p3PGaJIXg9Sju621JUfQn/9FatPI=
-----END RSA PRIVATE KEY-----"#;

    #[test]
    fn test_token_generation() {
        let generator = TokenGenerator::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();

        let user_id = 1000u32;
        let token = generator.generate_token(user_id).unwrap();

        // Token should be base64 encoded
        assert!(!token.is_empty());
        assert!(BASE64.decode(&token).is_ok());
    }

    #[test]
    fn test_token_format_consistency() {
        let generator = TokenGenerator::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();

        // Test with specific user IDs
        let test_cases = vec![
            (1, "00000001"),
            (1000, "000003e8"),
            (65535, "0000ffff"),
            (16777215, "00ffffff"),
        ];

        for (user_id, expected_hex_prefix) in test_cases {
            let token = generator.generate_token(user_id).unwrap();

            // Verify token is not empty
            assert!(!token.is_empty());

            // The hex format should start with the user ID
            let token_hex = format!(
                "{:08x}010101010101010101010101010101010101010101010101010101010101010155914510010403030101",
                user_id
            );
            assert!(token_hex.starts_with(expected_hex_prefix));
        }
    }

    #[test]
    fn test_token_uniqueness_for_different_users() {
        let generator = TokenGenerator::from_pem(TEST_PRIVATE_KEY_PEM).unwrap();

        let token1 = generator.generate_token(1).unwrap();
        let token2 = generator.generate_token(2).unwrap();

        // Tokens for different users should be different
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_hex_format() {
        let user_id = 0x12345678u32;
        let token_hex = format!(
            "{:08x}010101010101010101010101010101010101010101010101010101010101010155914510010403030101",
            user_id
        );

        assert_eq!(
            token_hex.len(),
            92,
            "Token hex string should be 92 characters"
        );
        assert!(token_hex.starts_with("12345678"));
        assert!(token_hex.ends_with("55914510010403030101"));

        // Verify it decodes to 46 bytes
        let bytes = hex::decode(&token_hex).unwrap();
        assert_eq!(bytes.len(), 46);
    }
}
