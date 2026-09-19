//! Dynamic hidraw device interface scanner and HID report descriptor parser.

use std::fs;
use std::path::{Path, PathBuf};

/// Detailed candidate information for a detected hidraw interface node.
#[derive(Debug, Clone, Default)]
pub struct HidrawCandidate {
    pub hidraw_path: PathBuf,
    pub vendor_id: u16,
    pub product_id: u16,
    pub name: String,
    pub hid_uniq: String,
    pub parent_usb_path: String,
    pub is_vendor_interface: bool,
    pub vendor_usage_pages: Vec<u16>,
    pub report_ids: Vec<u8>,
}

/// Scan raw HID report descriptor bytes and extract all declared Usage Pages and Report IDs.
pub fn parse_report_descriptor(descriptor: &[u8]) -> (Vec<u16>, Vec<u8>) {
    let mut usage_pages = Vec::new();
    let mut report_ids = Vec::new();
    let mut i = 0;

    while i < descriptor.len() {
        let header = descriptor[i];
        i += 1;

        if header == 0xFE {
            // Long item prefix, skip length + data
            if i + 1 < descriptor.len() {
                let data_size = descriptor[i] as usize;
                i += 2 + data_size;
            }
            continue;
        }

        let size_code = header & 0x03;
        let item_type = (header >> 2) & 0x03;
        let item_tag = (header >> 4) & 0x0F;

        let data_len = match size_code {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 4,
            _ => 0,
        };

        if i + data_len > descriptor.len() {
            break;
        }

        let data_bytes = &descriptor[i..i + data_len];
        i += data_len;

        // Global item: Usage Page (bType == 1, bTag == 0)
        if item_type == 1 && item_tag == 0 && !data_bytes.is_empty() {
            let page: u16 = match data_len {
                1 => data_bytes[0] as u16,
                2 => u16::from_le_bytes([data_bytes[0], data_bytes[1]]),
                4 => u16::from_le_bytes([data_bytes[0], data_bytes[1]]),
                _ => 0,
            };
            if !usage_pages.contains(&page) {
                usage_pages.push(page);
            }
        }

        // Global item: Report ID (bType == 1, bTag == 8)
        if item_type == 1 && item_tag == 8 && !data_bytes.is_empty() {
            let id = data_bytes[0];
            if !report_ids.contains(&id) {
                report_ids.push(id);
            }
        }
    }

    (usage_pages, report_ids)
}

/// Scan all `/sys/class/hidraw/hidraw*` devices dynamically without hardcoding paths.
pub fn scan_hidraw_interfaces() -> Vec<HidrawCandidate> {
    let mut candidates = Vec::new();
    let sysfs_hidraw_base = Path::new("/sys/class/hidraw");

    if !sysfs_hidraw_base.exists() {
        return candidates;
    }

    let entries = match fs::read_dir(sysfs_hidraw_base) {
        Ok(e) => e,
        Err(_) => return candidates,
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with("hidraw") {
            continue;
        }

        let dev_dir = entry.path().join("device");
        let uevent_path = dev_dir.join("uevent");
        let mut vid = 0u16;
        let mut pid = 0u16;
        let mut name_val = String::new();
        let mut uniq_val = String::new();

        if let Ok(uevent_content) = fs::read_to_string(&uevent_path) {
            for line in uevent_content.lines() {
                if let Some(rest) = line.strip_prefix("HID_ID=") {
                    // Format: BUS:VENDOR:PRODUCT (e.g. 0003:000024AE:0000186A)
                    let parts: Vec<&str> = rest.split(':').collect();
                    if parts.len() >= 3 {
                        vid = u16::from_str_radix(parts[1].trim_start_matches('0'), 16).unwrap_or(0);
                        pid = u16::from_str_radix(parts[2].trim_start_matches('0'), 16).unwrap_or(0);
                    }
                } else if let Some(rest) = line.strip_prefix("HID_NAME=") {
                    name_val = rest.trim_matches('"').to_string();
                } else if let Some(rest) = line.strip_prefix("HID_UNIQ=") {
                    uniq_val = rest.trim_matches('"').to_string();
                }
            }
        }

        let parent_path = dev_dir
            .canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let mut vendor_pages = Vec::new();
        let mut report_ids = Vec::new();
        let descriptor_path = dev_dir.join("report_descriptor");

        if let Ok(desc) = fs::read(&descriptor_path) {
            let (pages, ids) = parse_report_descriptor(&desc);
            report_ids = ids;
            vendor_pages = pages
                .into_iter()
                .filter(|&p| (0xFF00..=0xFFFF).contains(&p))
                .collect();
        }

        let is_vendor_interface = !vendor_pages.is_empty()
            || report_ids.contains(&0x07)
            || report_ids.contains(&0xA0);

        let node_num = name_str.strip_prefix("hidraw").unwrap_or("0");
        let dev_path = PathBuf::from(format!("/dev/hidraw{node_num}"));

        candidates.push(HidrawCandidate {
            hidraw_path: dev_path,
            vendor_id: vid,
            product_id: pid,
            name: name_val,
            hid_uniq: uniq_val,
            parent_usb_path: parent_path,
            is_vendor_interface,
            vendor_usage_pages: vendor_pages,
            report_ids,
        });
    }

    candidates
}

/// Locate vendor hidraw interface candidate matching a specific VID/PID or HID_UNIQ.
pub fn find_vendor_hidraw_candidate(vid: u16, pid: u16, uniq: Option<&str>) -> Option<HidrawCandidate> {
    let candidates = scan_hidraw_interfaces();

    // 1. First priority: match by VID, PID, HID_UNIQ, AND vendor interface flag
    if let Some(u) = uniq {
        if !u.is_empty() {
            let clean_u = u.replace([':', '-'], "").to_lowercase();
            if let Some(c) = candidates.iter().find(|c| {
                c.vendor_id == vid
                    && c.is_vendor_interface
                    && !c.hid_uniq.is_empty()
                    && c.hid_uniq.replace([':', '-'], "").to_lowercase() == clean_u
            }) {
                return Some(c.clone());
            }
        }
    }

    // 2. Second priority: match by VID, PID, AND vendor interface flag
    if let Some(c) = candidates.iter().find(|c| c.vendor_id == vid && (c.product_id == pid || pid == 0) && c.is_vendor_interface) {
        return Some(c.clone());
    }

    // 3. Fallback: match by VID and vendor interface
    candidates.into_iter().find(|c| c.vendor_id == vid && c.is_vendor_interface)
}

