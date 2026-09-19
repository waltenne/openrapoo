//! `openrapoo-diag` — Diagnostic CLI tool for the Rapoo MT760 Pro on Linux.
//!
//! # Safety
//! This tool is **read-only**: it never sends commands to the mouse.
//! All HID operations are limited to reading descriptors and feature reports.
//!
//! # Subcommands
//! - `list-devices`     — List all connected Rapoo devices
//! - `capture-events`   — Capture and display evdev events in real time
//! - `identify-buttons` — Interactive mode: press each button to identify it
//! - `hid-report`       — Dump HID descriptor and feature reports (read-only)
//! - `generate-report`  — Generate a complete technical report in Markdown
#![allow(
    clippy::print_literal,
    clippy::useless_format,
    clippy::print_with_newline,
    clippy::ptr_arg
)]

mod detector;
mod event_capture;
mod hid_probe;
mod identify;
mod report;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "openrapoo-diag",
    about = "OpenRapoo Diagnostic Tool — Read-only hardware investigation for Rapoo MT760 Pro",
    version,
    author,
    long_about = "
openrapoo-diag helps you investigate how the Rapoo MT760 Pro is recognised by Linux.

SAFETY: This tool never writes to the mouse. All operations are read-only.
Detected events are only displayed, never acted upon.

For contributing diagnostic data to the project, run:
  openrapoo-diag generate-report
and attach the output file to a GitHub issue.
"
)]
struct Cli {
    /// Enable verbose logging (use -v, -vv, or -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Output format for results
    #[arg(short, long, value_enum, default_value = "human", global = true)]
    output: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all Rapoo devices currently connected to this system
    #[command(alias = "ls")]
    ListDevices,

    /// Capture and display input events from the mouse in real time
    ///
    /// Press Ctrl+C to stop.
    CaptureEvents {
        /// Path to the evdev device node (e.g. /dev/input/event5)
        /// If not provided, uses the first detected Rapoo device
        #[arg(short, long)]
        device: Option<PathBuf>,

        /// Maximum number of events to capture (0 = unlimited)
        #[arg(short, long, default_value = "0")]
        limit: u64,

        /// Show raw event codes without decoding
        #[arg(long)]
        raw: bool,
    },

    /// Interactive button identification mode
    ///
    /// Press each button on the mouse to see its event code.
    /// Press Ctrl+C or Q to exit.
    IdentifyButtons {
        /// Path to the evdev device node
        #[arg(short, long)]
        device: Option<PathBuf>,
    },

    /// Dump HID descriptor and feature reports (read-only)
    ///
    /// This command only READS from the device. It never sends any command.
    HidReport {
        /// Path to the hidraw device node (e.g. /dev/hidraw2)
        #[arg(short, long)]
        device: Option<PathBuf>,

        /// Show raw bytes (hex dump) in addition to parsed output
        #[arg(long)]
        hex: bool,
    },

    /// Generate a complete technical diagnostic report
    ///
    /// The report includes:
    /// - All connected Rapoo devices (VID/PID, paths)
    /// - HID descriptor dump
    /// - Capability analysis
    /// - System information
    ///
    /// The report is saved to ~/.local/share/openrapoo/report-YYYY-MM-DD.md
    /// and does NOT contain any personal information.
    GenerateReport {
        /// Override output path for the report
        #[arg(short = 'p', long)]
        output_path: Option<PathBuf>,

        /// Open the report in the default Markdown viewer after generation
        #[arg(long)]
        open: bool,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum OutputFormat {
    /// Human-readable coloured output
    Human,
    /// JSON output for scripting
    Json,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging based on verbosity flag
    let level = match cli.verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
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

    match cli.command {
        Commands::ListDevices => {
            detector::run_list_devices(&cli.output).context("Failed to list devices")?;
        }

        Commands::CaptureEvents { device, limit, raw } => {
            event_capture::run_capture_events(device, limit, raw, &cli.output)
                .await
                .context("Failed to capture events")?;
        }

        Commands::IdentifyButtons { device } => {
            identify::run_identify_buttons(device)
                .await
                .context("Failed to run button identification")?;
        }

        Commands::HidReport { device, hex } => {
            hid_probe::run_hid_report(device, hex, &cli.output)
                .context("Failed to read HID report")?;
        }

        Commands::GenerateReport { output_path, open } => {
            report::run_generate_report(output_path, open)
                .await
                .context("Failed to generate report")?;
        }
    }

    Ok(())
}
