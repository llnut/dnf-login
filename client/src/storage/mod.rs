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

#[derive(Deserialize)]
#[serde(untagged)]
enum CredentialFile {
    Multiple(Vec<StoredCredentials>),
    Legacy(StoredCredentials),
}

/// Credential storage using Windows DPAPI.
pub struct CredentialStorage {
    config_path: std::path::PathBuf,
}

impl CredentialStorage {
    fn read_entries(&self) -> Result<Vec<StoredCredentials>> {
        if !self.config_path.exists() {
            return Ok(Vec::new());
        }
        let json = std::fs::read_to_string(&self.config_path)?;
        Ok(match serde_json::from_str::<CredentialFile>(&json)? {
            CredentialFile::Multiple(entries) => entries,
            CredentialFile::Legacy(entry) => vec![entry],
        })
    }

    pub fn remove(&self, username: &str) -> Result<()> {
        let mut entries = self.read_entries()?;
        entries.retain(|entry| entry.username != username);
        std::fs::write(&self.config_path, serde_json::to_vec(&entries)?)?;
        Ok(())
    }

    pub fn load_accounts(&self) -> Result<Vec<(String, String)>> {
        self.read_entries()?.into_iter().filter(|entry| entry.remember)
            .map(Self::decrypt_entry).collect()
    }
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
            return self.remove(username);
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

            let mut entries = self.read_entries()?;
            entries.retain(|entry| entry.username != username);
            entries.push(credentials);
            let json = serde_json::to_string(&entries)?;
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
        let entry = self.read_entries()?.pop()
            .ok_or_else(|| anyhow::anyhow!("No saved accounts"))?;
        Self::decrypt_entry(entry)
    }

    #[cfg(target_os = "windows")]
    fn decrypt_entry(credentials: StoredCredentials) -> Result<(String, String)> {
        use std::ptr;
        use windows::Win32::Foundation::{HLOCAL, LocalFree};
        use windows::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB, CryptUnprotectData};

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
    fn decrypt_entry(_credentials: StoredCredentials) -> Result<(String, String)> {
        anyhow::bail!("Credential storage only supported on Windows")
    }

    #[cfg(not(target_os = "windows"))]
    pub fn load(&self) -> Result<(String, String)> {
        anyhow::bail!("Credential storage only supported on Windows")
    }

    pub fn has_saved_credentials(&self) -> bool {
        self.config_path.exists()
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    #[test]
    fn multiple_accounts_migrate_update_and_remove_independently() {
        let path = std::env::temp_dir().join(format!("dnf-accounts-{}-{}.json",
            std::process::id(), std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let storage = CredentialStorage { config_path: path.clone() };
        storage.save("first", "password-one", true).unwrap();
        let entries = storage.read_entries().unwrap();
        // Simulate the old single-account file before loading and adding another account.
        std::fs::write(&path, serde_json::to_vec(&entries[0]).unwrap()).unwrap();
        assert_eq!(storage.load_accounts().unwrap(), vec![("first".into(), "password-one".into())]);
        storage.save("second", "password-two", true).unwrap();
        storage.save("first", "updated-password", true).unwrap();
        let accounts = storage.load_accounts().unwrap();
        assert_eq!(accounts.len(), 2);
        assert!(accounts.contains(&("first".into(), "updated-password".into())));
        storage.save("first", "", false).unwrap();
        assert_eq!(storage.load_accounts().unwrap(), vec![("second".into(), "password-two".into())]);
        storage.remove("second").unwrap();
        assert!(storage.load_accounts().unwrap().is_empty());
        std::fs::remove_file(path).unwrap();
    }
}
