//! `openrapoo-daemon` — Background remapping service for Rapoo MT760 Pro.

mod hardware_writer;
mod ipc_server;
mod remapper;
mod snapshot_manager;
mod virtual_device;

use anyhow::Result;
use clap::Parser;
use ipc_server::DaemonIpcServer;
use openrapoo_core::config::ProfileStore;
use openrapoo_core::ipc::{default_socket_path, DaemonIpcMessage, DaemonIpcResponse};
use remapper::{Remapper, RemapperConfig};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{info, warn, Level};
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

    // Check if another daemon instance is already running and active
    let socket_path = default_socket_path();
    if socket_path.exists() && check_existing_daemon(&socket_path) {
        info!(
            "Another active instance of openrapoo-daemon is responding on socket {}. Exiting cleanly.",
            socket_path.display()
        );
        return Ok(());
    }

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

    let config_file = cli
        .config
        .clone()
        .unwrap_or_else(ProfileStore::default_config_path);

    let profile_store = ProfileStore::load_from_file(&config_file).unwrap_or_else(|e| {
        warn!("Could not load config file: {e}. Using defaults.");
        ProfileStore::default()
    });

    let shared_store = Arc::new(RwLock::new(profile_store));

    let config = RemapperConfig {
        device_path: cli.device,
        config_path: cli.config,
        dry_run: cli.dry_run,
    };

    let mut remapper = Remapper::new(config, shared_store.clone(), shutdown_signal.clone())?;
    let ipc_server = DaemonIpcServer::new(shared_store, config_file, shutdown_signal.clone());

    tokio::select! {
        res = remapper.run() => {
            if let Err(e) = res {
                warn!("Remapper loop exited with error: {e}");
            }
        }
        res = ipc_server.run() => {
            if let Err(e) = res {
                warn!("IPC server loop exited with error: {e}");
            }
        }
    }

    info!("Daemon shut down cleanly.");
    Ok(())
}

/// Helper function to check if an active daemon responds on `socket_path`.
fn check_existing_daemon(socket_path: &PathBuf) -> bool {
    let Ok(mut stream) = UnixStream::connect(socket_path) else {
        return false;
    };

    let _ = stream.set_read_timeout(Some(Duration::from_millis(300)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(300)));

    let Ok(json) = serde_json::to_string(&DaemonIpcMessage::Ping) else {
        return false;
    };

    if writeln!(stream, "{json}").is_err() || stream.flush().is_err() {
        return false;
    }

    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
        return false;
    }

    matches!(
        serde_json::from_str::<DaemonIpcResponse>(line.trim()),
        Ok(DaemonIpcResponse::Pong { .. })
    )
}
