#![allow(dead_code)]
//! Internationalization (i18n) module for OpenRapoo GUI.
//! Supports Portuguese (pt-BR) and English (en-US).

use std::sync::atomic::{AtomicBool, Ordering};

static IS_PORTUGUESE: AtomicBool = AtomicBool::new(true);

/// Set global language preference.
pub fn set_language_pt_br(enable: bool) {
    IS_PORTUGUESE.store(enable, Ordering::Relaxed);
}

/// Returns true if language is Portuguese (pt-BR).
pub fn is_pt_br() -> bool {
    IS_PORTUGUESE.load(Ordering::Relaxed)
}

/// Translate text based on current language setting.
pub fn tr(pt: &'static str, en: &'static str) -> &'static str {
    if is_pt_br() {
        pt
    } else {
        en
    }
}
