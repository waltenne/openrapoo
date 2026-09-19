//! Persistent application settings for OpenRapoo GUI.
//! Handles saving and loading application-wide preferences (such as language)
//! to `$XDG_CONFIG_HOME/openrapoo/settings.json` or `~/.config/openrapoo/settings.json`.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Root configuration structure stored in `settings.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppSettings {
    /// Saved locale code (e.g., "pt-BR" or "en-US").
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_language() -> String {
    "pt-BR".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: default_language(),
        }
    }
}

impl AppSettings {
    /// Returns the default settings path: `$XDG_CONFIG_HOME/openrapoo/settings.json` or `~/.config/openrapoo/settings.json`.
    pub fn default_settings_path() -> PathBuf {
        let sudo_user = std::env::var("SUDO_USER").ok();
        let xdg_config = std::env::var("XDG_CONFIG_HOME").ok();
        let home = std::env::var("HOME").ok();

        Self::resolve_settings_path_internal(
            sudo_user.as_deref(),
            xdg_config.as_deref(),
            home.as_deref(),
        )
    }

    /// Internal helper for resolving settings path, decoupled for unit tests.
    pub fn resolve_settings_path_internal(
        sudo_user: Option<&str>,
        xdg_config: Option<&str>,
        home: Option<&str>,
    ) -> PathBuf {
        if let Some(user) = sudo_user {
            let user_trim = user.trim();
            if !user_trim.is_empty() && user_trim != "root" {
                let user_home = PathBuf::from("/home").join(user_trim);
                if user_home.exists() {
                    return user_home
                        .join(".config")
                        .join("openrapoo")
                        .join("settings.json");
                }
            }
        }

        if let Some(xdg) = xdg_config {
            let xdg_trim = xdg.trim();
            if !xdg_trim.is_empty() {
                return PathBuf::from(xdg_trim)
                    .join("openrapoo")
                    .join("settings.json");
            }
        }

        if let Some(h) = home {
            let h_trim = h.trim();
            if !h_trim.is_empty() {
                return PathBuf::from(h_trim)
                    .join(".config")
                    .join("openrapoo")
                    .join("settings.json");
            }
        }

        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("openrapoo")
            .join("settings.json")
    }

    /// Loads settings from file. If file does not exist or JSON is invalid,
    /// returns default `AppSettings` without panicking.
    pub fn load_from_file(path: &Path) -> Self {
        if !path.exists() {
            info!("Settings file not found at {}. Using defaults.", path.display());
            return Self::default();
        }

        match fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str::<AppSettings>(&content) {
                Ok(settings) => {
                    info!("Successfully loaded settings from {}", path.display());
                    settings
                }
                Err(err) => {
                    warn!(
                        "Failed to parse settings JSON at {}: {err}. Falling back to default settings.",
                        path.display()
                    );
                    Self::default()
                }
            },
            Err(err) => {
                warn!(
                    "Failed to read settings file at {}: {err}. Falling back to default settings.",
                    path.display()
                );
                Self::default()
            }
        }
    }

    /// Atomic save of settings to disk. Writes to a temporary `.tmp` file before renaming.
    pub fn save_to_file(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                let msg = format!("Failed to create settings directory {}: {err}", parent.display());
                warn!("{msg}");
                return Err(msg);
            }
        }

        let tmp_path = path.with_extension("tmp");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {e}"))?;

        fs::write(&tmp_path, &json)
            .map_err(|e| format!("Failed to write temp settings file {}: {e}", tmp_path.display()))?;

        fs::rename(&tmp_path, path)
            .map_err(|e| format!("Failed to atomically rename settings file to {}: {e}", path.display()))?;

        info!("Atomically saved app settings to {}", path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.language, "pt-BR");
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = std::env::temp_dir().join("openrapoo_test_settings");
        let settings_file = temp_dir.join("settings.json");

        let mut settings = AppSettings::default();
        settings.language = "en-US".to_string();

        assert!(settings.save_to_file(&settings_file).is_ok());

        let loaded = AppSettings::load_from_file(&settings_file);
        assert_eq!(loaded.language, "en-US");

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_load_corrupted_json_falls_back_to_default() {
        let temp_dir = std::env::temp_dir().join("openrapoo_test_corrupt");
        let settings_file = temp_dir.join("settings.json");

        let _ = fs::create_dir_all(&temp_dir);
        let _ = fs::write(&settings_file, "{ INVALID JSON }");

        let loaded = AppSettings::load_from_file(&settings_file);
        assert_eq!(loaded.language, "pt-BR");

        let _ = fs::remove_dir_all(temp_dir);
    }
}

