//! Configuration types for profiles and button mappings.

use crate::error::OpenRapooError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;
use uuid::Uuid;

/// A complete configuration profile for the mouse.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    /// Unique identifier for this profile
    pub id: Uuid,
    /// Human-readable profile name
    pub name: String,
    /// Button mappings: button code (hex string, e.g. "0x114") → action
    pub mappings: HashMap<String, ButtonAction>,
    /// Whether this is the default (fallback) profile
    pub is_default: bool,
    /// Optional application this profile is associated with (e.g. `"firefox"`, `"gimp"`)
    pub app_association: Option<String>,
}

impl Profile {
    /// Create a new empty profile with a generated UUID.
    pub fn new(name: impl Into<String>) -> Self {
        Profile {
            id: Uuid::new_v4(),
            name: name.into(),
            mappings: HashMap::new(),
            is_default: false,
            app_association: None,
        }
    }

    /// Create a default profile for the Rapoo MT760 Pro with standard Passthrough bindings.
    pub fn default_profile() -> Self {
        let mut mappings = HashMap::new();
        // Standard mouse buttons pass through by default
        mappings.insert("0x110".to_string(), ButtonAction::PassThrough); // Left
        mappings.insert("0x111".to_string(), ButtonAction::PassThrough); // Right
        mappings.insert("0x112".to_string(), ButtonAction::PassThrough); // Middle
        mappings.insert("0x113".to_string(), ButtonAction::PassThrough); // Side Back
        mappings.insert("0x114".to_string(), ButtonAction::PassThrough); // Side Forward

        Profile {
            id: Uuid::nil(),
            name: "Padrão (Passthrough)".to_string(),
            mappings,
            is_default: true,
            app_association: None,
        }
    }

    /// Look up the configured action for a button code string (e.g., "0x114" or "276").
    pub fn get_action(&self, button_code: &str) -> Option<&ButtonAction> {
        self.mappings.get(button_code)
    }
}

/// An action to be performed when a button is pressed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ButtonAction {
    /// Send a mouse button event
    MouseButton { button: MouseButton },
    /// Send a single keyboard key (e.g. "KEY_A", "KEY_ENTER", "a", "Ctrl")
    Key { key: String },
    /// Send a key combination (e.g. ["KEY_LEFTCTRL", "KEY_C"])
    KeyCombo { keys: Vec<String> },
    /// Paste from clipboard (Ctrl+V)
    Paste,
    /// Copy selection to clipboard (Ctrl+C)
    Copy,
    /// Open a terminal emulator
    OpenTerminal,
    /// Close the active window (Alt+F4)
    CloseWindow,
    /// Media: play/pause
    MediaPlayPause,
    /// Media: next track
    MediaNext,
    /// Media: previous track
    MediaPrev,
    /// Media: mute toggle
    MediaMute,
    /// Switch to a specific workspace/virtual desktop
    SwitchWorkspace { number: u32 },
    /// Execute a shell command (validated, no shell injection)
    /// The command must be a list of arguments — no shell interpolation.
    RunCommand { argv: Vec<String> },
    /// Type a predefined text string
    TypeText { text: String },
    /// Disable this button (absorb events, do nothing)
    Disabled,
    /// Keep the default OS behavior (pass through)
    PassThrough,
}

impl ButtonAction {
    /// Validates `RunCommand` to ensure it doesn't attempt empty execution.
    pub fn validate(&self) -> Result<(), OpenRapooError> {
        if let ButtonAction::RunCommand { argv } = self {
            if argv.is_empty() {
                return Err(OpenRapooError::Config(
                    "RunCommand argv cannot be empty".to_string(),
                ));
            }
        }
        Ok(())
    }
}

/// Mouse button constants for remapping.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Button6,
    Button7,
}

/// Storage container for all profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStore {
    /// List of configured profiles
    pub profiles: Vec<Profile>,
    /// ID of the currently active profile
    pub active_profile_id: Uuid,
}

impl Default for ProfileStore {
    fn default() -> Self {
        let default_prof = Profile::default_profile();
        let default_id = default_prof.id;
        ProfileStore {
            profiles: vec![default_prof],
            active_profile_id: default_id,
        }
    }
}

