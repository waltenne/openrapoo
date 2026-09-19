//! Device detection and listing for the `list-devices` subcommand.

use crate::OutputFormat;
use anyhow::Result;
use openrapoo_core::device::{detect_rapoo_devices, RapooDevice};

/// Run the `list-devices` subcommand.
pub fn run_list_devices(format: &OutputFormat) -> Result<()> {
    let devices = detect_rapoo_devices()?;

    match format {
        OutputFormat::Human => print_human(&devices),
        OutputFormat::Json => print_json(&devices)?,
    }

    Ok(())
}

fn print_human(devices: &[RapooDevice]) {
    if devices.is_empty() {
        println!();
        println!("  ✗ Nenhum dispositivo Rapoo detectado (VID 0x24AE).");
        println!();
        println!("  Possíveis causas:");
        println!("    • O mouse não está conectado");
        println!("    • O mouse está em modo Bluetooth (tente via receptor 2.4 GHz)");
        println!("    • O PID do MT760 Pro ainda não está na lista de PIDs conhecidos");
        println!("      → Execute `lsusb | grep 24ae` e reporte o PID no GitHub");
        println!();
        println!("  Dica: Para debug extra, execute com -v:");
        println!("    openrapoo-diag -v list-devices");
        println!();
        return;
    }

    println!();
    println!("  Dispositivos Rapoo detectados: {}", devices.len());
    println!("  ─────────────────────────────────────────────────");

    for (i, device) in devices.iter().enumerate() {
        let num = i + 1;
        let mt760_marker = if device.is_mt760_pro() {
            " ← alvo"
        } else {
            ""
        };

        println!();
        println!("  [{num}] {}{mt760_marker}", device.model);
        println!("      Nome:       {}", device.name);
        println!(
            "      VID:PID:    0x{:04X}:0x{:04X}",
            device.vendor_id, device.product_id
        );
        println!("      Conexão:    {}", device.connection);

        match &device.evdev_path {
            Some(p) => {
                let accessible = if device.evdev_accessible() {
                    "✓"
                } else {
                    "✗ (sem permissão)"
                };
                println!("      evdev:      {}  {}", p.display(), accessible);
            }
            None => println!("      evdev:      (não encontrado)"),
        }

        match &device.hidraw_path {
            Some(p) => {
                let accessible = if device.hidraw_accessible() {
                    "✓"
                } else {
                    "✗ (sem permissão)"
                };
                println!("      hidraw:     {}  {}", p.display(), accessible);
            }
            None => println!("      hidraw:     (não encontrado)"),
        }

        if let Some(phys) = &device.phys {
            println!("      Localização: {phys}");
        }
    }

    println!();

    // Check if any device has inaccessible nodes
    let has_permission_issues = devices.iter().any(|d| {
        (d.evdev_path.is_some() && !d.evdev_accessible())
            || (d.hidraw_path.is_some() && !d.hidraw_accessible())
    });

    if has_permission_issues {
        println!("  ⚠ Problema de permissão detectado. Para corrigir:");
        println!("    sudo cp udev/99-openrapoo.rules /etc/udev/rules.d/");
        println!("    sudo udevadm control --reload-rules");
        println!("    sudo usermod -aG input $USER");
        println!("    → Faça logout e login novamente");
        println!();
    }
}

fn print_json(devices: &[RapooDevice]) -> Result<()> {
    let json = serde_json::to_string_pretty(devices)?;
    println!("{json}");
    Ok(())
}
