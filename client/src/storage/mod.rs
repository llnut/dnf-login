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

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredentials {
    pub username: String,
    pub password_encrypted: Vec<u8>,
    pub remember: bool,
}

/// Credential storage using Windows DPAPI.
pub struct CredentialStorage {
    config_path: std::path::PathBuf,
}

impl CredentialStorage {
    pub fn new() -> Result<Self> {
        let config_path = Self::credentials_path()?;
        tracing::debug!("Credentials file path: {}", config_path.display());
        Ok(Self { config_path })
    }

    #[cfg(target_os = "windows")]
    fn credentials_path() -> Result<std::path::PathBuf> {
        let appdata = std::env::var("APPDATA")
            .map_err(|_| anyhow::anyhow!("APPDATA environment variable not set"))?;
        let dir = std::path::PathBuf::from(appdata).join("DNF Login");
        std::fs::create_dir_all(&dir)?;
        Ok(dir.join("credentials.dat"))
    }

    #[cfg(not(target_os = "windows"))]
    fn credentials_path() -> Result<std::path::PathBuf> {
        Ok(std::env::current_dir()?.join("credentials.dat"))
    }

    #[cfg(target_os = "windows")]
    pub fn save(&self, username: &str, password: &str, remember: bool) -> Result<()> {
        use std::ptr;
        use windows::Win32::Foundation::{HLOCAL, LocalFree};
        use windows::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB, CryptProtectData};
        use windows::core::PCWSTR;

        if !remember {
            let _ = std::fs::remove_file(&self.config_path);
            return Ok(());
        }

        let password_bytes = password.as_bytes();

        unsafe {
            let data_in = CRYPT_INTEGER_BLOB {
                cbData: password_bytes.len() as u32,
                pbData: password_bytes.as_ptr() as *mut u8,
            };

            let mut data_out = CRYPT_INTEGER_BLOB {
                cbData: 0,
                pbData: ptr::null_mut(),
            };

            let result =
                CryptProtectData(&data_in, PCWSTR::null(), None, None, None, 0, &mut data_out);

            if result.is_err() {
                anyhow::bail!("Failed to encrypt password with DPAPI");
            }

            let encrypted =
                std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec();

            let _ = LocalFree(Some(HLOCAL(data_out.pbData as *mut _)));

            let credentials = StoredCredentials {
                username: username.to_string(),
                password_encrypted: encrypted,
                remember,
            };

            let json = serde_json::to_string(&credentials)?;
            std::fs::write(&self.config_path, json)?;

            tracing::info!("Credentials saved");
            Ok(())
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn save(&self, _username: &str, _password: &str, _remember: bool) -> Result<()> {
        anyhow::bail!("Credential storage only supported on Windows")
    }

    #[cfg(target_os = "windows")]
    pub fn load(&self) -> Result<(String, String)> {
        use std::ptr;
        use windows::Win32::Foundation::{HLOCAL, LocalFree};
        use windows::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB, CryptUnprotectData};

        let json = std::fs::read_to_string(&self.config_path)?;
        let credentials: StoredCredentials = serde_json::from_str(&json)?;

        if !credentials.remember {
            anyhow::bail!("User chose not to remember credentials");
        }

        unsafe {
            let data_in = CRYPT_INTEGER_BLOB {
                cbData: credentials.password_encrypted.len() as u32,
                pbData: credentials.password_encrypted.as_ptr() as *mut u8,
            };

            let mut data_out = CRYPT_INTEGER_BLOB {
                cbData: 0,
                pbData: ptr::null_mut(),
            };

            let result = CryptUnprotectData(&data_in, None, None, None, None, 0, &mut data_out);

            if result.is_err() {
                anyhow::bail!("Failed to decrypt password with DPAPI");
            }

            // Copy data before freeing the DPAPI buffer so LocalFree is always called.
            let decrypted_vec =
                std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec();

            let _ = LocalFree(Some(HLOCAL(data_out.pbData as *mut _)));

            let password = String::from_utf8(decrypted_vec)?;

            tracing::info!("Credentials loaded");
            Ok((credentials.username, password))
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn load(&self) -> Result<(String, String)> {
        anyhow::bail!("Credential storage only supported on Windows")
    }

    pub fn has_saved_credentials(&self) -> bool {
        self.config_path.exists()
    }
}
