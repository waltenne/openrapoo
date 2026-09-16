//! Interactive button identification for the `identify-buttons` subcommand.
//!
//! Waits for button press events and displays which button was pressed,
//! building a map of button → event code as the user presses each button.
//! Does NOT use exclusive grab — other apps continue to receive events.

use anyhow::{bail, Context, Result};
use evdev::{Device, EventType};
use openrapoo_core::{
    device::detect_rapoo_devices,
    event::{ButtonCode},
};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

/// Run the `identify-buttons` subcommand.
pub async fn run_identify_buttons(device_path: Option<PathBuf>) -> Result<()> {
    let path = resolve_evdev_path(device_path)?;

    println!();
    println!("  ╔══════════════════════════════════════════════════╗");
    println!("  ║         OpenRapoo — Identificar Botões           ║");
    println!("  ╠══════════════════════════════════════════════════╣");
    println!("  ║  Pressione cada botão do mouse um por vez.       ║");
    println!("  ║  O código do evento será exibido.                ║");
    println!("  ║  Pressione Ctrl+C para encerrar.                 ║");
    println!("  ╚══════════════════════════════════════════════════╝");
    println!();

    let mut device = Device::open(&path)
        .with_context(|| format!("Não foi possível abrir {}", path.display()))?;

    println!("  Dispositivo: {}", device.name().unwrap_or("(sem nome)"));
    println!("  Nó evdev:    {}", path.display());
    println!();
    println!("  {:<30}  {:<12}  {:<8}  {}", "Botão identificado", "Tipo", "Código hex", "Valor");
    println!("  {:<30}  {:<12}  {:<8}  {}", "─".repeat(30), "─".repeat(12), "─".repeat(10), "─".repeat(6));

    let mut identified: HashMap<u16, String> = HashMap::new();

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
            if event.event_type() != EventType::KEY {
                continue;
            }
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

            let is_new = !identified.contains_key(&code);
            identified.insert(code, name.clone());

            let new_marker = if is_new { " ← NOVO" } else { "" };
            println!(
                "  {:<30}  {:<12}  0x{:<8X}  1 (pressionado){new_marker}",
                name, "EV_KEY/BTN", code
            );
        }
    }

    println!();
    println!("  ══════════════════════════════════════════════");
    println!("  Resumo: {} botão(ões) identificado(s)", identified.len());
    println!("  ══════════════════════════════════════════════");
    println!();

    for (code, name) in &identified {
        println!("  0x{code:04X}  →  {name}");
    }
    println!();

    if identified.iter().any(|(code, _)| matches!(ButtonCode::from_raw(*code), ButtonCode::Other(_))) {
        println!("  ℹ  Botões com código 'Desconhecido' geram eventos mas não têm");
        println!("     nome no evdev padrão. Eles PODEM ser remapeados por software.");
        println!("     Reporte esses códigos no GitHub para que possamos nomeá-los.");
        println!();
    }

    Ok(())
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
