//! Centralized theme palette and visual constants for OpenRapoo GPUI.

use gpui::{hsla, Hsla};

// Refined dark palette with zinc tones and subtle elevation
pub const BG_DARK: Hsla = hsla(0.67, 0.08, 0.09, 1.0); // #141417 (Zinc 950)
pub const PANEL_BG: Hsla = hsla(0.67, 0.07, 0.12, 1.0); // #1C1C21 (Zinc 900)
pub const PANEL_ELEVATED: Hsla = hsla(0.67, 0.06, 0.17, 1.0); // #27272A (Zinc 800)
pub const PANEL_BORDER: Hsla = hsla(0.67, 0.05, 0.24, 1.0); // #3F3F46 (Zinc 700)

pub const TEXT_PRIMARY: Hsla = hsla(0.60, 0.15, 0.95, 1.0); // #FAFAFA
pub const TEXT_MUTED: Hsla = hsla(0.60, 0.08, 0.65, 1.0); // #A1A1AA

pub const ACCENT_GREEN: Hsla = hsla(0.38, 0.70, 0.50, 1.0); // #22C55E
pub const ACCENT_BLUE: Hsla = hsla(0.60, 0.90, 0.60, 1.0); // #3B82F6
pub const COLOR_WARNING: Hsla = hsla(0.11, 0.90, 0.55, 1.0); // #F59E0B
pub const COLOR_ERROR: Hsla = hsla(0.0, 0.85, 0.60, 1.0); // #EF4444
pub const LINE_INACTIVE: Hsla = hsla(0.60, 0.15, 0.40, 0.3); // Soft muted line

pub const HEADER_HEIGHT: f32 = 54.0;
pub const STATUS_BAR_HEIGHT: f32 = 32.0;

#[derive(Clone, Copy, Debug)]
pub struct ThemePalette {
    pub bg: Hsla,
    pub panel: Hsla,
    pub panel_elevated: Hsla,
    pub border: Hsla,
    pub text_primary: Hsla,
    pub text_muted: Hsla,
    pub accent_green: Hsla,
    pub accent_blue: Hsla,
    pub warning: Hsla,
    pub error: Hsla,
    #[allow(dead_code)]
    pub line_inactive: Hsla,
}

pub fn current_theme() -> ThemePalette {
    ThemePalette {
        bg: BG_DARK,
        panel: PANEL_BG,
        panel_elevated: PANEL_ELEVATED,
        border: PANEL_BORDER,
        text_primary: TEXT_PRIMARY,
        text_muted: TEXT_MUTED,
        accent_green: ACCENT_GREEN,
        accent_blue: ACCENT_BLUE,
        warning: COLOR_WARNING,
        error: COLOR_ERROR,
        line_inactive: LINE_INACTIVE,
    }
}
