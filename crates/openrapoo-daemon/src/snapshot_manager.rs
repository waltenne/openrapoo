//! Snapshot manager for pre-write hardware state backup and rollback.

use anyhow::{anyhow, Result};
use openrapoo_core::ipc::HardwareConfigSnapshot;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use tracing::info;

/// Returns the snapshot directory path `~/.config/openrapoo/snapshots/`
pub fn snapshots_dir() -> PathBuf {
    if cfg!(test) {
        return std::env::temp_dir().join("openrapoo-test-snapshots");
    }
    let base = dirs::home_dir()
        .map(|h| h.join(".config").join("openrapoo"))
        .unwrap_or_else(std::env::temp_dir);
    base.join("snapshots")
}

/// Resolves path for device snapshot file: `device_snapshot_<sanitized_id>.json`
pub fn snapshot_file_path(device_id: &str) -> PathBuf {
    let sanitized = device_id
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>();
    snapshots_dir().join(format!("device_snapshot_{sanitized}.json"))
}

/// Save hardware config snapshot to disk.
pub fn save_snapshot(snapshot: &HardwareConfigSnapshot) -> Result<PathBuf> {
    let dir = snapshots_dir();
    fs::create_dir_all(&dir)?;

    let path = snapshot_file_path(&snapshot.device_id);
    let file = File::create(&path)?;
    let writer = BufWriter::new(file);

    serde_json::to_writer_pretty(writer, snapshot)?;
    info!(
        "Saved hardware config snapshot for device `{}` to `{}`",
        snapshot.device_id,
        path.display()
    );

    Ok(path)
}

/// Load hardware config snapshot from disk.
pub fn load_snapshot(device_id: &str) -> Result<HardwareConfigSnapshot> {
    let path = snapshot_file_path(device_id);
    if !path.exists() {
        return Err(anyhow!(
            "No hardware snapshot found for device `{device_id}`"
        ));
    }

    let file = File::open(&path)?;
    let reader = BufReader::new(file);

    let snapshot: HardwareConfigSnapshot = serde_json::from_reader(reader)?;
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_save_load() {
        let snapshot = HardwareConfigSnapshot {
            device_id: "test_device_123".to_string(),
            transport: "USB".to_string(),
            original_dpi: 1200,
            original_polling_rate: 1000,
            timestamp: 123456789,
        };

        let path = save_snapshot(&snapshot).unwrap();
        assert!(path.exists());

        let loaded = load_snapshot("test_device_123").unwrap();
        assert_eq!(loaded, snapshot);

        let _ = fs::remove_file(path);
    }
}
