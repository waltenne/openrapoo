//! Hotspot definitions and proportional scaling logic for Rapoo MT760 Pro PNG.

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendButton {
    Left,
    Right,
    Middle,
    Forward,
    Back,
    Dpi,
    DeviceSwitch,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotspotState {
    Normal,
    Hover,
    Selected,
    Pressed,
    Unavailable,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlCategory {
    MainButtons,
    SideButtons,
    Wheels,
    HardwareButtons,
    Undetected,
}

use crate::i18n::Language;

impl ControlCategory {
    pub fn label(&self, lang: Language) -> &'static str {
        match lang {
            Language::English => match self {
                ControlCategory::MainButtons => "Main Buttons",
                ControlCategory::SideButtons => "Side Buttons",
                ControlCategory::Wheels => "Wheels & Scroll",
                ControlCategory::HardwareButtons => "Hardware Controls",
                ControlCategory::Undetected => "Undetected Controls",
            },
            Language::Portuguese => match self {
                ControlCategory::MainButtons => "Botões Principais",
                ControlCategory::SideButtons => "Botões Laterais",
                ControlCategory::Wheels => "Rodas & Scroll",
                ControlCategory::HardwareButtons => "Controles de Hardware",
                ControlCategory::Undetected => "Controles Não Detectados",
            },
        }
    }

    #[allow(dead_code)]
    pub fn label_pt(&self) -> &'static str {
        self.label(Language::Portuguese)
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ControlCategory::MainButtons => "🖱️",
            ControlCategory::SideButtons => "👈",
            ControlCategory::Wheels => "🎡",
            ControlCategory::HardwareButtons => "⚙️",
            ControlCategory::Undetected => "🔒",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Hotspot {
    pub id: &'static str,
    pub backend_button: BackendButton,
    pub hex_code: &'static str,
    pub name_pt: &'static str,
    pub name_en: &'static str,
    pub category: ControlCategory,
    /// Center X coordinate normalized (0.0 .. 1.0) relative to image width
    pub norm_cx: f32,
    /// Center Y coordinate normalized (0.0 .. 1.0) relative to image height
    pub norm_cy: f32,
    /// Relative width normalized (0.0 .. 1.0) relative to image width
    pub norm_w: f32,
    /// Relative height normalized (0.0 .. 1.0) relative to image height
    pub norm_h: f32,
    pub side: Side,
    pub is_supported_on_linux: bool,
    pub tooltip_pt: &'static str,
    pub aria_label: &'static str,
}

impl Hotspot {
    pub fn name(&self, lang: Language) -> &'static str {
        match lang {
            Language::English => self.name_en,
            Language::Portuguese => self.name_pt,
        }
    }

    /// Returns the scaled (X, Y) center coordinates relative to rendered image bounding box
    pub fn scaled_center(&self, render_w: f32, render_h: f32) -> (f32, f32) {
        (self.norm_cx * render_w, self.norm_cy * render_h)
    }

    /// Returns the scaled rectangle `(left, top, width, height)` in pixels relative to rendered image bounding box
    #[allow(dead_code)]
    pub fn scaled_rect(&self, render_w: f32, render_h: f32) -> (f32, f32, f32, f32) {
        let w = (self.norm_w * render_w).max(18.0);
        let h = (self.norm_h * render_h).max(18.0);
        let cx = self.norm_cx * render_w;
        let cy = self.norm_cy * render_h;
        (cx - w / 2.0, cy - h / 2.0, w, h)
    }
}

pub const ALL_HOTSPOTS: &[Hotspot] = &[
    Hotspot {
        id: "left-button",
        backend_button: BackendButton::Left,
        hex_code: "0x110",
        name_pt: "Clique Esquerdo",
        name_en: "Left Click",
        category: ControlCategory::MainButtons,
        norm_cx: 0.42,
        norm_cy: 0.22,
        norm_w: 0.12,
        norm_h: 0.12,
        side: Side::Left,
        is_supported_on_linux: true,
        tooltip_pt: "Botão principal de clique esquerdo (0x110)",
        aria_label: "Botão principal esquerdo do mouse",
    },
    Hotspot {
        id: "right-button",
        backend_button: BackendButton::Right,
        hex_code: "0x111",
        name_pt: "Clique Direito",
        name_en: "Right Click",
        category: ControlCategory::MainButtons,
        norm_cx: 0.72,
        norm_cy: 0.26,
        norm_w: 0.12,
        norm_h: 0.12,
        side: Side::Right,
        is_supported_on_linux: true,
        tooltip_pt: "Botão secundário de clique direito (0x111)",
        aria_label: "Botão secundário direito do mouse",
    },
    Hotspot {
        id: "main-wheel",
        backend_button: BackendButton::WheelUp,
        hex_code: "SCROLL_UP",
        name_pt: "Roda Principal (Scroll)",
        name_en: "Main Scroll Wheel",
        category: ControlCategory::Wheels,
        norm_cx: 0.56,
        norm_cy: 0.11,
        norm_w: 0.06,
        norm_h: 0.08,
        side: Side::Right,
        is_supported_on_linux: true,
        tooltip_pt: "Roda de rolagem vertical principal",
        aria_label: "Roda de rolagem principal",
    },
    Hotspot {
        id: "middle-click",
        backend_button: BackendButton::Middle,
        hex_code: "0x112",
        name_pt: "Clique Central (MB3)",
        name_en: "Middle Click",
        category: ControlCategory::MainButtons,
        norm_cx: 0.58,
        norm_cy: 0.16,
        norm_w: 0.06,
        norm_h: 0.06,
        side: Side::Right,
        is_supported_on_linux: true,
        tooltip_pt: "Clique do botão central da roda (0x112 / MB3)",
        aria_label: "Botão do clique central da roda",
    },
    Hotspot {
        id: "dpi-button",
        backend_button: BackendButton::Dpi,
        hex_code: "0x117",
        name_pt: "Botão DPI",
        name_en: "DPI Switch",
        category: ControlCategory::HardwareButtons,
        norm_cx: 0.73,
        norm_cy: 0.33,
        norm_w: 0.06,
        norm_h: 0.05,
        side: Side::Right,
        is_supported_on_linux: true,
        tooltip_pt: "Botão de alternância de DPI de hardware (0x117)",
        aria_label: "Botão de alternância de DPI de hardware",
    },
    Hotspot {
        id: "thumb-wheel",
        backend_button: BackendButton::WheelLeft,
        hex_code: "SCROLL_LEFT",
        name_pt: "Roda de Polegar",
        name_en: "Thumb Wheel",
        category: ControlCategory::Wheels,
        norm_cx: 0.48,
        norm_cy: 0.54,
        norm_w: 0.08,
        norm_h: 0.08,
        side: Side::Left,
        is_supported_on_linux: true,
        tooltip_pt: "Roda de rolagem lateral de polegar",
        aria_label: "Roda de rolagem de polegar lateral",
    },
    Hotspot {
        id: "forward-button",
        backend_button: BackendButton::Forward,
        hex_code: "0x114",
        name_pt: "Avançar (MB5)",
        name_en: "Forward",
        category: ControlCategory::SideButtons,
        norm_cx: 0.37,
        norm_cy: 0.52,
        norm_w: 0.06,
        norm_h: 0.05,
        side: Side::Left,
        is_supported_on_linux: true,
        tooltip_pt: "Botão lateral de navegação avançar (0x114 / MB5)",
        aria_label: "Botão lateral de navegação avançar",
    },
    Hotspot {
        id: "back-button",
        backend_button: BackendButton::Back,
        hex_code: "0x113",
        name_pt: "Voltar (MB4)",
        name_en: "Back",
        category: ControlCategory::SideButtons,
        norm_cx: 0.35,
        norm_cy: 0.59,
        norm_w: 0.06,
        norm_h: 0.05,
        side: Side::Left,
        is_supported_on_linux: true,
        tooltip_pt: "Botão lateral de navegação voltar (0x113 / MB4)",
        aria_label: "Botão lateral de navegação voltar",
    },
    Hotspot {
        id: "device-switch",
        backend_button: BackendButton::DeviceSwitch,
        hex_code: "0x118",
        name_pt: "Troca de Dispositivo",
        name_en: "Device Switch",
        category: ControlCategory::Undetected,
        norm_cx: 0.65,
        norm_cy: 0.68,
        norm_w: 0.06,
        norm_h: 0.05,
        side: Side::Right,
        is_supported_on_linux: false,
        tooltip_pt: "Este controle não envia eventos ao sistema operacional neste modo.",
        aria_label: "Botão de alternância de canais Bluetooth / 2.4 GHz",
    },
];
