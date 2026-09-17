//! OpenRapoo GUI — Main application entry point (Phase 5).

mod app_window;
mod i18n;
mod views;

use app_window::AppWindowController;
use i18n::tr;
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

fn main() {
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
    println!("  ║  Grupo input: {}", if controller.permissions_status.input_group_member { "Membro ✓" } else { "Não membro ✗" });
    println!("  ╚══════════════════════════════════════════════════════════════╝");
    println!();
}
