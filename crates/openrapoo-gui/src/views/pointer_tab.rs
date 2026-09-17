//! "Ponteiro" (Pointer) Tab View Component.

#[derive(Debug, Clone)]
pub struct PointerTabSettings {
    pub pointer_speed: f64,
    pub pointer_acceleration: bool,
    pub natural_scroll: bool,
    pub scroll_speed: f64,

    // Hardware features not supported by generic Linux HID driver
    pub dpi_preset: u32,
    pub dpi_supported: bool,
    pub dpi_unsupported_reason_pt: &'static str,
    pub polling_rate_hz: u32,
    pub polling_rate_supported: bool,
    pub polling_rate_unsupported_reason_pt: &'static str,
}

impl Default for PointerTabSettings {
    fn default() -> Self {
        PointerTabSettings {
            pointer_speed: 1.0,
            pointer_acceleration: true,
            natural_scroll: false,
            scroll_speed: 1.0,
            dpi_preset: 1600,
            dpi_supported: false,
            dpi_unsupported_reason_pt: "Ajuste de DPI via software requer protocolo HID proprietário não documentado para Linux. O botão físico DPI alterna as velocidades gravadas no firmware do mouse.",
            polling_rate_hz: 1000,
            polling_rate_supported: false,
            polling_rate_unsupported_reason_pt: "Polling rate é fixado pelo firmware do mouse e pelo driver USB/NearLink do kernel Linux.",
        }
    }
}