impl ProfileStore {
    /// Load profile store from a JSON file, or return default if it doesn't exist.
    pub fn load_from_file(path: &Path) -> Result<Self, OpenRapooError> {
        if !path.exists() {
            info!("Config file {} not found; using defaults", path.display());
            let store = ProfileStore::default();
            store.save_to_file(path)?;
            return Ok(store);
        }

        let content = std::fs::read_to_string(path)?;
        let store: ProfileStore = serde_json::from_str(&content)?;
        Ok(store)
    }

    /// Save profile store to a JSON file.
    pub fn save_to_file(&self, path: &Path) -> Result<(), OpenRapooError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Return reference to the currently active profile.
    pub fn active_profile(&self) -> &Profile {
        self.profiles
            .iter()
            .find(|p| p.id == self.active_profile_id)
            .or_else(|| self.profiles.iter().find(|p| p.is_default))
            .unwrap_or_else(|| &self.profiles[0])
    }

    /// Set active profile by UUID.
    pub fn set_active(&mut self, id: Uuid) -> bool {
        if self.profiles.iter().any(|p| p.id == id) {
            self.active_profile_id = id;
            true
        } else {
            false
        }
    }

    /// Find profile by associated application name.
    pub fn find_by_app(&self, app_name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| {
            p.app_association
                .as_deref()
                .map(|a| a.eq_ignore_ascii_case(app_name))
                .unwrap_or(false)
        })
    }

    /// Get default config directory path (`$XDG_CONFIG_HOME/openrapoo/profiles.json` or `~/.config/openrapoo/profiles.json`).
    ///
    /// Respects `SUDO_USER` when executed under `sudo` to avoid writing to `/root/.config`.
    pub fn default_config_path() -> PathBuf {
        let sudo_user = std::env::var("SUDO_USER").ok();
        let xdg_config = std::env::var("XDG_CONFIG_HOME").ok();
        let home = std::env::var("HOME").ok();

        Self::resolve_config_path_internal(
            sudo_user.as_deref(),
            xdg_config.as_deref(),
            home.as_deref(),
        )
    }

    /// Internal logic for resolving the configuration path (decoupled for unit tests).
    pub fn resolve_config_path_internal(
        sudo_user: Option<&str>,
        xdg_config: Option<&str>,
        home: Option<&str>,
    ) -> PathBuf {
        // 1. If running under sudo, use real user's home dir
        if let Some(user) = sudo_user {
            let user_trim = user.trim();
            if !user_trim.is_empty() && user_trim != "root" {
                let user_home = PathBuf::from("/home").join(user_trim);
                if user_home.exists() {
                    return user_home.join(".config").join("openrapoo").join("profiles.json");
                }
            }
        }

        // 2. Respect XDG_CONFIG_HOME if set
        if let Some(xdg) = xdg_config {
            let xdg_trim = xdg.trim();
            if !xdg_trim.is_empty() {
                return PathBuf::from(xdg_trim).join("openrapoo").join("profiles.json");
            }
        }

        // 3. Fallback to HOME/.config
        if let Some(h) = home {
            let h_trim = h.trim();
            if !h_trim.is_empty() {
                return PathBuf::from(h_trim).join(".config").join("openrapoo").join("profiles.json");
            }
        }

        // 4. Fallback via dirs crate
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("openrapoo")
            .join("profiles.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_default() {
        let p = Profile::default_profile();
        assert!(p.is_default);
        assert_eq!(p.get_action("0x110"), Some(&ButtonAction::PassThrough));
    }

    #[test]
    fn test_run_command_validation() {
        let valid = ButtonAction::RunCommand {
            argv: vec!["ls".to_string(), "-la".to_string()],
        };
        assert!(valid.validate().is_ok());

        let invalid = ButtonAction::RunCommand { argv: vec![] };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_profile_store_serialization() {
        let store = ProfileStore::default();
        let json = serde_json::to_string(&store).unwrap();
        let decoded: ProfileStore = serde_json::from_str(&json).unwrap();
        assert_eq!(store.active_profile_id, decoded.active_profile_id);
    }
}
