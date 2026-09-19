//! Integration tests for device detection, device types, connection state, and battery status.
//!
//! These tests use simulated data to verify that the detection logic and device
//! representations correctly handle Rapoo mice and keyboards. No actual hardware is required.

use openrapoo_core::battery::BatteryStatus;
use openrapoo_core::device::*;
use openrapoo_core::{KnownDevice, RAPOO_VENDOR_ID};
use std::path::PathBuf;

#[test]
fn test_vendor_id_constant() {
    assert_eq!(RAPOO_VENDOR_ID, 0x24AE);
}

#[test]
fn test_rapoo_device_fields() {
    let device = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x186A, // Confirmed PID: ITON Corp. Rapoo NearLink Mouse
        name: "Rapoo MT760 Pro".to_string(),
        device_type: DeviceType::Mouse,
        connection_state: DeviceConnectionState::Connected,
        connection: ConnectionType::TwoPointFourGhz,
        receiver_state: ReceiverState::ReceiverActive,
        battery_status: BatteryStatus::Available {
            percentage: 85,
            charging: false,
            source: Some(openrapoo_core::battery::BatterySource::Sysfs),
            timestamp: None,
            diagnostic_message: None,
        },
        capabilities: DeviceCapabilities {
            has_buttons_remapping: true,
            has_pointer_settings: true,
            has_battery_reader: true,
            can_read_dpi: true,
            can_set_dpi: false,
            can_read_polling_rate: true,
            can_set_polling_rate: false,
            svg_asset_name: Some("openrapoo-mt760-pro.svg".to_string()),
        },
        model: KnownDevice::RapooMt760Pro,
        evdev_path: Some(PathBuf::from("/dev/input/event5")),
        hidraw_path: Some(PathBuf::from("/dev/hidraw2")),
        phys: Some("usb-0000:00:14.0-1/input0".to_string()),
        ..Default::default()
    };

    assert!(device.is_mt760_pro());
    assert_eq!(device.vendor_id, RAPOO_VENDOR_ID);
    assert_eq!(device.connection.to_string(), "Dongle USB / 2.4 GHz");
    assert_eq!(device.model.to_string(), "Rapoo MT760 Pro");
    assert_eq!(device.device_type.to_string(), "Mouse");
    assert_eq!(device.connection_state.to_string(), "Conectado");
    assert_eq!(
        device.capabilities.svg_asset_name,
        Some("openrapoo-mt760-pro.svg".to_string())
    );
}

#[test]
fn test_non_rapoo_device_not_mt760_pro() {
    let device = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x9999,
        name: "Rapoo Unknown".to_string(),
        device_type: DeviceType::Other,
        connection_state: DeviceConnectionState::Disconnected,
        connection: ConnectionType::Unknown,
        receiver_state: ReceiverState::Disconnected,
        battery_status: BatteryStatus::Unknown,
        capabilities: DeviceCapabilities::default(),
        model: KnownDevice::UnknownRapoo,
        evdev_path: None,
        hidraw_path: None,
        phys: None,
        ..Default::default()
    };

    assert!(!device.is_mt760_pro());
    assert_eq!(device.connection_state.to_string(), "Desconectado");
}

