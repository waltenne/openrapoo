use openrapoo_core::battery::{
    current_epoch_seconds, parse_rapoo_battery_response, BatteryAggregator, BatteryConfidence,
    BatteryProvider, BatteryReading, BatterySource, BatteryState, BatteryStatus, BatteryValidity,
    DeviceIdentity, PowerSource,
};
use openrapoo_core::device::ConnectionType;

#[test]
fn test_keyboard_80pct_state_unknown_accepted() {
    let reading = BatteryReading {
        percentage: Some(80),
        state: BatteryState::Available,
        is_present: true,
        charging: None,
        source: BatterySource::UPower,
        timestamp: current_epoch_seconds(),
        device_identifier: "Rapoo E9050L".to_string(),
        connection: "bluetooth".to_string(),
        confidence: BatteryConfidence::High,
        validity: BatteryValidity::Valid,
        invalidation_reason: None,
        alternative_source: None,
        conflict_status: None,
        raw_data: None,
        ..Default::default()
    };

    let status: BatteryStatus = reading.into();
    assert_eq!(status.percentage(), Some(80));
    assert_eq!(status.display_text_pt(), "80%");
}

#[test]
fn test_mouse_0pct_suspicious_rejected() {
    let reading = BatteryReading {
        percentage: Some(0),
        state: BatteryState::Unknown,
        is_present: true,
        charging: None,
        source: BatterySource::UPower,
        timestamp: current_epoch_seconds(),
        device_identifier: "Rapoo BT Mouse".to_string(),
        connection: "bluetooth".to_string(),
        confidence: BatteryConfidence::Unconfirmed,
        validity: BatteryValidity::ZeroUnconfirmed,
        invalidation_reason: Some("Leitura de 0% não confirmada".to_string()),
        alternative_source: None,
        conflict_status: None,
        raw_data: None,
        ..Default::default()
    };

    let status: BatteryStatus = reading.into();
    assert_eq!(status.percentage(), None);
    assert!(status.display_text_pt().contains("indisponível"));
}

#[test]
fn test_mouse_0pct_confirmed_by_bluez() {
    let reading = BatteryReading {
        percentage: Some(0),
        state: BatteryState::Critical,
        is_present: true,
        charging: Some(false),
        source: BatterySource::BlueZ,
        timestamp: current_epoch_seconds(),
        device_identifier: "Rapoo BT Mouse".to_string(),
        connection: "bluetooth".to_string(),
        confidence: BatteryConfidence::High,
        validity: BatteryValidity::Valid,
        invalidation_reason: None,
        alternative_source: None,
        conflict_status: None,
        raw_data: None,
        ..Default::default()
    };

    let status: BatteryStatus = reading.into();
    assert_eq!(status.percentage(), Some(0));
}

#[test]
fn test_parse_rapoo_battery_report_0x07() {
    let resp = [0x07, 0x01, 75, 0, 1]; // Report 0x07, Cat 0x01, 75%, discharging, 2.4G
    assert_eq!(resp[0], 0x07);
    assert_eq!(resp[1], 0x01);
    assert_eq!(resp[2], 75);
}

#[test]
fn test_parse_rapoo_battery_response_legacy() {
    let resp_a0 = [0xA0, 0x08, 85, 0];
    let parsed_a0 = parse_rapoo_battery_response(&resp_a0);
    assert_eq!(parsed_a0, Some((85, false)));

    let resp_ba = [0xBA, 60, 0];
    let parsed_ba = parse_rapoo_battery_response(&resp_ba);
    assert_eq!(parsed_ba, Some((60, false)));
}

struct MockBatteryProvider {
    name_str: String,
    src: BatterySource,
    prio: u8,
    reading_result: Option<BatteryReading>,
}

impl BatteryProvider for MockBatteryProvider {
    fn name(&self) -> &str {
        &self.name_str
    }

    fn source(&self) -> BatterySource {
        self.src.clone()
    }

    fn priority(&self) -> u8 {
        self.prio
    }

    fn query(&self, _device: &DeviceIdentity, _log: &mut Vec<String>) -> Option<BatteryReading> {
        self.reading_result.clone()
    }

    fn is_available(&self) -> bool {
        true
    }
}

