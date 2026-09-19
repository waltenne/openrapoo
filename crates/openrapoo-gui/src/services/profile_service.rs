//! Profile service for loading and persisting button profiles for Rapoo devices.

use openrapoo_core::config::{ButtonAction, Profile, ProfileStore};
use std::sync::Mutex;
use tracing::{info, warn};
use uuid::Uuid;

pub struct ProfileService {
    store: Mutex<ProfileStore>,
}

impl Default for ProfileService {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileService {
    pub fn new() -> Self {
        let path = ProfileStore::default_config_path();
        let store = ProfileStore::load_from_file(&path).unwrap_or_else(|e| {
            warn!(
                "Failed to load profile store from {}: {e} — using defaults",
                path.display()
            );
            ProfileStore::default()
        });

        Self {
            store: Mutex::new(store),
        }
    }

    pub fn get_active_profile(&self) -> Profile {
        if let Ok(store) = self.store.lock() {
            store.active_profile().clone()
        } else {
            Profile::default_profile()
        }
    }

    #[allow(dead_code)]
    pub fn get_all_profiles(&self) -> Vec<Profile> {
        if let Ok(store) = self.store.lock() {
            store.profiles.clone()
        } else {
            vec![Profile::default_profile()]
        }
    }

    #[allow(dead_code)]
    pub fn set_active_profile(&self, id: Uuid) -> Result<(), String> {
        if let Ok(mut store) = self.store.lock() {
            if store.set_active(id) {
                let path = ProfileStore::default_config_path();
                store
                    .save_to_file(&path)
                    .map_err(|e| format!("Failed to save profile store: {e}"))?;
                Ok(())
            } else {
                Err("Profile ID not found".to_string())
            }
        } else {
            Err("Failed to acquire lock on profile store".to_string())
        }
    }

    #[allow(dead_code)]
    pub fn create_profile(&self, name: &str) -> Result<Profile, String> {
        if let Ok(mut store) = self.store.lock() {
            let prof = Profile::new(name);
            store.add_profile(prof.clone());
            let path = ProfileStore::default_config_path();
            store
                .save_to_file(&path)
                .map_err(|e| format!("Failed to save profile: {e}"))?;
            Ok(prof)
        } else {
            Err("Failed to acquire lock on profile store".to_string())
        }
    }

    #[allow(dead_code)]
    pub fn duplicate_profile(&self, id: Uuid, name: &str) -> Result<Profile, String> {
        if let Ok(mut store) = self.store.lock() {
            if let Some(dup) = store.duplicate_profile(id, name) {
                let path = ProfileStore::default_config_path();
                store
                    .save_to_file(&path)
                    .map_err(|e| format!("Failed to save profile: {e}"))?;
                Ok(dup)
            } else {
                Err("Profile not found".to_string())
            }
        } else {
            Err("Failed to acquire lock on profile store".to_string())
        }
    }

    #[allow(dead_code)]
    pub fn delete_profile(&self, id: Uuid) -> Result<(), String> {
        if let Ok(mut store) = self.store.lock() {
            if store.delete_profile(id) {
                let path = ProfileStore::default_config_path();
                store
                    .save_to_file(&path)
                    .map_err(|e| format!("Failed to save profile: {e}"))?;
                Ok(())
            } else {
                Err("Cannot delete profile".to_string())
            }
        } else {
            Err("Failed to acquire lock on profile store".to_string())
        }
    }

    pub fn update_action(&self, hex_code: &str, action: ButtonAction) -> Result<(), String> {
        if let Ok(mut store) = self.store.lock() {
            let path = ProfileStore::default_config_path();
            let active_id = store.active_profile_id;
            if let Some(profile) = store.profiles.iter_mut().find(|p| p.id == active_id) {
                profile.mappings.insert(hex_code.to_string(), action);
            }
            store
                .save_to_file(&path)
                .map_err(|e| format!("Failed to save profile: {e}"))?;
            info!("Updated action for button {hex_code}");
            Ok(())
        } else {
            Err("Failed to acquire lock on profile store".to_string())
        }
    }

    pub fn reset_action(&self, hex_code: &str) -> Result<(), String> {
        if let Ok(mut store) = self.store.lock() {
            let path = ProfileStore::default_config_path();
            let active_id = store.active_profile_id;
            if let Some(profile) = store.profiles.iter_mut().find(|p| p.id == active_id) {
                profile.mappings.remove(hex_code);
            }
            store
                .save_to_file(&path)
                .map_err(|e| format!("Failed to save profile: {e}"))?;
            info!("Reset action for button {hex_code}");
            Ok(())
        } else {
            Err("Failed to acquire lock on profile store".to_string())
        }
    }

    pub fn update_dpi(&self, dpi: u32) -> Result<(), String> {
        if let Ok(mut store) = self.store.lock() {
            let path = ProfileStore::default_config_path();
            let active_id = store.active_profile_id;
            if let Some(profile) = store.profiles.iter_mut().find(|p| p.id == active_id) {
                profile.dpi = dpi;
            }
            store
                .save_to_file(&path)
                .map_err(|e| format!("Failed to save profile: {e}"))?;
            info!("Updated DPI setting to {dpi}");
            Ok(())
        } else {
            Err("Failed to acquire lock on profile store".to_string())
        }
    }

    pub fn update_polling_rate(&self, rate: u32) -> Result<(), String> {
        if let Ok(mut store) = self.store.lock() {
            let path = ProfileStore::default_config_path();
            let active_id = store.active_profile_id;
            if let Some(profile) = store.profiles.iter_mut().find(|p| p.id == active_id) {
                profile.polling_rate = rate;
            }
            store
                .save_to_file(&path)
                .map_err(|e| format!("Failed to save profile: {e}"))?;
            info!("Updated polling rate setting to {rate} Hz");
            Ok(())
        } else {
            Err("Failed to acquire lock on profile store".to_string())
        }
    }
}
