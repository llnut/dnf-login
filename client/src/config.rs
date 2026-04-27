use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::i18n::Language;

/// Controls how a background image is scaled to fit the window,
/// matching the five modes available in Windows desktop wallpaper settings.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum BgFillMode {
    /// Repeat the image in a grid without scaling.
    Tile,
    /// Stretch the image to exactly fill the window, ignoring aspect ratio.
    Stretch,
    /// Scale the image uniformly until it covers the window, then center-crop.
    #[default]
    Fill,
    /// Display the image at its decoded pixel size, centered; bars show for smaller images.
    Center,
    /// Scale the image uniformly to fit within the window; letterbox bars fill the remainder.
    Fit,
}

/// Selects which type of background drives the home screen.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum BgMode {
    #[default]
    Image,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_url: String,
    pub aes_key: String,
    pub language: Language,
    /// Directory scanned for background image files at load time.
    /// Relative to the working directory when the launcher starts.
    /// The serde alias keeps old configs that used `bg_custom_path` valid.
    #[serde(alias = "bg_custom_path", default = "default_pic_path")]
    pub bg_pic_path: String,
    /// How background images are scaled to fill the window.
    pub bg_fill_mode: BgFillMode,
    /// Index of the last selected background image.
    #[serde(default)]
    pub bg_index: usize,
    /// Which background source drives the home screen.
    #[serde(default)]
    pub bg_mode: BgMode,
    /// Index of the last selected background video.
    #[serde(default)]
    pub bg_video_index: usize,
    /// Directory scanned for background video files at load time.
    /// The serde alias keeps old configs that used `bg_video_custom_path` valid.
    #[serde(alias = "bg_video_custom_path", default = "default_vid_path")]
    pub bg_vid_path: String,
    /// Plugin directory path passed to DNF.exe via environment variable.
    pub plugins_path: String,
    /// Controls the DNF_PLUGIN_ENABLED environment variable passed to DNF.exe.
    pub plugin_inject_enabled: bool,
    /// When true, the launcher fetches the game server IP at login
    /// and passes it as the GAME_SERVER_IP environment variable to DNF.exe.
    #[serde(default = "default_true")]
    pub game_server_ip_enabled: bool,
}

fn default_true() -> bool {
    true
}

fn default_pic_path() -> String {
    "assets/pic".to_string()
}

fn default_vid_path() -> String {
    "assets/vid".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            aes_key: String::new(),
            language: Language::default(),
            bg_pic_path: default_pic_path(),
            bg_fill_mode: BgFillMode::Fill,
            bg_index: 0,
            bg_mode: BgMode::Image,
            bg_video_index: 0,
            bg_vid_path: default_vid_path(),
            plugins_path: "plugins".to_string(),
            plugin_inject_enabled: true,
            game_server_ip_enabled: true,
        }
    }
}

impl AppConfig {
    fn config_path() -> Result<PathBuf> {
        let config_path = std::env::current_dir()?.join("Config.toml");
        tracing::debug!("Config file path: {}", config_path.display());
        Ok(config_path)
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            tracing::info!("Config file not found at: {}", path.display());
            let config = Self::default();
            return Ok(config);
        }

        tracing::info!("Loading config from: {}", path.display());
        let content = std::fs::read_to_string(&path)?;

        match toml::from_str::<AppConfig>(&content) {
            Ok(config) => {
                if content.contains("bg_custom_path") || content.contains("bg_video_custom_path") {
                    tracing::info!(
                        "Legacy bg_custom_path / bg_video_custom_path detected; \
                         migrating to bg_pic_path / bg_vid_path on next save"
                    );
                }
                tracing::info!(
                    "Config loaded: server_url={}, aes_key_len={}",
                    config.server_url,
                    config.aes_key.len()
                );
                Ok(config)
            }
            Err(e) => {
                // Move the unreadable file aside before returning so the user
                // can inspect it. Without this, the next save would overwrite
                // their previous settings with a fresh default.
                let backup = path.with_extension("toml.corrupted");
                if let Err(rename_err) = std::fs::rename(&path, &backup) {
                    tracing::error!(
                        "Failed to back up corrupted config from {} to {}: {}",
                        path.display(),
                        backup.display(),
                        rename_err
                    );
                } else {
                    tracing::error!(
                        "Could not parse {}; moved to {} for inspection. \
                         Falling back to defaults; please re-enter your settings.",
                        path.display(),
                        backup.display(),
                    );
                }
                Err(anyhow::anyhow!("Config parse failed: {}", e))
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        tracing::info!("Config saved to: {}", path.display());
        Ok(())
    }

    /// Parse the AES key into 32 bytes.
    /// Only accepts exactly 64 hexadecimal characters (0–9, a–f, A–F).
    pub fn get_aes_key_bytes(&self) -> Result<[u8; 32]> {
        let decoded = hex::decode(&self.aes_key)
            .map_err(|e| anyhow::anyhow!("AES key is not valid hex: {}", e))?;
        if decoded.len() != 32 {
            anyhow::bail!(
                "AES key must decode to exactly 32 bytes (got {})",
                decoded.len()
            );
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&decoded);
        Ok(key)
    }

    pub fn validate(&self) -> Result<()> {
        if self.server_url.is_empty() {
            anyhow::bail!("Server URL must not be empty");
        }
        if !self.server_url.starts_with("http://") && !self.server_url.starts_with("https://") {
            anyhow::bail!("Server URL must begin with http:// or https://");
        }
        if self.aes_key.is_empty() {
            anyhow::bail!("AES key must not be empty");
        }
        if self.aes_key.len() != 64 {
            anyhow::bail!(
                "AES key must be exactly 64 hex characters (got {})",
                self.aes_key.len()
            );
        }
        if !self.aes_key.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("AES key must contain only hex characters (0-9, a-f, A-F)");
        }
        Ok(())
    }

