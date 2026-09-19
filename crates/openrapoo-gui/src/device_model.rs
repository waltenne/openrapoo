//! UI data models and helpers wrapping openrapoo_core device primitives.

pub use openrapoo_core::battery::BatteryStatus;
pub use openrapoo_core::device::{
    ConnectionType, DeviceCapabilities, DeviceConnectionState, DeviceType, KnownDevice, RapooDevice,
};

use crate::i18n::{self, Language};

/// UI helper extension methods for RapooDevice
pub trait DeviceUiExt {
    #[allow(dead_code)]
    fn battery_label_pt(&self) -> String;
    #[allow(dead_code)]
    fn connection_badge_pt(&self) -> String;
    fn connection_badge(&self, lang: Language) -> String;
    #[allow(dead_code)]
    fn type_badge_pt(&self) -> &'static str;
    fn type_badge(&self, lang: Language) -> &'static str;
    fn is_connected(&self) -> bool;
}

impl DeviceUiExt for RapooDevice {
    fn battery_label_pt(&self) -> String {
        self.battery_status.display_text_pt()
    }

    fn connection_badge_pt(&self) -> String {
        self.connection_badge(Language::Portuguese)
    }

    fn connection_badge(&self, lang: Language) -> String {
        match self.connection {
            ConnectionType::UsbCable | ConnectionType::UsbWired => match lang {
                Language::English => "USB Cable".to_string(),
                Language::Portuguese => "Cabo USB".to_string(),
            },
            ConnectionType::TwoPointFourGhz => i18n::conn_24ghz(lang).to_string(),
            ConnectionType::Bluetooth => i18n::conn_bluetooth(lang).to_string(),
            ConnectionType::NearLink => "NearLink".to_string(),
            ConnectionType::Dock => match lang {
                Language::English => "Charging Dock".to_string(),
                Language::Portuguese => "Base de Carga".to_string(),
            },
            ConnectionType::Disconnected | ConnectionType::Unknown => {
                i18n::conn_disconnected(lang).to_string()
            }
        }
    }

    fn type_badge_pt(&self) -> &'static str {
        self.type_badge(Language::Portuguese)
    }

    fn type_badge(&self, lang: Language) -> &'static str {
        match self.device_type {
            DeviceType::Mouse => "Mouse",
            DeviceType::Keyboard => match lang {
                Language::English => "Keyboard",
                Language::Portuguese => "Teclado",
            },
            DeviceType::Other => match lang {
                Language::English => "Other",
                Language::Portuguese => "Outro",
            },
        }
    }

    fn is_connected(&self) -> bool {
        self.connection_state == DeviceConnectionState::Connected
    }
}