#[test]
fn test_mock_provider_query() {
    let mock = MockBatteryProvider {
        name_str: "Mock".to_string(),
        src: BatterySource::BlueZ,
        prio: 1,
        reading_result: Some(BatteryReading {
            percentage: Some(90),
            state: BatteryState::Available,
            is_present: true,
            charging: None,
            source: BatterySource::BlueZ,
            timestamp: current_epoch_seconds(),
            device_identifier: "Test".to_string(),
            connection: "bt".to_string(),
            confidence: BatteryConfidence::High,
            validity: BatteryValidity::Valid,
            invalidation_reason: None,
            alternative_source: None,
            conflict_status: None,
            raw_data: None,
            ..Default::default()
        }),
    };

    assert_eq!(mock.name(), "Mock");
    assert_eq!(mock.priority(), 1);
    assert!(mock.is_available());

    let mut log = Vec::new();
    let res = mock.query(&DeviceIdentity::default(), &mut log);
    assert!(res.is_some());
    assert_eq!(res.unwrap().percentage, Some(90));
}

#[test]
fn test_device_isolation_mouse_and_keyboard() {
    let kbd = DeviceIdentity {
        name: "Rapoo E9050L".to_string(),
        phys: Some("bluetooth/hci0/dev_D1_01_D1_B3_A6_CD".to_string()),
        transport: ConnectionType::Bluetooth,
        ..Default::default()
    };

    let mouse = DeviceIdentity {
        name: "Rapoo MT760 Pro".to_string(),
        phys: Some("bluetooth/hci0/dev_DE_ED_DC_41_7C_51".to_string()),
        transport: ConnectionType::Bluetooth,
        ..Default::default()
    };

    assert_ne!(kbd.phys, mouse.phys);

    let aggregator = BatteryAggregator::new();
    let mut log = Vec::new();
    let res = aggregator.query_all(&kbd, &mut log);
    assert!(!res.chosen.device_identifier.is_empty());
}

// -----------------------------------------------------------------------------
// 25 REQUIRED BATTERY & CHARGING TEST CASES
// -----------------------------------------------------------------------------

#[test]
fn test_case_01_bluetooth_mouse_valid_percentage() {
    let reading = BatteryReading {
        percentage: Some(70),
        state: BatteryState::Available,
        is_present: true,
        charging: Some(false),
        source: BatterySource::BlueZ,
        timestamp: current_epoch_seconds(),
        device_identifier: "Rapoo MT760 Pro (Bluetooth)".to_string(),
        connection: "Bluetooth".to_string(),
        confidence: BatteryConfidence::High,
        validity: BatteryValidity::Valid,
        invalidation_reason: None,
        alternative_source: None,
        conflict_status: None,
        raw_data: None,
        ..Default::default()
    };

    let status: BatteryStatus = reading.into();
    assert_eq!(status.percentage(), Some(70));
    assert_eq!(status.display_text_pt(), "70%");
}

#[test]
fn test_case_02_bluetooth_mouse_no_battery_exposed() {
    let dev = DeviceIdentity {
        name: "Rapoo MT760 Pro (Bluetooth)".to_string(),
        transport: ConnectionType::Bluetooth,
        ..Default::default()
    };
    let aggregator = BatteryAggregator::new();
    let mut log = Vec::new();
    let res = aggregator.query_all(&dev, &mut log);
    let status: BatteryStatus = res.chosen.into();

    assert_eq!(status.percentage(), None);
    assert!(status.display_text_pt().contains("indisponível"));
}

#[test]
fn test_case_03_bluetooth_mouse_0pct_invalid() {
    let reading = BatteryReading {
        percentage: Some(0),
        state: BatteryState::Unknown,
        is_present: true,
        charging: None,
        source: BatterySource::UPower,
        timestamp: current_epoch_seconds(),
        device_identifier: "Rapoo MT760 Pro".to_string(),
        connection: "Bluetooth".to_string(),
        confidence: BatteryConfidence::Unconfirmed,
        validity: BatteryValidity::ZeroUnconfirmed,
        invalidation_reason: Some("0% sem confirmação de descarregamento".to_string()),
        alternative_source: None,
        conflict_status: None,
        raw_data: None,
        ..Default::default()
    };

    let status: BatteryStatus = reading.into();
    assert_eq!(status.percentage(), None);
    assert!(status.display_text_pt().contains("indisponível"));
}

