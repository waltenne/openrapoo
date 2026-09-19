//! Unit and integration tests for USB receivers, Bluetooth connections, and device isolation.

use openrapoo_core::device::{
    ConnectionType, DeviceConnectionState, DeviceType, KnownDevice, RapooDevice, ReceiverState,
};

#[test]
fn test_receiver_state_variants() {
    let state = ReceiverState::ReceiverPresent;
    assert_eq!(format!("{state}"), "Receptor USB Conectado (Sem Mouse)");

    let active = ReceiverState::ReceiverActive;
    assert_eq!(format!("{active}"), "Mouse Ativo via 2.4GHz");

    let bt = ReceiverState::BluetoothConnected;
    assert_eq!(format!("{bt}"), "Conectado via Bluetooth");
}

#[test]
fn test_bluetooth_connected_and_dongle_without_mouse_returns_bt_only() {
    let bt_mouse = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x4510,
        name: "Rapoo MT760 Pro (Bluetooth)".to_string(),
        device_type: DeviceType::Mouse,
        connection_state: DeviceConnectionState::Connected,
        connection: ConnectionType::Bluetooth,
        receiver_state: ReceiverState::BluetoothConnected,
        battery_status: openrapoo_core::battery::BatteryStatus::Unknown,
        capabilities: Default::default(),
        model: KnownDevice::RapooMt760Pro,
        evdev_path: None,
        hidraw_path: None,
        phys: Some("bluetooth/hci0/dev_DE_ED_DC_41_7C_51".to_string()),
        ..Default::default()
    };

    let idle_dongle = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x186A,
        name: "Rapoo MT760 Pro".to_string(),
        device_type: DeviceType::Mouse,
        connection_state: DeviceConnectionState::Connected,
        connection: ConnectionType::TwoPointFourGhz,
        receiver_state: ReceiverState::ReceiverPresent,
        battery_status: openrapoo_core::battery::BatteryStatus::Unknown,
        capabilities: Default::default(),
        model: KnownDevice::RapooMt760Pro,
        evdev_path: None,
        hidraw_path: None,
        phys: Some("usb-0000:00:14.0-1/input0".to_string()),
        ..Default::default()
    };

    let all_detected = vec![bt_mouse.clone(), idle_dongle];
    let active_logical_devices: Vec<_> = all_detected
        .into_iter()
        .filter(|d| d.receiver_state != ReceiverState::ReceiverPresent)
        .collect();

    // Must return exactly 1 device (the Bluetooth connected mouse)
    assert_eq!(active_logical_devices.len(), 1);
    assert_eq!(
        active_logical_devices[0].connection,
        ConnectionType::Bluetooth
    );
}

#[test]
fn test_only_bluetooth_connected() {
    let bt_mouse = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x4510,
        name: "Rapoo MT760 Pro (Bluetooth)".to_string(),
        device_type: DeviceType::Mouse,
        connection_state: DeviceConnectionState::Connected,
        connection: ConnectionType::Bluetooth,
        receiver_state: ReceiverState::BluetoothConnected,
        battery_status: openrapoo_core::battery::BatteryStatus::Unknown,
        capabilities: Default::default(),
        model: KnownDevice::RapooMt760Pro,
        evdev_path: None,
        hidraw_path: None,
        phys: Some("bluetooth/hci0/dev_DE_ED_DC_41_7C_51".to_string()),
        ..Default::default()
    };

    let devices = vec![bt_mouse];
    let active: Vec<_> = devices
        .into_iter()
        .filter(|d| d.receiver_state != ReceiverState::ReceiverPresent)
        .collect();
    assert_eq!(active.len(), 1);
}

#[test]
fn test_only_dongle_connected_with_mouse_active() {
    let active_mouse = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x186A,
        name: "Rapoo MT760 Pro".to_string(),
        device_type: DeviceType::Mouse,
        connection_state: DeviceConnectionState::Connected,
        connection: ConnectionType::TwoPointFourGhz,
        receiver_state: ReceiverState::ReceiverActive,
        battery_status: openrapoo_core::battery::BatteryStatus::Unknown,
        capabilities: Default::default(),
        model: KnownDevice::RapooMt760Pro,
        evdev_path: Some(std::path::PathBuf::from("/dev/input/event5")),
        hidraw_path: Some(std::path::PathBuf::from("/dev/hidraw2")),
        phys: Some("usb-0000:00:14.0-1/input0".to_string()),
        ..Default::default()
    };

    let devices = vec![active_mouse];
    let active: Vec<_> = devices
        .into_iter()
        .filter(|d| d.receiver_state != ReceiverState::ReceiverPresent)
        .collect();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].connection, ConnectionType::TwoPointFourGhz);
}

