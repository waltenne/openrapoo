//! OpenRapoo GUI & Main Application Entry Point.

mod app_window;
mod i18n;
mod views;

use app_window::AppWindowController;
use clap::Parser;
use i18n::tr;
use openrapoo_core::{
    device::detect_rapoo_devices,
    permissions::install_udev_rules,
};
use std::process::exit;
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

/// OpenRapoo — Linux configuration tool for Rapoo MT760 Pro mouse.
#[derive(Parser, Debug)]
#[command(
    name = "openrapoo-gui",
    about = "OpenRapoo — Configuration utility for Rapoo MT760 Pro mouse",
    version,
    author
)]
pub struct Cli {
    /// Install udev rules into /etc/udev/rules.d/99-openrapoo.rules (requires root)
    #[arg(long)]
    pub install_udev: bool,

    /// List connected Rapoo devices and exit
    #[arg(long)]
    pub list_devices: bool,

    /// Perform diagnostic check and output technical report
    #[arg(long)]
    pub diagnose: bool,
}

fn main() {
    let cli = Cli::parse();

    // 1. Handle --install-udev CLI command without initializing GTK
    if cli.install_udev {
        match install_udev_rules(None) {
            Ok(path) => {
                println!("✓ Regras udev instaladas com sucesso em: {}", path.display());
                println!("  Executado udevadm control --reload-rules && udevadm trigger.");
                exit(0);
            }
            Err(err) => {
                eprintln!("{err}");
                exit(1);
            }
        }
    }

    // 2. Handle --list-devices CLI command without initializing GTK
    if cli.list_devices {
        println!("Buscando dispositivos Rapoo conectados...");
        match detect_rapoo_devices() {
            Ok(devices) if devices.is_empty() => {
                println!("Nenhum dispositivo Rapoo encontrado.");
            }
            Ok(devices) => {
                println!("Encontrado(s) {} dispositivo(s) Rapoo:", devices.len());
                for (i, dev) in devices.iter().enumerate() {
                    println!(
                        "  [{}] {} (0x{:04X}:0x{:04X}) - Conexão: {}",
                        i + 1,
                        dev.name,
                        dev.vendor_id,
                        dev.product_id,
                        dev.connection
                    );
                    if let Some(ref ev) = dev.evdev_path {
                        println!("      evdev: {}", ev.display());
                    }
                }
            }
            Err(e) => {
                eprintln!("Erro ao detectar dispositivos: {e}");
                exit(1);
            }
        }
        exit(0);
    }

    // 3. Handle --diagnose CLI command without initializing GTK
    if cli.diagnose {
        println!("=== Diagnóstico OpenRapoo ===");
        println!("Versão: v{}", env!("CARGO_PKG_VERSION"));
        let status = openrapoo_core::permissions::check_input_group_status();
        println!("Grupo input: {}", status.display_message_pt());
        println!("Caminho de configuração: {}", openrapoo_core::config::ProfileStore::default_config_path().display());

        match detect_rapoo_devices() {
            Ok(devices) => println!("Dispositivos detectados: {}", devices.len()),
            Err(e) => println!("Erro na detecção: {e}"),
        }
        exit(0);
    }

    // 4. Default: Launch GUI application
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .with_target(false)
        .init();

    info!("OpenRapoo GUI v{}", env!("CARGO_PKG_VERSION"));
    info!("{}", tr("Inicializando interface gráfica GTK4 + libadwaita...", "Initializing GTK4 + libadwaita GUI..."));

    let controller = AppWindowController::default();

    println!();
    println!("  ╔══════════════════════════════════════════════════════════════╗");
    println!("  ║                   OpenRapoo GTK4 Interface                   ║");
    println!("  ╠══════════════════════════════════════════════════════════════╣");
    println!("  ║  Status: {}", controller.home_state.status_message);
    if let Some(ref dev) = controller.home_state.detected_device {
        println!("  ║  Dispositivo: {} (0x{:04X}:0x{:04X})", dev.name, dev.vendor_id, dev.product_id);
        println!("  ║  Conexão: {}", dev.connection);
    }
    println!("  ║  Perfis carregados: {}", controller.profiles_state.store.profiles.len());
    println!("  ║  Regras udev: {}", if controller.permissions_status.udev_rule_exists { "Instaladas ✓" } else { "Ausentes ✗" });
    println!("  ║  Grupo input: {}", controller.permissions_status.input_group_state.display_message_pt());
    println!("  ╚══════════════════════════════════════════════════════════════╝");
    println!();
}
