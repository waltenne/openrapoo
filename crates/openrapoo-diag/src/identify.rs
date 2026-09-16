//! Interactive button identification for the `identify-buttons` subcommand.
//!
//! Waits for button press and scroll events and displays what was detected,
//! building a map of input → event code as the user interacts with the mouse.
//! Does NOT use exclusive grab — other apps continue to receive events.
//!
//! Captures both:
//!   - EV_KEY events: button clicks (BTN_LEFT, BTN_SIDE, etc.)
//!   - EV_REL events: scroll wheels (REL_WHEEL, REL_HWHEEL, etc.)

use anyhow::{bail, Context, Result};
use evdev::{Device, EventType};
use openrapoo_core::{
    device::detect_rapoo_devices,
    event::ButtonCode,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

/// Run the `identify-buttons` subcommand.
pub async fn run_identify_buttons(device_path: Option<PathBuf>) -> Result<()> {
    let path = resolve_evdev_path(device_path)?;

    println!();
    println!("  ╔══════════════════════════════════════════════════════════╗");
    println!("  ║         OpenRapoo — Identificar Botões e Scroll         ║");
    println!("  ╠══════════════════════════════════════════════════════════╣");
    println!("  ║  Pressione cada botão do mouse um por vez.              ║");
    println!("  ║  Role a roda lateral para detectar scroll horizontal.   ║");
    println!("  ║  O código do evento será exibido.                       ║");
    println!("  ║  Pressione Ctrl+C para encerrar.                        ║");
    println!("  ╚══════════════════════════════════════════════════════════╝");
    println!();

    let mut device = Device::open(&path)
        .with_context(|| format!("Não foi possível abrir {}", path.display()))?;

    let device_name = device.name().unwrap_or("(sem nome)").to_string();
    println!("  Dispositivo: {device_name}");
    println!("  Nó evdev:    {}", path.display());
    println!();
    println!("  {:<35}  {:<12}  {:<12}  {}", "Entrada identificada", "Tipo", "Código hex", "Valor");
    println!("  {:<35}  {:<12}  {:<12}  {}", "─".repeat(35), "─".repeat(12), "─".repeat(12), "─".repeat(6));

    // Track seen events: key = "TYPE:CODE", value = description
    let mut identified: HashMap<String, String> = HashMap::new();

    loop {
        let events = tokio::task::block_in_place(|| {
            device.fetch_events().map(|e| e.collect::<Vec<_>>())
        });

        let events = match events {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => break,
            Err(e) => return Err(e).context("Erro ao ler eventos"),
        };

        for event in events {
            match event.event_type() {
                EventType::KEY => {
                    // Only show press (value=1), not repeat (value=2) or release (value=0)
                    if event.value() != 1 {
                        continue;
                    }

                    let code = event.code();
                    let button = ButtonCode::from_raw(code);
                    let name = match &button {
                        ButtonCode::Other(_) => format!("Botão desconhecido (0x{code:04X})"),
                        b => b.name().to_string(),
                    };

                    let map_key = format!("KEY:{code}");
                    let is_new = !identified.contains_key(&map_key);
                    identified.insert(map_key, name.clone());

                    let new_marker = if is_new { " ← NOVO" } else { "" };
                    println!(
                        "  {:<35}  {:<12}  0x{:<12X}  1 (pressionado){new_marker}",
                        name, "EV_KEY/BTN", code
                    );
                }

                EventType::RELATIVE => {
                    let code = event.code();
                    let value = event.value();

                    // Only show non-zero scroll values (ignore movement noise)
                    // For REL_X/REL_Y (codes 0 and 1), skip unless interesting
                    if code <= 1 {
                        continue; // skip mouse X/Y movement
                    }

                    let axis_name = rel_axis_name(code);
                    let map_key = format!("REL:{code}");
                    let is_new = !identified.contains_key(&map_key);
                    // For scroll, insert with direction
                    let dir = if value > 0 { "↓/→" } else { "↑/←" };
                    identified.entry(map_key).or_insert_with(|| axis_name.to_string());

                    let new_marker = if is_new { " ← NOVO" } else { "" };
                    println!(
                        "  {:<35}  {:<12}  0x{:<12X}  {value} ({dir}){new_marker}",
                        axis_name, "EV_REL", code
                    );
                }

                _ => {}
            }
        }
    }

    println!();
    println!("  ══════════════════════════════════════════════════════════");
    println!("  Resumo: {} entrada(s) identificada(s)", identified.len());
    println!("  ══════════════════════════════════════════════════════════");
    println!();

    // Separate buttons from axes for summary
    let buttons: Vec<_> = identified.iter()
        .filter(|(k, _)| k.starts_with("KEY:"))
        .collect();
    let axes: Vec<_> = identified.iter()
        .filter(|(k, _)| k.starts_with("REL:"))
        .collect();

    if !buttons.is_empty() {
        println!("  Botões ({}):", buttons.len());
        for (key, name) in &buttons {
            let code: u16 = key.strip_prefix("KEY:").unwrap_or("0").parse().unwrap_or(0);
            let remappable = !matches!(ButtonCode::from_raw(code), ButtonCode::Other(c) if c == 0);
            println!("    0x{code:04X}  →  {name}  {}", if remappable { "(remapeável via software)" } else { "" });
        }
        println!();
    }

    if !axes.is_empty() {
        println!("  Eixos de scroll ({}):", axes.len());
        for (key, name) in &axes {
            let code: u16 = key.strip_prefix("REL:").unwrap_or("0").parse().unwrap_or(0);
            println!("    0x{code:04X}  →  {name}  (remapeável via software)");
        }
        println!();
    }

    // Warn about unknown button codes
    let unknown_btns: Vec<_> = buttons.iter()
        .filter(|(key, _)| {
            let code: u16 = key.strip_prefix("KEY:").unwrap_or("0").parse().unwrap_or(0);
            matches!(ButtonCode::from_raw(code), ButtonCode::Other(_))
        })
        .collect();

    if !unknown_btns.is_empty() {
        println!("  ℹ  Botões com código 'Desconhecido' geram eventos mas não têm");
        println!("     nome no evdev padrão. Eles PODEM ser remapeados por software.");
        println!("     Reporte esses códigos no GitHub para que possamos nomeá-los.");
        println!();
    }

    Ok(())
}

/// Return a human-readable name for a REL axis code.
fn rel_axis_name(code: u16) -> &'static str {
    match code {
        0 => "Movimento X",
        1 => "Movimento Y",
        6 => "Scroll Horizontal (REL_HWHEEL)",
        8 => "Scroll Vertical (REL_WHEEL)",
        11 => "Scroll Horizontal Discreto (REL_HWHEEL_HI_RES)",
        12 => "Scroll Vertical Discreto (REL_WHEEL_HI_RES)",
        _ => "Eixo relativo desconhecido",
    }
}

fn resolve_evdev_path(given: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = given {
        return Ok(p);
    }
    let devices = detect_rapoo_devices()?;
    for device in &devices {
        if let Some(path) = &device.evdev_path {
            info!("Auto-detectado: {} → {}", device.model, path.display());
            return Ok(path.clone());
        }
    }
    if devices.is_empty() {
        bail!("Nenhum dispositivo Rapoo encontrado. Conecte o mouse e tente novamente.");
    } else {
        bail!("Dispositivo Rapoo sem nó evdev acessível. Verifique permissões.");
    }
}