#[test]
fn test_only_dongle_connected_without_mouse_idle() {
    let idle_dongle = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x186A,
        name: "Rapoo MT760 Pro".to_string(),
        device_type: DeviceType::Mouse,
        connection_state: DeviceConnectionState::Connected,
        connection: ConnectionType::TwoPointFourGhz,
        receiver_state: ReceiverState::ReceiverPresent,
        battery_status: openrapoo_core::battery::BatteryStatus::Unknown,
        capabilities: Default::default(),
        model: KnownDevice::RapooMt760Pro,
        evdev_path: None,
        hidraw_path: None,
        phys: Some("usb-0000:00:14.0-1/input0".to_string()),
        ..Default::default()
    };

    let devices = vec![idle_dongle];
    let active: Vec<_> = devices
        .into_iter()
        .filter(|d| d.receiver_state != ReceiverState::ReceiverPresent)
        .collect();
    assert_eq!(active.len(), 0);
}

#[test]
fn test_two_rapoo_devices_keyboard_and_mouse() {
    let kbd = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x1008,
        name: "Rapoo E9050L".to_string(),
        device_type: DeviceType::Keyboard,
        connection_state: DeviceConnectionState::Connected,
        connection: ConnectionType::TwoPointFourGhz,
        receiver_state: ReceiverState::ReceiverActive,
        battery_status: openrapoo_core::battery::BatteryStatus::Unknown,
        capabilities: Default::default(),
        model: KnownDevice::RapooE9050L,
        evdev_path: Some(std::path::PathBuf::from("/dev/input/event3")),
        hidraw_path: None,
        phys: Some("usb-0000:00:14.0-2/input0".to_string()),
        ..Default::default()
    };

    let mouse = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x4510,
        name: "Rapoo MT760 Pro (Bluetooth)".to_string(),
        device_type: DeviceType::Mouse,
        connection_state: DeviceConnectionState::Connected,
        connection: ConnectionType::Bluetooth,
        receiver_state: ReceiverState::BluetoothConnected,
        battery_status: openrapoo_core::battery::BatteryStatus::Unknown,
        capabilities: Default::default(),
        model: KnownDevice::RapooMt760Pro,
        evdev_path: None,
        hidraw_path: None,
        phys: Some("bluetooth/hci0/dev_DE_ED_DC_41_7C_51".to_string()),
        ..Default::default()
    };

    let devices = vec![kbd, mouse];
    let active: Vec<_> = devices
        .into_iter()
        .filter(|d| d.receiver_state != ReceiverState::ReceiverPresent)
        .collect();
    assert_eq!(active.len(), 2);
}

#[test]
fn test_same_name_different_pid_distinct() {
    let mouse_24g = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x186A,
        name: "Rapoo MT760 Pro".to_string(),
        device_type: DeviceType::Mouse,
        connection_state: DeviceConnectionState::Connected,
        connection: ConnectionType::TwoPointFourGhz,
        receiver_state: ReceiverState::ReceiverActive,
        battery_status: openrapoo_core::battery::BatteryStatus::Unknown,
        capabilities: Default::default(),
        model: KnownDevice::RapooMt760Pro,
        evdev_path: None,
        hidraw_path: None,
        phys: None,
        ..Default::default()
    };

    let mouse_bt = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x4510,
        name: "Rapoo MT760 Pro (Bluetooth)".to_string(),
        device_type: DeviceType::Mouse,
        connection_state: DeviceConnectionState::Connected,
        connection: ConnectionType::Bluetooth,
        receiver_state: ReceiverState::BluetoothConnected,
        battery_status: openrapoo_core::battery::BatteryStatus::Unknown,
        capabilities: Default::default(),
        model: KnownDevice::RapooMt760Pro,
        evdev_path: None,
        hidraw_path: None,
        phys: None,
        ..Default::default()
    };

    assert_ne!(mouse_24g.product_id, mouse_bt.product_id);
    assert_ne!(mouse_24g.connection, mouse_bt.connection);
}

#[test]
fn test_usb_cable_connected_mouse_priority() {
    let wired_mouse = RapooDevice {
        vendor_id: 0x24AE,
        product_id: 0x4510,
        name: "Rapoo MT760 Pro".to_string(),
        device_type: DeviceType::Mouse,
        connection_state: DeviceConnectionState::Connected,
        connection: ConnectionType::UsbCable,
        receiver_state: ReceiverState::ReceiverActive,
        battery_status: openrapoo_core::battery::BatteryStatus::Unknown,
        capabilities: Default::default(),
        model: KnownDevice::RapooMt760Pro,
        evdev_path: Some(std::path::PathBuf::from("/dev/input/event5")),
        hidraw_path: Some(std::path::PathBuf::from("/dev/hidraw2")),
        phys: Some("usb-0000:04:00.0-3/input0".to_string()),
        ..Default::default()
    };

    assert_eq!(wired_mouse.connection, ConnectionType::UsbCable);
    assert_eq!(wired_mouse.connection.to_string(), "USB por cabo");
}
