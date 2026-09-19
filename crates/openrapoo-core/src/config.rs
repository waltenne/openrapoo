//! Configuration types for profiles and button mappings.

use crate::error::OpenRapooError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;
use uuid::Uuid;

fn default_dpi() -> u32 {
    1200
}

fn default_polling_rate() -> u32 {
    1000
}

/// Validates whether a DPI value is supported by Rapoo MT760 Pro hardware.
pub fn validate_dpi(dpi: u32) -> Result<(), OpenRapooError> {
    const VALID_DPI_LEVELS: &[u32] = &[800, 1000, 1200, 1600, 2400, 3200, 4000];
    if VALID_DPI_LEVELS.contains(&dpi) || (dpi >= 50 && dpi <= 26000 && dpi % 50 == 0) {
        Ok(())
    } else {
        Err(OpenRapooError::Config(format!(
            "DPI {dpi} is invalid. Must be between 50 and 26000 in steps of 50."
        )))
    }
}

/// Validates whether a Polling Rate (in Hz) is supported by Rapoo MT760 Pro hardware.
pub fn validate_polling_rate(rate_hz: u32) -> Result<(), OpenRapooError> {
    const VALID_POLLING_RATES: &[u32] = &[125, 250, 500, 1000];
    if VALID_POLLING_RATES.contains(&rate_hz) {
        Ok(())
    } else {
        Err(OpenRapooError::Config(format!(
            "Polling rate {rate_hz}Hz is unsupported. Supported values: 125, 250, 500, 1000 Hz."
        )))
    }
}

/// A complete configuration profile for the mouse.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    /// Unique identifier for this profile
    pub id: Uuid,
    /// Human-readable profile name
    pub name: String,
    /// Button mappings: button code (hex string, e.g. "0x114") → action
    pub mappings: HashMap<String, ButtonAction>,
    /// Target DPI sensitivity setting (e.g. 800, 1000, 1200, 1600, 2400, 3200, 4000)
    #[serde(default = "default_dpi")]
    pub dpi: u32,
    /// Target USB polling rate setting in Hz (e.g. 125, 250, 500, 1000)
    #[serde(default = "default_polling_rate")]
    pub polling_rate: u32,
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
            dpi: 1200,
            polling_rate: 1000,
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
            dpi: 1200,
            polling_rate: 1000,
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
    /// Execute a recorded key/button macro sequence
    Macro { macro_id: Uuid },
    /// Disable this button (absorb events, do nothing)
    Disabled,
    /// Keep the default OS behavior (pass through)
    PassThrough,
}

/// Event step in a macro sequence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroEvent {
    pub key: String,
    pub is_press: bool,
    pub delay_ms: u64,
}

/// Macro definition for keystroke automation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroDefinition {
    pub id: Uuid,
    pub name: String,
    pub events: Vec<MacroEvent>,
    pub repeat_count: u32,
}

impl MacroDefinition {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            events: Vec::new(),
            repeat_count: 1,
        }
    }
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

    /// Return mutable reference to the currently active profile.
    pub fn active_profile_mut(&mut self) -> &mut Profile {
        let active_id = self.active_profile_id;
        if let Some(pos) = self.profiles.iter().position(|p| p.id == active_id) {
            &mut self.profiles[pos]
        } else if let Some(pos) = self.profiles.iter().position(|p| p.is_default) {
            &mut self.profiles[pos]
        } else {
            &mut self.profiles[0]
        }
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

    /// Add a new profile and return its UUID.
    pub fn add_profile(&mut self, mut profile: Profile) -> Uuid {
        let id = Uuid::new_v4();
        profile.id = id;
        profile.is_default = false;
        self.profiles.push(profile);
        id
    }

    /// Duplicate an existing profile by ID.
    pub fn duplicate_profile(&mut self, id: Uuid, new_name: &str) -> Option<Profile> {
        let target = self.profiles.iter().find(|p| p.id == id)?.clone();
        let mut dup = target;
        dup.id = Uuid::new_v4();
        dup.name = new_name.to_string();
        dup.is_default = false;
        self.profiles.push(dup.clone());
        Some(dup)
    }

    /// Rename a profile by ID.
    pub fn rename_profile(&mut self, id: Uuid, new_name: &str) -> bool {
        if let Some(prof) = self.profiles.iter_mut().find(|p| p.id == id) {
            prof.name = new_name.to_string();
            true
        } else {
            false
        }
    }

    /// Delete a non-default profile by ID.
    pub fn delete_profile(&mut self, id: Uuid) -> bool {
        if self.profiles.len() <= 1 {
            return false;
        }
        if let Some(idx) = self
            .profiles
            .iter()
            .position(|p| p.id == id && !p.is_default)
        {
            self.profiles.remove(idx);
            if self.active_profile_id == id {
                self.active_profile_id = self.profiles[0].id;
            }
            true
        } else {
            false
        }
    }

    /// Export a profile to JSON string.
    pub fn export_profile_json(&self, id: Uuid) -> Option<String> {
        let prof = self.profiles.iter().find(|p| p.id == id)?;
        serde_json::to_string_pretty(prof).ok()
    }

    /// Import a profile from JSON string and add it to store.
    pub fn import_profile_json(&mut self, json: &str) -> Result<Uuid, OpenRapooError> {
        let mut prof: Profile = serde_json::from_str(json)
            .map_err(|e| OpenRapooError::Config(format!("Invalid profile JSON: {e}")))?;
        prof.id = Uuid::new_v4();
        prof.is_default = false;
        let id = prof.id;
        self.profiles.push(prof);
        Ok(id)
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
                    return user_home
                        .join(".config")
                        .join("openrapoo")
                        .join("profiles.json");
                }
            }
        }

        // 2. Respect XDG_CONFIG_HOME if set
        if let Some(xdg) = xdg_config {
            let xdg_trim = xdg.trim();
            if !xdg_trim.is_empty() {
                return PathBuf::from(xdg_trim)
                    .join("openrapoo")
                    .join("profiles.json");
            }
        }

        // 3. Fallback to HOME/.config
        if let Some(h) = home {
            let h_trim = h.trim();
            if !h_trim.is_empty() {
                return PathBuf::from(h_trim)
                    .join(".config")
                    .join("openrapoo")
                    .join("profiles.json");
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

    #[test]
    fn test_profile_management_ops() {
        let mut store = ProfileStore::default();
        let original_id = store.active_profile_id;

        // Duplicate
        let dup = store
            .duplicate_profile(original_id, "Perfil Cópia")
            .expect("dup");
        assert_eq!(dup.name, "Perfil Cópia");
        assert_eq!(store.profiles.len(), 2);

        // Rename
        assert!(store.rename_profile(dup.id, "Perfil Renomeado"));
        assert_eq!(
            store.profiles.iter().find(|p| p.id == dup.id).unwrap().name,
            "Perfil Renomeado"
        );

        // Export & Import
        let json = store.export_profile_json(dup.id).expect("export");
        let imported_id = store.import_profile_json(&json).expect("import");
        assert_eq!(store.profiles.len(), 3);

        // Delete
        assert!(store.delete_profile(imported_id));
        assert_eq!(store.profiles.len(), 2);
    }
}
