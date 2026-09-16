//! HID descriptor and feature report reader for the `hid-report` subcommand.
//!
//! SAFETY: This module is strictly READ-ONLY. It never sends any command
//! or output report to the device. It only reads:
//!   1. The HID Report Descriptor from sysfs
//!   2. Feature reports (GET_REPORT only, no SET_REPORT)
//!
//! No HID write operations are ever performed by this module.

use crate::OutputFormat;
use anyhow::{bail, Context, Result};
use openrapoo_core::device::detect_rapoo_devices;
use std::path::PathBuf;
use tracing::{debug, warn};

/// HID report descriptor analysis result.
#[derive(Debug, serde::Serialize)]
pub struct HidReportInfo {
    pub hidraw_path: String,
    pub descriptor_bytes: usize,
    pub descriptor_hex: String,
    pub collections: Vec<HidCollection>,
    pub input_reports: Vec<HidReportEntry>,
    pub output_reports: Vec<HidReportEntry>,
    pub feature_reports: Vec<HidReportEntry>,
}

#[derive(Debug, serde::Serialize)]
pub struct HidCollection {
    pub usage_page: u16,
    pub usage: u16,
    pub description: String,
}

#[derive(Debug, serde::Serialize)]
pub struct HidReportEntry {
    pub report_id: u8,
    pub size_bits: u32,
}

/// Run the `hid-report` subcommand.
pub fn run_hid_report(device_path: Option<PathBuf>, show_hex: bool, format: &OutputFormat) -> Result<()> {
    let hidraw_path = resolve_hidraw_path(device_path)?;

    println!();
    println!("  HID Report Descriptor — Modo leitura apenas (read-only)");
    println!("  Dispositivo: {}", hidraw_path.display());
    println!();

    // Read HID descriptor from sysfs (safer than hidraw ioctl)
    let descriptor = read_hid_descriptor_sysfs(&hidraw_path)?;

    if descriptor.is_empty() {
        println!("  ✗ Descritor HID vazio ou não acessível.");
        println!("    Tente executar com sudo ou verifique as regras udev.");
        return Ok(());
    }

    println!("  Tamanho do descritor: {} bytes", descriptor.len());
    println!();

    if show_hex || matches!(format, OutputFormat::Human) {
        println!("  Dump hex do HID Report Descriptor:");
        print_hex_dump(&descriptor, 2);
        println!();
    }

    // Parse the descriptor
    let info = parse_hid_descriptor(&hidraw_path, &descriptor);

    match format {
        OutputFormat::Human => print_human_hid_info(&info),
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
    }

    // Attempt to read feature report 0 (READ-ONLY)
    println!("  Tentando ler feature report (somente leitura)...");
    match read_feature_report_zero(&hidraw_path) {
        Ok(data) if !data.is_empty() => {
            println!("  Feature report 0x00 ({} bytes):", data.len());
            print_hex_dump(&data, 4);
        }
        Ok(_) => {
            println!("  Feature report retornou dados vazios.");
        }
        Err(e) => {
            println!("  ✗ Não foi possível ler feature report: {e}");
            println!("    (Normal se o dispositivo não suportar ou permissão insuficiente)");
        }
    }

    println!();
    println!("  ℹ  AVISO: Nenhum dado foi enviado ao dispositivo.");
    println!("     Todas as operações acima são estritamente de leitura.");
    println!();

    Ok(())
}

