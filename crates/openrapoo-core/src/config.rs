//! Configuration types for profiles and button mappings.
//!
//! NOTE: This module contains data structures only. Actual profile management
//! (CRUD, persistence) is implemented in the daemon crate (Phase 4).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A complete configuration profile for the mouse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Unique identifier for this profile
    pub id: Uuid,
    /// Human-readable profile name
    pub name: String,
    /// Button mappings: button code (hex string) → action
    pub mappings: HashMap<String, ButtonAction>,
    /// Whether this is the default (fallback) profile
    pub is_default: bool,
    /// Optional application this profile is associated with
    /// (e.g. `"firefox"`, `"gimp"`)
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
}

/// An action to be performed when a button is pressed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ButtonAction {
    /// Send a mouse button event
    MouseButton { button: MouseButton },
    /// Send a single keyboard key
    Key { key: String },
    /// Send a key combination (e.g. Ctrl+C)
    KeyCombo { keys: Vec<String> },
    /// Paste from clipboard
    Paste,
    /// Copy selection to clipboard
    Copy,
    /// Open a terminal emulator
    OpenTerminal,
    /// Close the active window
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

/// Mouse button constants for remapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