#[test]
fn test_connection_type_serialization() {
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
fn test_device_type_and_connection_state_serialization() {
    let dev_types = [DeviceType::Mouse, DeviceType::Keyboard, DeviceType::Other];
    for dt in &dev_types {
        let json = serde_json::to_string(dt).expect("serialize");
        let decoded: DeviceType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(*dt, decoded);
    }

    let states = [
        DeviceConnectionState::Connected,
        DeviceConnectionState::Disconnected,
        DeviceConnectionState::Reconnecting,
    ];
    for st in &states {
        let json = serde_json::to_string(&st).expect("serialize");
        let decoded: DeviceConnectionState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(*st, decoded);
    }
}

#[test]
fn test_battery_status_variants() {
    let avail = BatteryStatus::Available {
        percentage: 75,
        charging: true,
        source: Some(openrapoo_core::battery::BatterySource::UPower),
        timestamp: Some(1000),
        diagnostic_message: None,
    };
    let unavail = BatteryStatus::Unavailable {
        reason: "Sem leitor de bateria no driver hid-generic".to_string(),
        source: Some(openrapoo_core::battery::BatterySource::HidVendorSpecific),
    };
    let unknown = BatteryStatus::Unknown;

    let json_avail = serde_json::to_string(&avail).expect("serialize battery avail");
    let dec_avail: BatteryStatus = serde_json::from_str(&json_avail).expect("deserialize");
    assert_eq!(avail, dec_avail);

    let json_unavail = serde_json::to_string(&unavail).expect("serialize battery unavail");
    let dec_unavail: BatteryStatus = serde_json::from_str(&json_unavail).expect("deserialize");
    assert_eq!(unavail, dec_unavail);

    let json_unk = serde_json::to_string(&unknown).expect("serialize battery unknown");
    let dec_unk: BatteryStatus = serde_json::from_str(&json_unk).expect("deserialize");
    assert_eq!(unknown, dec_unk);
}

#[test]
fn test_mouse_and_keyboard_simultaneous_types() {
    let mouse = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x186A,
        name: "Rapoo MT760 Pro Mouse".to_string(),
        device_type: DeviceType::Mouse,
        connection_state: DeviceConnectionState::Connected,
        connection: ConnectionType::TwoPointFourGhz,
        receiver_state: ReceiverState::ReceiverActive,
        battery_status: BatteryStatus::Available {
            percentage: 90,
            charging: false,
            source: Some(openrapoo_core::battery::BatterySource::Sysfs),
            timestamp: None,
            diagnostic_message: None,
        },
        capabilities: DeviceCapabilities {
            has_buttons_remapping: true,
            has_pointer_settings: true,
            has_battery_reader: true,
            can_read_dpi: true,
            can_set_dpi: false,
            can_read_polling_rate: true,
            can_set_polling_rate: false,
            svg_asset_name: Some("openrapoo-mt760-pro.svg".to_string()),
        },
        model: KnownDevice::RapooMt760Pro,
        evdev_path: Some(PathBuf::from("/dev/input/event5")),
        hidraw_path: Some(PathBuf::from("/dev/hidraw2")),
        phys: Some("usb-0000:00:14.0-1/input0".to_string()),
        ..Default::default()
    };

    let keyboard = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x2019,
        name: "Rapoo E9500M Keyboard".to_string(),
        device_type: DeviceType::Keyboard,
        connection_state: DeviceConnectionState::Connected,
        connection: ConnectionType::Bluetooth,
        receiver_state: ReceiverState::BluetoothConnected,
        battery_status: BatteryStatus::Available {
            percentage: 60,
            charging: false,
            source: Some(openrapoo_core::battery::BatterySource::BlueZ),
            timestamp: None,
            diagnostic_message: None,
        },
        capabilities: DeviceCapabilities {
            has_buttons_remapping: true,
            has_pointer_settings: false,
            has_battery_reader: true,
            can_read_dpi: false,
            can_set_dpi: false,
            can_read_polling_rate: false,
            can_set_polling_rate: false,
            svg_asset_name: None,
        },
        model: KnownDevice::UnknownRapoo,
        evdev_path: Some(PathBuf::from("/dev/input/event6")),
        hidraw_path: Some(PathBuf::from("/dev/hidraw3")),
        phys: Some("bluetooth/hci0:12345".to_string()),
        ..Default::default()
    };

    assert_eq!(mouse.device_type, DeviceType::Mouse);
    assert_eq!(keyboard.device_type, DeviceType::Keyboard);
    assert!(mouse.capabilities.has_pointer_settings);
    assert!(!keyboard.capabilities.has_pointer_settings);
    assert_eq!(mouse.connection.to_string(), "Dongle USB / 2.4 GHz");
    assert_eq!(keyboard.connection.to_string(), "Bluetooth");
}

#[test]
fn test_rapoo_device_evdev_accessible_false_when_path_not_set() {
    let device = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x2018,
        name: "Test".to_string(),
        device_type: DeviceType::Mouse,
        connection_state: DeviceConnectionState::Connected,
        connection: ConnectionType::Unknown,
        receiver_state: ReceiverState::Unknown,
        battery_status: BatteryStatus::Unknown,
        capabilities: DeviceCapabilities::default(),
        model: KnownDevice::RapooMt760Pro,
        evdev_path: None,
        hidraw_path: None,
        phys: None,
        ..Default::default()
    };

    assert!(!device.evdev_accessible());
    assert!(!device.hidraw_accessible());
}

#[test]
fn test_rapoo_device_evdev_accessible_false_when_path_does_not_exist() {
    let device = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x2018,
        name: "Test".to_string(),
        device_type: DeviceType::Mouse,
        connection_state: DeviceConnectionState::Connected,
        connection: ConnectionType::Unknown,
        receiver_state: ReceiverState::Unknown,
        battery_status: BatteryStatus::Unknown,
        capabilities: DeviceCapabilities::default(),
        model: KnownDevice::RapooMt760Pro,
        evdev_path: Some(PathBuf::from("/dev/input/event_does_not_exist_99999")),
        hidraw_path: Some(PathBuf::from("/dev/hidraw_does_not_exist_99999")),
        phys: None,
        ..Default::default()
    };

    assert!(!device.evdev_accessible());
    assert!(!device.hidraw_accessible());
}

#[test]
fn test_bluetooth_pid_0x4510_recognition() {
    let dev = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x4510,
        name: "Rapoo BT Mouse".to_string(),
        device_type: DeviceType::Mouse,
        connection_state: DeviceConnectionState::Connected,
        connection: ConnectionType::Bluetooth,
        receiver_state: ReceiverState::BluetoothConnected,
        battery_status: BatteryStatus::Available {
            percentage: 70,
            charging: false,
            source: Some(openrapoo_core::battery::BatterySource::BlueZ),
            timestamp: Some(1000),
            diagnostic_message: None,
        },
        capabilities: DeviceCapabilities::default(),
        model: KnownDevice::RapooMt760Pro,
        evdev_path: None,
        hidraw_path: None,
        phys: Some("bluetooth/hci1:dev_DE_ED_DC_41_7C_51".to_string()),
        ..Default::default()
    };

    assert!(dev.is_mt760_pro());
    assert_eq!(dev.connection, ConnectionType::Bluetooth);
    assert_eq!(dev.battery_status.percentage(), Some(70));
}
