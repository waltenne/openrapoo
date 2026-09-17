#![allow(dead_code)]
//! Diagnostics view model for OpenRapoo GUI.

use openrapoo_core::device::{detect_rapoo_devices, RapooDevice};

pub struct DiagViewState {
    pub devices: Vec<RapooDevice>,
}

impl Default for DiagViewState {
    fn default() -> Self {
        DiagViewState {
            devices: detect_rapoo_devices().unwrap_or_default(),
        }
    }
}
