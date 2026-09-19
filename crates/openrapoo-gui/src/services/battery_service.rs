//! Asynchronous battery service and diagnostic generator for OpenRapoo GPUI.

use openrapoo_core::battery::{
    generate_battery_diagnostic_for_device, BatteryDiagnosticReport, BatteryStatus,
};
use openrapoo_core::device::RapooDevice;

/// Battery service providing async-safe battery status refresh and full diagnostic report generation.
pub struct BatteryService;

impl BatteryService {
    /// Refresh battery status for a single device.
    #[allow(dead_code)]
    pub fn refresh_battery(device: &mut RapooDevice) -> BatteryStatus {
        let status =
            openrapoo_core::battery::query_battery_for_identity(&device.to_device_identity());
        device.battery_status = status.clone();
        status
    }

    /// Generate full battery diagnostic report for a device.
    pub fn get_diagnostic_report(device: &RapooDevice) -> BatteryDiagnosticReport {
        let mut log = Vec::new();
        generate_battery_diagnostic_for_device(
            &device.name,
            device.connection,
            device.phys.as_deref(),
            device.hidraw_path.as_deref(),
            &mut log,
        )
    }
}