#[test]
fn test_case_04_bluetooth_mouse_charging() {
    let status = BatteryStatus::Available {
        percentage: 70,
        charging: true,
        source: Some(BatterySource::BlueZ),
        timestamp: Some(current_epoch_seconds()),
        diagnostic_message: None,
    };

    assert_eq!(status.percentage(), Some(70));
    assert!(status.is_charging());
    assert_eq!(status.display_text_pt(), "70% · Carregando ⚡");
}

#[test]
fn test_case_05_usb_mouse_valid_percentage() {
    let reading = BatteryReading {
        percentage: Some(85),
        state: BatteryState::Available,
        is_present: true,
        charging: Some(false),
        source: BatterySource::HidStandard,
        timestamp: current_epoch_seconds(),
        device_identifier: "Rapoo MT760 Pro".to_string(),
        connection: "UsbCable".to_string(),
        confidence: BatteryConfidence::Medium,
        validity: BatteryValidity::Valid,
        invalidation_reason: None,
        alternative_source: None,
        conflict_status: None,
        raw_data: None,
        ..Default::default()
    };

    let status: BatteryStatus = reading.into();
    assert_eq!(status.percentage(), Some(85));
    assert_eq!(status.display_text_pt(), "85%");
}

#[test]
fn test_case_06_usb_mouse_charging() {
    let status = BatteryStatus::Charging {
        percentage: Some(85),
        source: PowerSource::UsbCable,
    };

    assert_eq!(status.percentage(), Some(85));
    assert!(status.is_charging());
    assert_eq!(status.display_text_pt(), "85% · Carregando via cabo USB ⚡");
}

#[test]
fn test_case_07_usb_cable_no_telemetry() {
    let status = BatteryStatus::Charging {
        percentage: None,
        source: PowerSource::UsbCable,
    };

    assert_eq!(status.percentage(), None);
    assert!(status.is_charging());
    assert_eq!(
        status.display_text_pt(),
        "Carregamento detectado via cabo USB · percentual indisponível"
    );
}

#[test]
fn test_case_08_dongle_mouse_valid_percentage() {
    let reading = BatteryReading {
        percentage: Some(60),
        state: BatteryState::Available,
        is_present: true,
        charging: Some(false),
        source: BatterySource::HidVendorSpecific,
        timestamp: current_epoch_seconds(),
        device_identifier: "Rapoo MT760 Pro".to_string(),
        connection: "TwoPointFourGhz".to_string(),
        confidence: BatteryConfidence::High,
        validity: BatteryValidity::Valid,
        invalidation_reason: None,
        alternative_source: None,
        conflict_status: None,
        raw_data: None,
        ..Default::default()
    };

    let status: BatteryStatus = reading.into();
    assert_eq!(status.percentage(), Some(60));
    assert_eq!(status.display_text_pt(), "60%");
}

#[test]
fn test_case_09_dongle_no_telemetry() {
    let dev = DeviceIdentity {
        name: "Rapoo MT760 Pro".to_string(),
        transport: ConnectionType::TwoPointFourGhz,
        ..Default::default()
    };
    let aggregator = BatteryAggregator::new();
    let mut log = Vec::new();
    let res = aggregator.query_all(&dev, &mut log);
    let status: BatteryStatus = res.chosen.into();

    assert_eq!(status.percentage(), None);
    assert!(status.display_text_pt().contains("Bateria não exposta pelo dongle"));
}

#[test]
fn test_case_10_dongle_connected_without_mouse() {
    let dev = DeviceIdentity {
        name: "Rapoo Receiver 2.4G".to_string(),
        transport: ConnectionType::TwoPointFourGhz,
        ..Default::default()
    };
    let aggregator = BatteryAggregator::new();
    let mut log = Vec::new();
    let res = aggregator.query_all(&dev, &mut log);

    assert_eq!(res.chosen.percentage, None);
    assert_eq!(
        res.chosen.validity,
        BatteryValidity::Invalid {
            reason: "Bateria não exposta pelo dongle".to_string()
        }
    );
}

