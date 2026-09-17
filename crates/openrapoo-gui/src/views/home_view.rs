#![allow(dead_code)]
use openrapoo_core::device::{detect_rapoo_devices, RapooDevice};

#[derive(Clone, Debug)]
pub struct HomeViewState {
    pub detected_device: Option<RapooDevice>,
    pub status_message: String,
    pub active_remapping: bool,
}

impl Default for HomeViewState {
    fn default() -> Self {
        let devices = detect_rapoo_devices().unwrap_or_default();
        let first_device = devices.into_iter().find(|d| d.is_mt760_pro())
            .or_else(|| detect_rapoo_devices().unwrap_or_default().into_iter().next());

        let (status, device) = match first_device {
            Some(dev) => (
                format!("Conectado: {} ({})", dev.name, dev.connection),
                Some(dev),
            ),
            None => (
                "Nenhum dispositivo Rapoo detectado".to_string(),
                None,
            ),
        };

        HomeViewState {
            detected_device: device,
            status_message: status,
            active_remapping: true,
        }
    }
}
