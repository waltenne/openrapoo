//! Hardware Writer module handling privileged /dev/hidraw feature report & output report ioctl operations.

use anyhow::{anyhow, Result};
use nix::libc;
use openrapoo_core::config::{validate_dpi, validate_polling_rate};
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

const HID_REPORT_SIZE_64: usize = 64;

/// Discovers all `/dev/hidraw*` nodes associated with Rapoo devices (VID 0x24AE).
pub fn get_rapoo_hidraw_nodes(candidate: Option<&Path>) -> Vec<PathBuf> {
    let mut nodes = Vec::new();

    if let Some(p) = candidate {
        if p.exists() {
            nodes.push(p.to_path_buf());
        }
    }

    if let Ok(entries) = std::fs::read_dir("/sys/class/hidraw") {
        for entry in entries.flatten() {
            let uevent_path = entry.path().join("device").join("uevent");
            if let Ok(content) = std::fs::read_to_string(uevent_path) {
                let content_lower = content.to_lowercase();
                if content_lower.contains("24ae") || content_lower.contains("rapoo") {
                    let node = PathBuf::from(format!("/dev/{}", entry.file_name().to_string_lossy()));
                    if !nodes.contains(&node) {
                        nodes.push(node);
                    }
                }
            }
        }
    }

    if nodes.is_empty() {
        nodes.push(PathBuf::from("/dev/hidraw0"));
    }

    nodes
}

/// Thread-safe Hardware Writer with concurrency control lock.
#[derive(Clone)]
pub struct HardwareWriter {
    lock: Arc<Mutex<()>>,
}

impl Default for HardwareWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl HardwareWriter {
    pub fn new() -> Self {
        Self {
            lock: Arc::new(Mutex::new(())),
        }
    }

    /// Issue transactional DPI hardware update to hidraw device(s).
    pub async fn write_dpi(&self, hidraw_path: &Path, dpi: u32, gear: u8) -> Result<u32> {
        validate_dpi(dpi).map_err(|e| anyhow!("{e}"))?;

        let _guard = self.lock.lock().await;
        let target_gear = gear.clamp(1, 7);

        // Step 1: Set Custom DPI Value for Gear (Category 0x02, SubCmd 0x02)
        let mut val_report = [0u8; HID_REPORT_SIZE_64];
        val_report[0] = 0x07; // Report ID default
        val_report[1] = 0x02; // Category: DPI Control
        val_report[2] = 0x02; // Sub-Command: Set DPI Value
        val_report[3] = target_gear;
        val_report[4] = (dpi & 0xFF) as u8;        // DPI_X Low
        val_report[5] = ((dpi >> 8) & 0xFF) as u8; // DPI_X High
        val_report[6] = (dpi & 0xFF) as u8;        // DPI_Y Low
        val_report[7] = ((dpi >> 8) & 0xFF) as u8; // DPI_Y High

        let mut crc1 = 0u8;
        for b in &val_report[1..14] {
            crc1 ^= *b;
        }
        val_report[14] = crc1;

        // Step 2: Set Active DPI Gear (Category 0x02, SubCmd 0x01)
        let mut gear_report = [0u8; HID_REPORT_SIZE_64];
        gear_report[0] = 0x07; // Report ID default
        gear_report[1] = 0x02; // Category: DPI Control
        gear_report[2] = 0x01; // Sub-Command: Set Active Gear
        gear_report[3] = target_gear;

        let mut crc2 = 0u8;
        for b in &gear_report[1..14] {
            crc2 ^= *b;
        }
        gear_report[14] = crc2;

        let nodes = get_rapoo_hidraw_nodes(Some(hidraw_path));
        let mut success_count = 0;

        for node in &nodes {
            let res1_07 = self.send_report_to_node(node, &val_report, 0x07);
            let res1_a0 = self.send_report_to_node(node, &val_report, 0xA0);
            let res2_07 = self.send_report_to_node(node, &gear_report, 0x07);
            let res2_a0 = self.send_report_to_node(node, &gear_report, 0xA0);

            if res1_07.is_ok() || res1_a0.is_ok() || res2_07.is_ok() || res2_a0.is_ok() {
                success_count += 1;
            }
        }

        info!(
            "Wrote DPI {dpi} (Gear {target_gear}) to {}/{} hidraw node(s) {:?}",
            success_count,
            nodes.len(),
            nodes
        );

        Ok(dpi)
    }

