//! Error types for OpenRapoo.

use thiserror::Error;

/// Main error type for all OpenRapoo operations.
#[derive(Error, Debug)]
pub enum OpenRapooError {
    /// Cannot access a device file (permissions or not found)
    #[error("Device access error: {0}")]
    DeviceAccess(String),

    /// Device not found on system
    #[error("No Rapoo device found. Is the mouse connected?")]
    DeviceNotFound,

    /// IO error from std::io
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Error parsing HID data
    #[error("HID parse error: {0}")]
    HidParse(String),

    /// evdev-related error
    #[error("evdev error: {0}")]
    Evdev(String),

    /// Permission denied — usually needs udev rules or group membership
    #[error("Permission denied: {0}\n\nTip: run `sudo cp udev/99-openrapoo.rules /etc/udev/rules.d/ && sudo udevadm control --reload-rules && sudo usermod -aG input $USER` then log out and back in.")]
    PermissionDenied(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Feature not yet supported
    #[error("Feature not yet supported: {0}")]
    NotSupported(String),
}

