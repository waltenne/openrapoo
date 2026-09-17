//! # openrapoo-core
//!
//! Core library for OpenRapoo. Provides shared types, device detection,
//! error handling, and configuration structures used by all other crates.

pub mod device;
pub mod error;
pub mod event;
pub mod config;
pub mod permissions;

pub use device::{ConnectionType, KnownDevice, RapooDevice};
pub use error::OpenRapooError;

/// Rapoo Vendor ID (Shenzhen Rapoo Technology Co., Ltd.)
pub const RAPOO_VENDOR_ID: u16 = 0x24AE;

/// Known Rapoo MT760 Pro Product IDs.
/// PIDs confirmed by running `openrapoo-diag list-devices` on real hardware.
/// The NearLink/2.4GHz receiver exposes multiple HID interfaces under the same PID.
pub mod known_pids {
    /// MT760 Pro via NearLink/2.4 GHz USB receiver (CONFIRMED: ITON Corp. Rapoo NearLink Mouse)
    /// Exposes 3 HID interfaces: Mouse (input0), Keyboard (input1), Mouse (input1)
    pub const MT760_PRO_NEARLINK: u16 = 0x186A;
    /// MT760 Pro via Bluetooth — UNVERIFIED, needs hardware test
    pub const MT760_PRO_BT: u16 = 0x186B; // placeholder
    /// MT760 Pro wired USB-C — UNVERIFIED, needs hardware test
    pub const MT760_PRO_WIRED: u16 = 0x186C; // placeholder
}