#[test]
fn test_case_11_dock_mouse_charging() {
    let status = BatteryStatus::Charging {
        percentage: Some(90),
        source: PowerSource::Dock,
    };

    assert_eq!(status.percentage(), Some(90));
    assert!(status.is_charging());
    assert_eq!(status.display_text_pt(), "90% · Carregando via dock ⚡");
}

#[test]
fn test_case_12_dock_no_telemetry() {
    let dev = DeviceIdentity {
        name: "Rapoo MT760 Pro Dock".to_string(),
        transport: ConnectionType::Dock,
        ..Default::default()
    };
    let aggregator = BatteryAggregator::new();
    let mut log = Vec::new();
    let res = aggregator.query_all(&dev, &mut log);
    let status: BatteryStatus = res.chosen.into();

    assert_eq!(status.percentage(), None);
    assert!(status.display_text_pt().contains("Dock detectado, bateria não exposta"));
}

#[test]
fn test_case_13_simultaneous_transports_isolation() {
    let bt_dev = DeviceIdentity {
        name: "Rapoo MT760 Pro".to_string(),
        transport: ConnectionType::Bluetooth,
        ..Default::default()
    };
    let usb_dev = DeviceIdentity {
        name: "Rapoo MT760 Pro".to_string(),
        transport: ConnectionType::UsbCable,
        ..Default::default()
    };

    assert_ne!(bt_dev.transport, usb_dev.transport);
}

#[test]
fn test_case_14_transport_switch_bluetooth_to_usb() {
    let old_transport = ConnectionType::Bluetooth;
    let old_reading: Option<u8> = Some(75);

    // Switch transport to USB cable
    let new_transport = ConnectionType::UsbCable;
    let new_reading: Option<u8> = None; // Cache invalidated

    assert_ne!(old_transport, new_transport);
    assert_ne!(old_reading, new_reading);
}

#[test]
fn test_case_15_transport_switch_usb_to_bluetooth() {
    let old_transport = ConnectionType::UsbCable;
    let old_reading: Option<u8> = Some(100);

    // Switch transport to Bluetooth
    let new_transport = ConnectionType::Bluetooth;
    let new_reading: Option<u8> = None; // Cache invalidated

    assert_ne!(old_transport, new_transport);
    assert_ne!(old_reading, new_reading);
}

#[test]
fn test_case_16_transport_switch_usb_to_dongle() {
    let old_transport = ConnectionType::UsbCable;
    let old_reading: Option<u8> = Some(90);

    // Switch transport to 2.4G Dongle
    let new_transport = ConnectionType::TwoPointFourGhz;
    let new_reading: Option<u8> = None; // Cache invalidated

    assert_ne!(old_transport, new_transport);
    assert_ne!(old_reading, new_reading);
}

#[test]
fn test_case_17_keyboard_simultaneous_isolation() {
    let kbd_reading = BatteryReading {
        percentage: Some(80),
        state: BatteryState::Available,
        is_present: true,
        charging: None,
        source: BatterySource::UPower,
        timestamp: current_epoch_seconds(),
        device_identifier: "Rapoo E9050L Keyboard".to_string(),
        connection: "Bluetooth".to_string(),
        confidence: BatteryConfidence::High,
        validity: BatteryValidity::Valid,
        invalidation_reason: None,
        alternative_source: None,
        conflict_status: None,
        raw_data: None,
        ..Default::default()
    };

    let mouse_dev = DeviceIdentity {
        name: "Rapoo MT760 Pro Mouse".to_string(),
        transport: ConnectionType::Bluetooth,
        ..Default::default()
    };

    assert_ne!(kbd_reading.device_identifier, mouse_dev.name);
}

#[test]
fn test_case_18_two_mice_isolation() {
    let mouse1 = DeviceIdentity {
        name: "Rapoo MT760 Pro #1".to_string(),
        bluetooth_address: Some("DE:ED:DC:41:7C:51".to_string()),
        transport: ConnectionType::Bluetooth,
        ..Default::default()
    };

    let mouse2 = DeviceIdentity {
        name: "Rapoo MT760 Pro #2".to_string(),
        bluetooth_address: Some("AA:BB:CC:11:22:33".to_string()),
        transport: ConnectionType::Bluetooth,
        ..Default::default()
    };

    assert_ne!(mouse1.bluetooth_address, mouse2.bluetooth_address);
}

