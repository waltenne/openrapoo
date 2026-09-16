//! Real-time event capture for the `capture-events` subcommand.
//!
//! This module reads events from an evdev device node and displays them.
//! It NEVER grabs exclusive access (no EVIOCGRAB) — other applications
//! continue to receive events normally.

use crate::OutputFormat;
use anyhow::{bail, Context, Result};
use evdev::{Device, EventType};
use openrapoo_core::{
    device::detect_rapoo_devices,
    event::{Axis, ButtonCode, ButtonEvent, MouseEvent, MouseEventKind, ScrollAxis},
};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use tracing::info;

/// Run the `capture-events` subcommand.
pub async fn run_capture_events(
    device_path: Option<PathBuf>,
    limit: u64,
    raw: bool,
    format: &OutputFormat,
) -> Result<()> {
    let path = resolve_evdev_path(device_path)?;

    println!();
    println!("  Capturando eventos de: {}", path.display());
    println!("  Mova o mouse ou pressione botões.");
    println!("  Pressione Ctrl+C para parar.");
    if limit > 0 {
        println!("  Limite: {limit} eventos");
    }
    println!("  ─────────────────────────────────────────────");
    println!();

    let mut device = Device::open(&path)
        .with_context(|| format!("Não foi possível abrir {}: verifique permissões (grupo 'input')", path.display()))?;

    let device_name = device.name().unwrap_or("(sem nome)").to_string();
    println!("  Dispositivo: {device_name}");
    println!();

    let mut count = 0u64;

    // Print column headers
    match format {
        OutputFormat::Human => {
            if raw {
                println!("  {:>16}  {:>8}  {:>8}  {:>8}  {}", "timestamp_µs", "type", "code", "value", "description");
                println!("  {:->16}  {:->8}  {:->8}  {:->8}  {}", "", "", "", "", "─────────────");
            } else {
                println!("  {:>16}  {:>40}  {}", "timestamp_µs", "evento", "detalhe");
                println!("  {:->16}  {:->40}  {}", "", "", "─────────────");
            }
        }
        OutputFormat::Json => {
            println!("[");
        }
    }

    let mut first_json = true;

    loop {
        // tokio::select! to allow graceful Ctrl+C handling
        let events = tokio::task::block_in_place(|| {
            device.fetch_events().map(|e| e.collect::<Vec<_>>())
        });

        let events = match events {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => break,
            Err(e) => return Err(e).context("Erro ao ler eventos"),
        };

        for event in events {
            let ts = event.timestamp();
            let ts_us = ts
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_micros() as u64)
                .unwrap_or(0);
            let ev_type = event.event_type();
            let code = event.code();
            let value = event.value();

            let mouse_event = decode_event(ts_us, ev_type, code, value);

            // Skip sync events in human mode for brevity
            if matches!(format, OutputFormat::Human)
                && matches!(mouse_event.kind, MouseEventKind::Sync)
                && !raw
            {
                continue;
            }

            match format {
                OutputFormat::Human => {
                    if raw {
                        println!(
                            "  {:>16}  {:>8}  {:>8}  {:>8}  {}",
                            ts_us,
                            ev_type.0,
                            code,
                            value,
                            event_type_name(ev_type)
                        );
                    } else {
                        let (name, detail) = describe_event(&mouse_event);
                        println!("  {:>16}  {:>40}  {detail}", ts_us, name);
                    }
                }
                OutputFormat::Json => {
                    if !first_json {
                        print!(",\n");
                    }
                    first_json = false;
                    print!("  {}", serde_json::to_string(&mouse_event)?);
                }
            }

            count += 1;
            if limit > 0 && count >= limit {
                println!();
                println!("  Limite de {limit} eventos atingido.");
                if matches!(format, OutputFormat::Json) {
                    println!("\n]");
                }
                return Ok(());
            }
        }
    }

    if matches!(format, OutputFormat::Json) {
        println!("\n]");
    }
    println!();
    println!("  Total de eventos capturados: {count}");

    Ok(())
}

/// Resolve the evdev device path, auto-detecting if not provided.
fn resolve_evdev_path(given: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(p) = given {
        return Ok(p);
    }

    // Auto-detect first Rapoo device
    let devices = detect_rapoo_devices()?;
    for device in &devices {
        if let Some(path) = &device.evdev_path {
            info!("Auto-detectado: {} → {}", device.model, path.display());
            return Ok(path.clone());
        }
    }

    if devices.is_empty() {
        bail!(
            "Nenhum dispositivo Rapoo encontrado.\n\
            Conecte o mouse e tente novamente, ou especifique o caminho com --device /dev/input/eventX"
        );
    } else {
        bail!(
            "Dispositivo Rapoo detectado mas sem nó evdev acessível.\n\
            Verifique permissões ou especifique --device /dev/input/eventX"
        );
    }
}

/// Decode a raw evdev event into a `MouseEvent`.
fn decode_event(ts_us: u64, ev_type: EventType, code: u16, value: i32) -> MouseEvent {
    let kind = match ev_type {
        EventType::KEY => {
            let button = ButtonCode::from_raw(code);
            MouseEventKind::Button(ButtonEvent {
                button,
                pressed: value != 0,
            })
        }
        EventType::RELATIVE => match code {
            0 => MouseEventKind::Movement { axis: Axis::X, delta: value },
            1 => MouseEventKind::Movement { axis: Axis::Y, delta: value },
            8 => MouseEventKind::Scroll { axis: ScrollAxis::Vertical, delta: value },
            6 => MouseEventKind::Scroll { axis: ScrollAxis::Horizontal, delta: value },
            _ => MouseEventKind::Unknown,
        },
        EventType::SYNCHRONIZATION => MouseEventKind::Sync,
        _ => MouseEventKind::Unknown,
    };

    MouseEvent {
        timestamp_us: ts_us,
        kind,
        raw_type: ev_type.0,
        raw_code: code,
        raw_value: value,
    }
}

/// Return a human-readable description of an event.
fn describe_event(event: &MouseEvent) -> (String, String) {
    match &event.kind {
        MouseEventKind::Button(btn) => {
            let action = if btn.pressed { "PRESS" } else { "release" };
            let name = format!("Button: {}", btn.button.name());
            let detail = format!("{action} (code=0x{:04X})", event.raw_code);
            (name, detail)
        }
        MouseEventKind::Movement { axis, delta } => {
            let name = format!("Move {:?}", axis);
            (name, format!("delta={delta}"))
        }
        MouseEventKind::Scroll { axis, delta } => {
            let dir = match axis {
                ScrollAxis::Vertical => if *delta > 0 { "↓" } else { "↑" },
                ScrollAxis::Horizontal => if *delta > 0 { "→" } else { "←" },
            };
            (format!("Scroll {:?} {dir}", axis), format!("delta={delta}"))
        }
        MouseEventKind::Sync => ("SYNC".to_string(), String::new()),
        MouseEventKind::Unknown => (
            "Unknown".to_string(),
            format!("type={} code={} value={}", event.raw_type, event.raw_code, event.raw_value),
        ),
    }
}

/// Return the name of an evdev event type.
fn event_type_name(ev_type: EventType) -> &'static str {
    match ev_type {
        EventType::SYNCHRONIZATION => "EV_SYN",
        EventType::KEY => "EV_KEY",
        EventType::RELATIVE => "EV_REL",
        EventType::ABSOLUTE => "EV_ABS",
        EventType::MISC => "EV_MSC",
        _ => "EV_???",
    }
}