    pub fn is_configured(&self) -> bool {
        !self.server_url.is_empty() && !self.aes_key.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(config.server_url.is_empty());
        assert!(config.aes_key.is_empty());
        assert!(!config.is_configured());
    }

    #[test]
    fn test_validate() {
        let mut config = AppConfig::default();
        // Default config has no URL or key set.
        assert!(config.validate().is_err());

        // Invalid URL scheme.
        config.server_url = "ftp://example.com".to_string();
        assert!(config.validate().is_err());

        // Valid URL but still no AES key.
        config.server_url = "https://example.com".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_aes_key_bytes() {
        let mut config = AppConfig::default();

        // Valid 64-char hex key
        config.aes_key =
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string();
        let key_bytes = config.get_aes_key_bytes().unwrap();
        assert_eq!(key_bytes.len(), 32);
        assert_eq!(key_bytes[0], 0x01);
        assert_eq!(key_bytes[1], 0x23);

        // Too short.
        config.aes_key = "deadbeef".to_string();
        assert!(config.get_aes_key_bytes().is_err());

        // Non-hex characters.
        config.aes_key =
            "gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg".to_string();
        assert!(config.get_aes_key_bytes().is_err());
    }

    #[test]
    fn test_bg_index_defaults_and_alias_compat() {
        // Old configs used `bg_custom_path` and `bg_custom_prepend`; the new
        // schema reads the path through a serde alias and ignores the
        // now-removed prepend flag.
        let toml_str = r#"
server_url = ""
aes_key = ""
language = "English"
bg_custom_path = "assets/bg"
bg_custom_prepend = false
bg_fill_mode = "Fill"
plugins_path = "plugins"
plugin_inject_enabled = true
game_server_ip_enabled = true
"#;
        let config: AppConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.bg_index, 0);
        assert_eq!(config.bg_pic_path, "assets/bg");
        assert_eq!(config.bg_vid_path, "assets/vid");
    }

    #[test]
    fn test_alias_and_canonical_field_conflict_is_rejected() {
        // serde_derive treats an aliased field and its canonical name as the
        // same logical key; declaring both at once is a duplicate. The test
        // pins this behavior so any future move to manual migration surfaces
        // as a test break.
        let toml_str = r#"
server_url = ""
aes_key = ""
language = "English"
bg_custom_path = "assets/bg"
bg_pic_path = "assets/pic"
bg_fill_mode = "Fill"
plugins_path = "plugins"
plugin_inject_enabled = true
game_server_ip_enabled = true
"#;
        assert!(toml::from_str::<AppConfig>(toml_str).is_err());
    }

    #[test]
    fn test_default_config_round_trip_uses_new_field_names() {
        // Verifies that a freshly default-constructed config saves with the
        // new schema, then re-loads correctly.
        let config = AppConfig::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        assert!(serialized.contains("bg_pic_path"));
        assert!(serialized.contains("bg_vid_path"));
        assert!(!serialized.contains("bg_custom_path"));
        assert!(!serialized.contains("bg_custom_prepend"));
        assert!(!serialized.contains("bg_video_custom_path"));
        let parsed: AppConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(parsed.bg_pic_path, "assets/pic");
        assert_eq!(parsed.bg_vid_path, "assets/vid");
    }

    #[test]
    fn test_bg_index_round_trip() {
        let mut config = AppConfig::default();
        config.bg_index = 7;
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.bg_index, 7);
    }
}