/// Read the HID report descriptor from sysfs (preferred — no ioctl needed).
fn read_hid_descriptor_sysfs(hidraw_path: &PathBuf) -> Result<Vec<u8>> {
    // Extract hidraw number from path (e.g. /dev/hidraw2 → 2)
    let name = hidraw_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    // Try multiple sysfs paths
    let sysfs_paths = [
        format!("/sys/class/hidraw/{name}/device/report_descriptor"),
        format!("/sys/bus/hid/drivers/hid-generic/*/hidraw/{name}/device/report_descriptor"),
    ];

    for path_str in &sysfs_paths {
        // Handle glob patterns simply (just try without glob first)
        let path = std::path::Path::new(path_str);
        if path.exists() {
            debug!("Reading HID descriptor from {path_str}");
            return std::fs::read(path).context("Erro ao ler descritor HID do sysfs");
        }
    }

    // Fallback: try to enumerate via /sys/class/hidraw/
    let class_path = format!("/sys/class/hidraw/{name}");
    if let Ok(_target) = std::fs::read_link(&class_path) {
        let device_path = std::path::Path::new("/sys/class/hidraw")
            .join(name)
            .join("device/report_descriptor");
        if device_path.exists() {
            return std::fs::read(&device_path).context("Erro ao ler descritor HID");
        }
    }

    warn!("Não foi possível encontrar o descritor HID em sysfs para {}", hidraw_path.display());
    Ok(Vec::new())
}

/// Read feature report 0 using a GET_REPORT ioctl via the hidraw device.
/// This is read-only — no SET_REPORT is ever called.
fn read_feature_report_zero(hidraw_path: &PathBuf) -> Result<Vec<u8>> {
    use std::fs::OpenOptions;
    use std::os::unix::fs::OpenOptionsExt;

    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_RDONLY | libc::O_NONBLOCK)
        .open(hidraw_path)
        .with_context(|| format!("Não foi possível abrir {}", hidraw_path.display()))?;

    // HIDIOCGFEATURE ioctl for report ID 0
    // Buffer: first byte is report ID, rest is data
    let mut buf = vec![0u8; 64];
    buf[0] = 0; // report ID

    // HIDIOCGFEATURE(len) = _IOWR('H', 0x07, int)
    const HIDIOCGFEATURE: u64 = 0xC0404807; // for 64-byte buffer on x86_64

    let ret = unsafe {
        libc::ioctl(
            std::os::unix::io::AsRawFd::as_raw_fd(&file),
            HIDIOCGFEATURE,
            buf.as_mut_ptr(),
        )
    };

    if ret < 0 {
        let err = std::io::Error::last_os_error();
        return Err(err.into());
    }

    buf.truncate(ret as usize);
    Ok(buf)
}

/// Parse the HID report descriptor into a structured format.
/// This is a simplified parser that identifies collections and report types.
fn parse_hid_descriptor(hidraw_path: &PathBuf, descriptor: &[u8]) -> HidReportInfo {
    let mut collections = Vec::new();
    let mut input_reports = Vec::new();
    let mut output_reports = Vec::new();
    let mut feature_reports = Vec::new();

    // Simple HID descriptor parser
    // Full parsing is complex; this identifies key items for diagnostic purposes
    let mut i = 0;
    let mut current_usage_page: u16 = 0;
    let mut current_usage: u16 = 0;
    let mut current_report_id: u8 = 0;
    let mut current_report_size: u32 = 0;
    let mut current_report_count: u32 = 0;

    while i < descriptor.len() {
        let item = descriptor[i];
        let item_type = (item >> 2) & 0x03;
        let item_tag = (item >> 4) & 0x0F;
        let item_size = match item & 0x03 {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 4,
            _ => 0,
        };

        if i + 1 + item_size > descriptor.len() {
            break;
        }

        let data = &descriptor[i + 1..i + 1 + item_size];
        let value = match item_size {
            0 => 0u32,
            1 => data[0] as u32,
            2 => u16::from_le_bytes([data[0], data[1]]) as u32,
            4 => u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            _ => 0,
        };

        match (item_type, item_tag) {
            // Global items
            (1, 0) => current_usage_page = value as u16,  // Usage Page
            (1, 7) => current_report_size = value,         // Report Size
            (1, 8) => current_report_id = value as u8,     // Report ID
            (1, 9) => current_report_count = value,         // Report Count
            // Local items
            (2, 0) => current_usage = value as u16,         // Usage
            // Main items
            (0, 10) => {
                // Collection
                let desc = match current_usage_page {
                    0x01 => match current_usage {
                        0x02 => "Mouse",
                        0x06 => "Keyboard",
                        _ => "Generic Desktop",
                    },
                    0xFF00..=0xFFFF => "Vendor Defined",
                    _ => "Unknown",
                };
                collections.push(HidCollection {
                    usage_page: current_usage_page,
                    usage: current_usage,
                    description: desc.to_string(),
                });
            }
            (0, 8) => {
                // Input
                let bits = current_report_size * current_report_count;
                input_reports.push(HidReportEntry {
                    report_id: current_report_id,
                    size_bits: bits,
                });
            }
            (0, 9) => {
                // Output
                let bits = current_report_size * current_report_count;
                output_reports.push(HidReportEntry {
                    report_id: current_report_id,
                    size_bits: bits,
                });
            }
            (0, 11) => {
                // Feature
                let bits = current_report_size * current_report_count;
                feature_reports.push(HidReportEntry {
                    report_id: current_report_id,
                    size_bits: bits,
                });
            }
            _ => {}
        }

        i += 1 + item_size;
    }

    let hex = descriptor
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ");

    HidReportInfo {
        hidraw_path: hidraw_path.display().to_string(),
        descriptor_bytes: descriptor.len(),
        descriptor_hex: hex,
        collections,
        input_reports,
        output_reports,
        feature_reports,
    }
}