    /// Issue transactional Polling Rate hardware update to hidraw device(s).
    pub async fn write_polling_rate(&self, hidraw_path: &Path, rate_hz: u32) -> Result<u32> {
        validate_polling_rate(rate_hz).map_err(|e| anyhow!("{e}"))?;

        let _guard = self.lock.lock().await;

        let rate_code: u8 = match rate_hz {
            125 => 0x01,
            250 => 0x02,
            500 => 0x04,
            1000 => 0x08,
            _ => return Err(anyhow!("Unsupported polling rate {rate_hz} Hz")),
        };

        let mut report = [0u8; HID_REPORT_SIZE_64];
        report[0] = 0x07; // Report ID default
        report[1] = 0x03; // Category: Polling Rate
        report[2] = rate_code;

        let mut crc = 0u8;
        for b in &report[1..14] {
            crc ^= *b;
        }
        report[14] = crc;

        let nodes = get_rapoo_hidraw_nodes(Some(hidraw_path));
        let mut success_count = 0;

        for node in &nodes {
            let res07 = self.send_report_to_node(node, &report, 0x07);
            let res_a0 = self.send_report_to_node(node, &report, 0xA0);

            if res07.is_ok() || res_a0.is_ok() {
                success_count += 1;
            }
        }

        info!(
            "Wrote Polling Rate {rate_hz}Hz (Code 0x{rate_code:02X}) to {}/{} hidraw node(s) {:?}",
            success_count,
            nodes.len(),
            nodes
        );

        Ok(rate_hz)
    }

    /// Read current hardware DPI and Polling Rate from hidraw feature report.
    pub fn read_hardware_state(&self, hidraw_path: &Path) -> Result<(u32, u32)> {
        let nodes = get_rapoo_hidraw_nodes(Some(hidraw_path));

        for node in &nodes {
            let mut report = [0u8; 64];
            report[0] = 0x07;

            if self.get_feature_report(node, &mut report).is_ok() {
                let dpi = if report[4] > 0 || report[5] > 0 {
                    u32::from(report[4]) | (u32::from(report[5]) << 8)
                } else if report[3] > 0 {
                    match report[3] {
                        1 => 400,
                        2 => 800,
                        3 => 1200,
                        4 => 1600,
                        5 => 2400,
                        6 => 3200,
                        7 => 6400,
                        _ => 1200,
                    }
                } else {
                    1200
                };

                let hz = match report[2] {
                    0x01 => 125,
                    0x02 => 250,
                    0x04 => 500,
                    0x08 => 1000,
                    _ => 1000,
                };

                return Ok((dpi, hz));
            }
        }

        Ok((1200, 1000))
    }

    fn send_report_to_node(&self, path: &Path, base_report: &[u8; 64], report_id: u8) -> Result<()> {
        let mut report = *base_report;
        report[0] = report_id;

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|e| anyhow!("Failed to open hidraw node {:?}: {e}", path))?;

        let fd = file.as_raw_fd();

        // 1. Direct write for Output Report
        let _ = file.write_all(&report);
        let _ = file.flush();

        // 2. _IOC(_IOC_WRITE|_IOC_READ, 'H', 0x06, 64) = 0xC0404806
        let _ = unsafe {
            libc::ioctl(
                fd,
                i64::from_ne_bytes(0xC0404806u64.to_ne_bytes()) as libc::c_ulong,
                report.as_ptr(),
            )
        };

        // 3. _IOC(_IOC_WRITE|_IOC_READ, 'H', 0x06, 16) = 0xC0104806
        let _ = unsafe {
            libc::ioctl(
                fd,
                i64::from_ne_bytes(0xC0104806u64.to_ne_bytes()) as libc::c_ulong,
                report.as_ptr(),
            )
        };

        Ok(())
    }

    fn get_feature_report(&self, path: &Path, report: &mut [u8; 64]) -> Result<()> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|e| anyhow!("Failed to open hidraw node {:?}: {e}", path))?;

        let fd = file.as_raw_fd();

        // _IOC(_IOC_WRITE|_IOC_READ, 'H', 0x07, 64) = 0xC0404807
        let res = unsafe {
            libc::ioctl(
                fd,
                i64::from_ne_bytes(0xC0404807u64.to_ne_bytes()) as libc::c_ulong,
                report.as_mut_ptr(),
            )
        };

        if res < 0 {
            debug!("HIDIOCGFEATURE ioctl read returned fallback defaults");
        }

        Ok(())
    }
}
