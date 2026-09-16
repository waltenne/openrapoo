//! Input event types and classification for Rapoo mouse events.
//!
//! This module provides a high-level abstraction over raw evdev events,
//! helping to identify which physical button or action produced each event.

use serde::{Deserialize, Serialize};

/// A decoded input event from the mouse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEvent {
    /// Monotonic timestamp in microseconds
    pub timestamp_us: u64,
    /// The type of event
    pub kind: MouseEventKind,
    /// Raw evdev event type code
    pub raw_type: u16,
    /// Raw evdev event code
    pub raw_code: u16,
    /// Raw evdev event value
    pub raw_value: i32,
}

/// Classification of a mouse event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseEventKind {
    /// A button was pressed or released
    Button(ButtonEvent),
    /// Mouse moved in X or Y direction
    Movement { axis: Axis, delta: i32 },
    /// Scroll wheel turned
    Scroll { axis: ScrollAxis, delta: i32 },
    /// Synchronisation event (EV_SYN)
    Sync,
    /// Unknown event type
    Unknown,
}

/// A button press or release event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ButtonEvent {
    /// The button that was pressed or released
    pub button: ButtonCode,
    /// Whether the button is pressed (true) or released (false)
    pub pressed: bool,
}

/// Identified mouse buttons.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonCode {
    Left,
    Right,
    Middle,
    /// Side button — back navigation (BTN_SIDE, code 275)
    SideBack,
    /// Side button — forward navigation (BTN_EXTRA, code 276)
    SideForward,
    /// Any other button with its raw code
    Other(u16),
}

impl ButtonCode {
    /// Parse from a raw evdev KEY/BTN code.
    pub fn from_raw(code: u16) -> Self {
        match code {
            0x110 => ButtonCode::Left,   // BTN_LEFT
            0x111 => ButtonCode::Right,  // BTN_RIGHT
            0x112 => ButtonCode::Middle, // BTN_MIDDLE
            0x113 => ButtonCode::SideBack,    // BTN_SIDE
            0x114 => ButtonCode::SideForward, // BTN_EXTRA
            other => ButtonCode::Other(other),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ButtonCode::Left => "Left Click",
            ButtonCode::Right => "Right Click",
            ButtonCode::Middle => "Middle Click / Scroll Press",
            ButtonCode::SideBack => "Side Back (BTN_SIDE)",
            ButtonCode::SideForward => "Side Forward (BTN_EXTRA)",
            ButtonCode::Other(_) => "Unknown Button",
        }
    }
}

/// Movement axis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
}

/// Scroll axis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrollAxis {
    Vertical,
    Horizontal,
}
