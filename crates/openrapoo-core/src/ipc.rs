//! IPC communication protocol data structures between GUI and daemon.

use crate::config::ButtonAction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Resolves the default Unix domain socket path for OpenRapoo daemon.
/// Tries `$XDG_RUNTIME_DIR/openrapoo.sock`, then `~/.config/openrapoo/daemon.sock`, then `/tmp/openrapoo-user-$UID.sock`.
pub fn default_socket_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let trimmed = runtime_dir.trim();
        if !trimmed.is_empty() {
            let path = PathBuf::from(trimmed);
            if path.is_dir() {
                return path.join("openrapoo.sock");
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        let config_dir = home.join(".config").join("openrapoo");
        if std::fs::create_dir_all(&config_dir).is_ok() {
            return config_dir.join("daemon.sock");
        }
    }

    let uid = unsafe { libc::getuid() };
    PathBuf::from(format!("/tmp/openrapoo-user-{uid}.sock"))
}

/// Transactional request sent by GUI to daemon when applying a profile configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApplyProfileRequest {
    pub transaction_id: String,
    pub device_name: String,
    pub profile_id: Uuid,
    pub profile_version: u32,
    pub actions: HashMap<String, ButtonAction>,
    pub timestamp: u64,
}

/// Status outcome of an IPC application transaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApplyStatus {
    Received,
    Applying,
    AppliedSucceeded,
    Failed { reason: String },
    DeviceDisconnected,
    FeatureUnsupported,
}

/// Specifies where/how the remapping is enforced.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApplyMode {
    /// Remapped dynamically in Linux user-space via evdev grab + uinput virtual device
    SoftwareSession,
    /// Written directly to mouse EEPROM/firmware (if hardware persistence is supported)
    HardwareFirmware,
}

/// Transactional response returned by daemon to GUI after applying profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApplyProfileResponse {
    pub transaction_id: String,
    pub status: ApplyStatus,
    pub active_rules_count: usize,
    pub applied_mode: ApplyMode,
    pub daemon_pid: u32,
    pub message: String,
}

/// Transactional request sent by GUI to daemon when updating DPI sensitivity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetDpiRequest {
    pub transaction_id: String,
    pub device_id: String,
    pub dpi: u32,
    pub gear: u8,
    pub timestamp: u64,
}

/// Transactional request sent by GUI to daemon when updating USB Polling Rate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SetPollingRateRequest {
    pub transaction_id: String,
    pub device_id: String,
    pub rate_hz: u32,
    pub timestamp: u64,
}

/// Hardware configuration snapshot stored for safe transactional rollback.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareConfigSnapshot {
    pub device_id: String,
    pub transport: String,
    pub original_dpi: u32,
    pub original_polling_rate: u32,
    pub timestamp: u64,
}

/// Hardware operation response returned by daemon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareOpResponse {
    pub transaction_id: String,
    pub status: ApplyStatus,
    pub confirmed_dpi: Option<u32>,
    pub confirmed_polling_rate: Option<u32>,
    pub message: String,
}

/// Diagnostic daemon status summary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DaemonStatusInfo {
    pub pid: u32,
    pub version: String,
    pub active_device_nodes: Vec<String>,
    pub active_rules_count: usize,
    pub is_grabbed: bool,
    pub socket_path: String,
}

/// IPC Message envelope sent from client (GUI / CLI) to daemon.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum DaemonIpcMessage {
    ApplyProfile(ApplyProfileRequest),
    SetDpi(SetDpiRequest),
    SetPollingRate(SetPollingRateRequest),
    RestoreSnapshot { device_id: String },
    ReadHardwareState { device_id: String },
    Ping,
    GetStatus,
}

/// IPC Response envelope returned by daemon to client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum DaemonIpcResponse {
    ApplyProfile(ApplyProfileResponse),
    HardwareOp(HardwareOpResponse),
    HardwareState {
        confirmed_dpi: u32,
        confirmed_polling_rate: u32,
        transport: String,
    },
    Pong {
        pid: u32,
        version: String,
        active_rules: usize,
    },
    StatusInfo(DaemonStatusInfo),
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_message_serialization() {
        let req = ApplyProfileRequest {
            transaction_id: "tx-12345".to_string(),
            device_name: "Rapoo MT760 Pro".to_string(),
            profile_id: Uuid::nil(),
            profile_version: 1,
            actions: HashMap::new(),
            timestamp: 1600000000,
        };

        let msg = DaemonIpcMessage::ApplyProfile(req);
        let json = serde_json::to_string(&msg).unwrap();
        let decoded: DaemonIpcMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_socket_path_resolution() {
        let path = default_socket_path();
        assert!(path.to_string_lossy().contains("openrapoo"));
    }
}
