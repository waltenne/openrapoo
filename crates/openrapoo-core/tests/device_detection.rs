//! Integration tests for device detection.
//!
//! These tests use simulated data to verify that the detection logic correctly
//! identifies Rapoo devices. No actual hardware is required.

use openrapoo_core::device::*;
use openrapoo_core::{KnownDevice, RAPOO_VENDOR_ID};

#[test]
fn test_vendor_id_constant() {
    assert_eq!(RAPOO_VENDOR_ID, 0x24AE);
}

#[test]
fn test_rapoo_device_fields() {
    use openrapoo_core::device::ConnectionType;
    use std::path::PathBuf;

    let device = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x186A, // Confirmed PID: ITON Corp. Rapoo NearLink Mouse
        name: "Rapoo MT760 Pro".to_string(),
        connection: ConnectionType::TwoPointFourGhz,
        model: KnownDevice::RapooMt760Pro,
        evdev_path: Some(PathBuf::from("/dev/input/event5")),
        hidraw_path: Some(PathBuf::from("/dev/hidraw2")),
        phys: Some("usb-0000:00:14.0-1/input0".to_string()),
    };

    assert!(device.is_mt760_pro());
    assert_eq!(device.vendor_id, RAPOO_VENDOR_ID);
    assert_eq!(device.connection.to_string(), "2.4 GHz wireless");
    assert_eq!(device.model.to_string(), "Rapoo MT760 Pro");
}

#[test]
fn test_non_rapoo_device_not_mt760_pro() {
    use openrapoo_core::device::ConnectionType;

    let device = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x9999,
        name: "Rapoo Unknown".to_string(),
        connection: ConnectionType::Unknown,
        model: KnownDevice::UnknownRapoo,
        evdev_path: None,
        hidraw_path: None,
        phys: None,
    };

    assert!(!device.is_mt760_pro());
}

#[test]
fn test_connection_type_serialization() {
    use openrapoo_core::device::ConnectionType;

    let types = [
        ConnectionType::UsbWired,
        ConnectionType::TwoPointFourGhz,
        ConnectionType::Bluetooth,
        ConnectionType::NearLink,
        ConnectionType::Unknown,
    ];

    for conn_type in &types {
        let json = serde_json::to_string(conn_type).expect("serialize");
        let decoded: ConnectionType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(*conn_type, decoded);
    }
}

#[test]
fn test_known_device_serialization() {
    let models = [KnownDevice::RapooMt760Pro, KnownDevice::UnknownRapoo];
    for model in &models {
        let json = serde_json::to_string(model).expect("serialize");
        let decoded: KnownDevice = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(*model, decoded);
    }
}

#[test]
fn test_rapoo_device_evdev_accessible_false_when_path_not_set() {
    use openrapoo_core::device::ConnectionType;

    let device = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x2018,
        name: "Test".to_string(),
        connection: ConnectionType::Unknown,
        model: KnownDevice::RapooMt760Pro,
        evdev_path: None,
        hidraw_path: None,
        phys: None,
    };

    assert!(!device.evdev_accessible());
    assert!(!device.hidraw_accessible());
}

#[test]
fn test_rapoo_device_evdev_accessible_false_when_path_does_not_exist() {
    use openrapoo_core::device::ConnectionType;
    use std::path::PathBuf;

    let device = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x2018,
        name: "Test".to_string(),
        connection: ConnectionType::Unknown,
        model: KnownDevice::RapooMt760Pro,
        evdev_path: Some(PathBuf::from("/dev/input/event_does_not_exist_99999")),
        hidraw_path: Some(PathBuf::from("/dev/hidraw_does_not_exist_99999")),
        phys: None,
    };

    assert!(!device.evdev_accessible());
    assert!(!device.hidraw_accessible());
}