fn print_human_hid_info(info: &HidReportInfo) {
    println!("  Coleções HID detectadas: {}", info.collections.len());
    for col in &info.collections {
        println!(
            "    Usage Page: 0x{:04X}  Usage: 0x{:04X}  → {}",
            col.usage_page, col.usage, col.description
        );
    }
    println!();

    println!("  Input reports: {}", info.input_reports.len());
    for r in &info.input_reports {
        println!("    ID=0x{:02X}  {} bits ({} bytes)", r.report_id, r.size_bits, r.size_bits / 8);
    }

    if !info.output_reports.is_empty() {
        println!("  Output reports: {}", info.output_reports.len());
        for r in &info.output_reports {
            println!("    ID=0x{:02X}  {} bits", r.report_id, r.size_bits);
        }
    }

    if !info.feature_reports.is_empty() {
        println!("  Feature reports: {}", info.feature_reports.len());
        for r in &info.feature_reports {
            println!("    ID=0x{:02X}  {} bits", r.report_id, r.size_bits);
        }
    }

    if info.collections.iter().any(|c| c.usage_page >= 0xFF00) {
        println!();
        println!("  ⚠  Coleção 'Vendor Defined' detectada (usage_page >= 0xFF00).");
        println!("     Isso indica que o dispositivo possui relatórios HID proprietários.");
        println!("     Esses relatórios controlam funcionalidades como DPI, LED, etc.");
        println!("     O protocolo exato precisa de engenharia reversa (Fase 8).");
    }
}

fn print_hex_dump(data: &[u8], indent: usize) {
    let prefix = " ".repeat(indent);
    for (i, chunk) in data.chunks(16).enumerate() {
        let offset = i * 16;
        let hex: String = chunk.iter().map(|b| format!("{b:02X} ")).collect();
        let ascii: String = chunk
            .iter()
            .map(|&b| if b.is_ascii_graphic() { b as char } else { '.' })
            .collect();
        println!("{prefix}{offset:04X}  {hex:<48}  |{ascii}|");
    }
}

fn resolve_hidraw_path(given: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = given {
        return Ok(p);
    }
    let devices = detect_rapoo_devices()?;
    for device in &devices {
        if let Some(path) = &device.hidraw_path {
            return Ok(path.clone());
        }
    }
    bail!(
        "Nenhum nó hidraw encontrado para dispositivos Rapoo.\n\
        Especifique com --device /dev/hidrawX\n\
        Verifique as regras udev e permissões do grupo 'input'."
    );
}
