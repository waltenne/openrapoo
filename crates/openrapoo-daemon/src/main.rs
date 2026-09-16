//! `openrapoo-daemon` — Background remapping service for Rapoo MT760 Pro.

mod remapper;
mod virtual_device;

use anyhow::Result;
use clap::Parser;
use remapper::{Remapper, RemapperConfig};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "openrapoo-daemon",
    about = "OpenRapoo Daemon — Background event remapping service",
    version,
    author
)]
struct Cli {
    /// Path to evdev device node (e.g. /dev/input/event22). If omitted, auto-detects Rapoo devices.
    #[arg(short, long)]
    device: Option<PathBuf>,

    /// Path to configuration file (defaults to ~/.config/openrapoo/profiles.json)
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Run in dry-run mode (logs actions without capturing device or creating uinput virtual node)
    #[arg(long)]
    dry_run: bool,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let level = match cli.verbose {
        0 => Level::INFO,
        1 => Level::DEBUG,
        _ => Level::TRACE,
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(level.into())
                .from_env_lossy(),
        )
        .with_target(false)
        .init();

    info!("openrapoo-daemon v{}", env!("CARGO_PKG_VERSION"));

    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let shutdown_flag = shutdown_signal.clone();

    // Signal handler for SIGINT (Ctrl+C) and SIGTERM
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        info!("Received shutdown signal. Stopping daemon...");
        shutdown_flag.store(true, Ordering::Relaxed);
    });

    let config = RemapperConfig {
        device_path: cli.device,
        config_path: cli.config,
        dry_run: cli.dry_run,
    };

    let mut remapper = Remapper::new(config, shutdown_signal)?;
    remapper.run().await?;

    info!("Daemon shut down cleanly.");
    Ok(())
}