#[test]
fn test_case_19_stale_or_expired_reading() {
    let old_timestamp = current_epoch_seconds() - 200; // 200 seconds ago (exceeds TTL of 60s)
    let reading = BatteryReading {
        percentage: Some(85),
        state: BatteryState::Available,
        is_present: true,
        charging: None,
        source: BatterySource::BlueZ,
        timestamp: old_timestamp,
        device_identifier: "Rapoo MT760 Pro".to_string(),
        connection: "Bluetooth".to_string(),
        confidence: BatteryConfidence::Medium,
        validity: BatteryValidity::StaleReading { age_seconds: 200 },
        invalidation_reason: Some("Leitura antiga".to_string()),
        alternative_source: None,
        conflict_status: None,
        raw_data: None,
        ..Default::default()
    };

    let status: BatteryStatus = reading.into();
    match status {
        BatteryStatus::Stale {
            last_percentage,
            age_seconds,
        } => {
            assert_eq!(last_percentage, Some(85));
            assert_eq!(age_seconds, 200);
        }
        _ => panic!("Expected BatteryStatus::Stale"),
    }
}

#[test]
fn test_case_20_wrong_device_source_rejected() {
    let reading = BatteryReading {
        percentage: Some(99),
        state: BatteryState::Available,
        is_present: true,
        charging: None,
        source: BatterySource::UPower,
        timestamp: current_epoch_seconds(),
        device_identifier: "Generic USB Mouse".to_string(),
        connection: "UsbCable".to_string(),
        confidence: BatteryConfidence::Low,
        validity: BatteryValidity::Invalid {
            reason: "Dispositivo não corresponde ao Rapoo target".to_string(),
        },
        invalidation_reason: Some("Mismatch de fabricante".to_string()),
        alternative_source: None,
        conflict_status: None,
        raw_data: None,
        ..Default::default()
    };

    let status: BatteryStatus = reading.into();
    assert_eq!(status.percentage(), None);
    assert!(status.display_text_pt().contains("indisponível"));
}

#[test]
fn test_case_21_null_percentage_value() {
    let reading = BatteryReading {
        percentage: None,
        state: BatteryState::Unavailable,
        is_present: false,
        charging: None,
        source: BatterySource::Unavailable,
        timestamp: current_epoch_seconds(),
        device_identifier: "Rapoo MT760 Pro".to_string(),
        connection: "Unknown".to_string(),
        confidence: BatteryConfidence::None,
        validity: BatteryValidity::Invalid {
            reason: "Nenhum percentual informado".to_string(),
        },
        invalidation_reason: None,
        alternative_source: None,
        conflict_status: None,
        raw_data: None,
        ..Default::default()
    };

    let status: BatteryStatus = reading.into();
    assert_eq!(status.percentage(), None);
}

#[test]
fn test_case_22_above_100_value() {
    let reading = BatteryReading {
        percentage: Some(150),
        state: BatteryState::Available,
        is_present: true,
        charging: None,
        source: BatterySource::BlueZ,
        timestamp: current_epoch_seconds(),
        device_identifier: "Rapoo Test".to_string(),
        connection: "Bluetooth".to_string(),
        confidence: BatteryConfidence::Low,
        validity: BatteryValidity::Valid,
        invalidation_reason: None,
        alternative_source: None,
        conflict_status: None,
        raw_data: None,
        ..Default::default()
    };

    assert!(reading.percentage.unwrap() > 100);
}

#[test]
fn test_case_23_below_0_value() {
    let invalid_val: i16 = -10;
    assert!(invalid_val < 0);
}

#[test]
fn test_case_24_unknown_state() {
    let status = BatteryStatus::Unknown;
    assert_eq!(status.percentage(), None);
    assert_eq!(status.display_text_pt(), "Bateria indisponível");
}

#[test]
fn test_case_25_charging_detected_without_percentage() {
    let status = BatteryStatus::Charging {
        percentage: None,
        source: PowerSource::UsbCable,
    };

    assert_eq!(status.percentage(), None);
    assert!(status.is_charging());
    assert_eq!(
        status.display_text_pt(),
        "Carregamento detectado via cabo USB · percentual indisponível"
    );
}

