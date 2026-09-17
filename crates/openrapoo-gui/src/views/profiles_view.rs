#![allow(dead_code)]
//! Profiles manager view for OpenRapoo GUI.

use openrapoo_core::config::ProfileStore;
use std::path::PathBuf;

pub struct ProfilesViewState {
    pub store: ProfileStore,
    pub config_file: PathBuf,
}

impl Default for ProfilesViewState {
    fn default() -> Self {
        let config_file = ProfileStore::default_config_path();
        let store = ProfileStore::load_from_file(&config_file).unwrap_or_default();
        ProfilesViewState { store, config_file }
    }
}

impl ProfilesViewState {
    pub fn reload(&mut self) {
        if let Ok(store) = ProfileStore::load_from_file(&self.config_file) {
            self.store = store;
        }
    }

    pub fn save(&self) -> Result<(), openrapoo_core::error::OpenRapooError> {
        self.store.save_to_file(&self.config_file)
    }
}
