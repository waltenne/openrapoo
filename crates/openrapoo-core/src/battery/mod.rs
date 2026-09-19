//! Multi-provider, multi-transport battery detection subsystem for OpenRapoo.

pub mod aggregator;
pub mod bluez;
pub mod bluez_gatt;
pub mod diagnostic;
pub mod environment;
pub mod hid_standard;
pub mod hid_vendor;
pub mod hidraw;
pub mod provider;
pub mod sysfs;
pub mod types;
pub mod upower;

// Public re-exports for module access
pub use aggregator::{AggregatedResult, BatteryAggregator};
pub use bluez::BluezBatteryProvider;
pub use bluez_gatt::BluezGattProvider;
pub use diagnostic::{generate_battery_diagnostic_report, BatteryDiagnosticReport};
pub use environment::{collect_system_environment, SystemEnvironment};
pub use hid_standard::StandardHidProvider;
pub use hid_vendor::{parse_rapoo_battery_response, RapooVendorHidProvider};
pub use hidraw::{scan_hidraw_interfaces, HidrawCandidate};
pub use provider::{current_epoch_seconds, BatteryProvider, DeviceIdentity};
pub use sysfs::SysfsProvider;
pub use types::{
    BatteryConfidence, BatteryReading, BatterySource, BatteryState, BatteryStatus, BatteryValidity,
    ChargingInfo, ConflictInfo, PowerSource, RawProviderData,
};
pub use upower::UPowerProvider;

use std::path::Path;

/// Query battery status for a complete DeviceIdentity.
pub fn query_battery_for_identity(identity: &DeviceIdentity) -> BatteryStatus {
    let aggregator = BatteryAggregator::new();
    let mut log = Vec::new();
    let result = aggregator.query_all(identity, &mut log);

    result.chosen.into()
}

/// Query battery status for a device specifying its attributes.
pub fn query_battery_for_device(
    device_name: &str,
    transport: crate::device::ConnectionType,
    phys: Option<&str>,
    hidraw_path: Option<&Path>,
) -> BatteryStatus {
    let identity = DeviceIdentity {
        name: device_name.to_string(),
        transport,
        phys: phys.map(|s| s.to_string()),
        hidraw_path: hidraw_path.map(|p| p.to_path_buf()),
        ..Default::default()
    };

    query_battery_for_identity(&identity)
}

/// Public backward-compatible entry point for querying battery status across all providers.
pub fn query_battery_multi_provider(
    device_name: &str,
    phys: Option<&str>,
    hidraw_path: Option<&Path>,
) -> BatteryStatus {
    query_battery_for_device(
        device_name,
        crate::device::ConnectionType::Unknown,
        phys,
        hidraw_path,
    )
}

/// Public entry point for generating full battery diagnostic reports for a device specifying transport.
pub fn generate_battery_diagnostic_for_device(
    device_name: &str,
    transport: crate::device::ConnectionType,
    phys: Option<&str>,
    hidraw_path: Option<&Path>,
    log: &mut Vec<String>,
) -> BatteryDiagnosticReport {
    let report = generate_battery_diagnostic_report(device_name, transport, phys, hidraw_path);
    log.extend(report.diagnostic_log.clone());
    report
}

/// Public backward-compatible entry point for generating full battery diagnostic reports.
pub fn generate_battery_diagnostic(
    device_name: &str,
    phys: Option<&str>,
    hidraw_path: Option<&Path>,
    log: &mut Vec<String>,
) -> BatteryDiagnosticReport {
    generate_battery_diagnostic_for_device(
        device_name,
        crate::device::ConnectionType::Unknown,
        phys,
        hidraw_path,
        log,
    )
}

/// Helper for sysfs direct battery query.
pub fn read_sysfs_battery(device_name: &str) -> BatteryStatus {
    query_battery_multi_provider(device_name, None, None)
}

/// Convenience wrapper for hidraw battery query.
pub fn query_rapoo_hidraw_battery(
    primary_hidraw_path: &Path,
    log: &mut Vec<String>,
) -> Option<BatteryStatus> {
    let identity = DeviceIdentity {
        hidraw_path: Some(primary_hidraw_path.to_path_buf()),
        ..Default::default()
    };
    let provider = RapooVendorHidProvider::new();
    provider.query(&identity, log).map(|r| r.into())
}