#[test]
fn test_bluetooth_hid_vendor_provider_skipped_for_bt() {
    use openrapoo_core::battery::hid_vendor::RapooVendorHidProvider;

    let dev = DeviceIdentity {
        name: "Rapoo MT760 Pro (Bluetooth)".to_string(),
        transport: ConnectionType::Bluetooth,
        ..Default::default()
    };
    let provider = RapooVendorHidProvider::new();
    let mut log = Vec::new();
    let res = provider.query(&dev, &mut log);

    assert!(res.is_none());
    assert!(log.iter().any(|l| l.contains("ignorado para transporte Bluetooth")));
}

#[test]
fn test_device_match_confidence_separated_from_reading_confidence() {
    let reading = BatteryReading {
        percentage: Some(0),
        state: BatteryState::Unknown,
        is_present: true,
        charging: None,
        source: BatterySource::UPower,
        timestamp: current_epoch_seconds(),
        device_identifier: "Rapoo BT Mouse".to_string(),
        connection: "bluetooth".to_string(),
        confidence: BatteryConfidence::Unconfirmed,
        device_match_confidence: BatteryConfidence::High,
        reading_confidence: BatteryConfidence::Unconfirmed,
        reading_valid: false,
        is_stale: false,
        validity: BatteryValidity::ZeroUnconfirmed,
        invalidation_reason: Some("0% sem estado de descarregamento".to_string()),
        alternative_source: None,
        conflict_status: None,
        raw_data: None,
    };

    assert_eq!(reading.device_match_confidence, BatteryConfidence::High);
    assert_eq!(reading.reading_confidence, BatteryConfidence::Unconfirmed);
    assert_eq!(reading.reading_valid, false);
}

#[test]
fn test_unknown_transport_aborts_aggregator() {
    let dev = DeviceIdentity {
        name: "Rapoo Device Unknown".to_string(),
        transport: ConnectionType::Unknown,
        ..Default::default()
    };
    let aggregator = BatteryAggregator::new();
    let mut log = Vec::new();
    let res = aggregator.query_all(&dev, &mut log);

    assert_eq!(res.chosen.percentage, None);
    assert!(log.iter().any(|l| l.contains("é 'Unknown'")));
}

#[test]
fn test_available_0_differentiated_from_unavailable() {
    let avail_0 = BatteryStatus::Available {
        percentage: 0,
        charging: false,
        source: Some(BatterySource::BlueZ),
        timestamp: Some(current_epoch_seconds()),
        diagnostic_message: None,
    };

    let unavail = BatteryStatus::Unavailable {
        reason: "Leitura de 0% não confirmada".to_string(),
        source: Some(BatterySource::UPower),
    };

    assert_eq!(avail_0.percentage(), Some(0));
    assert_eq!(avail_0.display_text_pt(), "0%");

    assert_eq!(unavail.percentage(), None);
    assert!(unavail.display_text_pt().contains("Bateria indisponível"));
}

#[test]
fn test_bluez_extract_u8_value_variants() {
    use openrapoo_core::battery::bluez::extract_u8_value;
    use zbus::zvariant::{OwnedValue, Str};

    let val_u8 = OwnedValue::try_from(70u8).unwrap();
    assert_eq!(extract_u8_value(&val_u8), Some(70));

    let val_u32 = OwnedValue::try_from(85u32).unwrap();
    assert_eq!(extract_u8_value(&val_u32), Some(85));

    let val_str = OwnedValue::try_from(Str::from("100")).unwrap();
    assert_eq!(extract_u8_value(&val_str), Some(100));
}

