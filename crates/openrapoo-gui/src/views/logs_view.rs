#![allow(dead_code)]
//! Log viewer component model for OpenRapoo GUI.

use std::path::PathBuf;

pub struct LogsViewState {
    pub log_dir: PathBuf,
}

impl Default for LogsViewState {
    fn default() -> Self {
        let log_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("openrapoo");
        LogsViewState { log_dir }
    }
}
