//! openrapoo-daemon — Background remapping service.
//!
//! ## Status: STUB (Phase 1)
//!
//! This binary is a placeholder for Phase 3 (Software Remapping).
//! The actual implementation will:
//!   - Open evdev device with exclusive grab (EVIOCGRAB)
//!   - Read events from the physical mouse
//!   - Apply user-configured remapping rules
//!   - Inject remapped events via /dev/uinput
//!   - Listen for profile change requests via D-Bus
//!   - Switch profiles based on active application
//!
//! ## Architecture (planned)
//!
//! ```
//! [physical mouse] → evdev grab → [remapping engine] → uinput inject → [OS]
//!                                          ↑
//!                              [D-Bus interface] ← [GUI / CLI]
//! ```

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .init();

    info!("openrapoo-daemon v{}", env!("CARGO_PKG_VERSION"));
    info!("Status: stub — Phase 3 not yet implemented");
    info!("The remapping daemon will be implemented in Phase 3.");
    info!("For now, use `openrapoo-diag` for hardware investigation.");

    eprintln!();
    eprintln!("openrapoo-daemon: Phase 3 (Software Remapping) not yet implemented.");
    eprintln!("See ROADMAP.md for the development plan.");
    eprintln!();

    Ok(())
}