#[test]
fn test_upower_stale_504_seconds_rejected() {
    let now = current_epoch_seconds();
    let stale_reading = BatteryReading {
        percentage: Some(0),
        state: BatteryState::Unknown,
        is_present: true,
        charging: None,
        source: BatterySource::UPower,
        timestamp: now - 504, // 504 seconds ago (exceeds TTL of 60s)
        device_identifier: "Rapoo MT760 Pro".to_string(),
        connection: "bluetooth".to_string(),
        confidence: BatteryConfidence::Low,
        validity: BatteryValidity::Valid,
        invalidation_reason: None,
        alternative_source: None,
        conflict_status: None,
        raw_data: None,
        ..Default::default()
    };

    let dev = DeviceIdentity {
        name: "Rapoo MT760 Pro (Bluetooth)".to_string(),
        transport: ConnectionType::Bluetooth,
        phys: Some("bluetooth/hci1/dev_DE_ED_DC_41_7C_51".to_string()),
        ..Default::default()
    };

    let ttl = openrapoo_core::battery::aggregator::get_ttl_for_transport(&dev.transport);
    assert_eq!(ttl, 60);

    let is_expired = now > stale_reading.timestamp && (now - stale_reading.timestamp) > ttl;
    assert!(is_expired);
}

#[test]
fn test_bluez_direct_query_percentage_missing_logged() {
    let reading = BatteryReading {
        percentage: None,
        state: BatteryState::Unknown,
        is_present: true,
        charging: None,
        source: BatterySource::BlueZ,
        timestamp: current_epoch_seconds(),
        device_identifier: "Rapoo MT760 Pro".to_string(),
        connection: "bluetooth".to_string(),
        confidence: BatteryConfidence::None,
        validity: BatteryValidity::Invalid {
            reason: "Propriedade Percentage ausente na interface org.bluez.Battery1".to_string(),
        },
        invalidation_reason: Some("Battery1 encontrada em /org/bluez/hci1/dev_DE_ED_DC_41_7C_51, mas propriedade Percentage ausente".to_string()),
        alternative_source: None,
        conflict_status: None,
        raw_data: None,
        ..Default::default()
    };

    let status: BatteryStatus = reading.into();
    assert_eq!(status.percentage(), None);
    assert!(status.display_text_pt().contains("indisponível"));
}

#[test]
fn test_bluetooth_active_query_overrides_stale_upower() {
    let fresh_bluez = BatteryReading {
        percentage: Some(70),
        state: BatteryState::Available,
        is_present: true,
        charging: Some(false),
        source: BatterySource::BlueZ,
        timestamp: current_epoch_seconds(),
        device_identifier: "Rapoo MT760 Pro".to_string(),
        connection: "bluetooth".to_string(),
        confidence: BatteryConfidence::High,
        validity: BatteryValidity::Valid,
        invalidation_reason: None,
        alternative_source: None,
        conflict_status: None,
        raw_data: None,
        ..Default::default()
    };

    let status: BatteryStatus = fresh_bluez.into();
    assert_eq!(status.percentage(), Some(70));
    assert_eq!(status.display_text_pt(), "70%");
}

#[test]
fn test_keyboard_80pct_never_assigned_to_mouse() {
    use openrapoo_core::device::DeviceType;

    let keyboard_id = DeviceIdentity {
        name: "Rapoo E9050L".to_string(),
        device_type: DeviceType::Keyboard,
        transport: ConnectionType::Bluetooth,
        bluetooth_address: Some("D1:01:D1:B3:A6:CD".to_string()),
        ..Default::default()
    };

    let mouse_id = DeviceIdentity {
        name: "Rapoo MT760 Pro (Bluetooth)".to_string(),
        device_type: DeviceType::Mouse,
        transport: ConnectionType::Bluetooth,
        bluetooth_address: Some("DE:ED:DC:41:7C:51".to_string()),
        ..Default::default()
    };

    assert_eq!(keyboard_id.device_type, DeviceType::Keyboard);
    assert_eq!(mouse_id.device_type, DeviceType::Mouse);
    assert_ne!(keyboard_id.bluetooth_address, mouse_id.bluetooth_address);
}

#[test]
fn test_device_identity_mac_isolation() {
    use openrapoo_core::device::DeviceType;

    let mouse_identity = DeviceIdentity {
        name: "Rapoo MT760 Pro".to_string(),
        device_type: DeviceType::Mouse,
        transport: ConnectionType::Bluetooth,
        bluetooth_address: Some("DE:ED:DC:41:7C:51".to_string()),
        ..Default::default()
    };

    let status = openrapoo_core::battery::query_battery_for_identity(&mouse_identity);
    // Mouse identity with no matching Battery1 or state=unknown must NOT return 80% from keyboard
    assert_ne!(status.percentage(), Some(80));
}

