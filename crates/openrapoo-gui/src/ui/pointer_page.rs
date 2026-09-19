//! Pointer & DPI settings tab page for Rapoo MT760 Pro.

use crate::device_model::RapooDevice;
use crate::i18n::{self, Language};
use crate::theme::current_theme;
use gpui::{
    div, px, ElementId, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled,
};
use gpui_component::{h_flex, v_flex};
use openrapoo_core::config::Profile;
use openrapoo_core::device::ConnectionType;
use std::rc::Rc;

pub fn render_pointer_page(
    device: &RapooDevice,
    active_profile: &Profile,
    language: Language,
    on_update_dpi: Rc<dyn Fn(u32, &mut gpui::App)>,
    on_update_polling_rate: Rc<dyn Fn(u32, &mut gpui::App)>,
    on_apply_hardware: Rc<dyn Fn(&mut gpui::App)>,
    on_restore_hardware: Rc<dyn Fn(&mut gpui::App)>,
    on_read_hardware: Rc<dyn Fn(&mut gpui::App)>,
) -> impl IntoElement {
    let theme = current_theme();

    let is_bluetooth = device.connection == ConnectionType::Bluetooth;
    let conn_str = match device.connection {
        ConnectionType::UsbCable | ConnectionType::UsbWired => match language {
            Language::English => "USB Cable",
            Language::Portuguese => "Cabo USB",
        },
        ConnectionType::TwoPointFourGhz => i18n::conn_24ghz(language),
        ConnectionType::Bluetooth => i18n::conn_bluetooth(language),
        ConnectionType::NearLink => "NearLink",
        ConnectionType::Dock => match language {
            Language::English => "Charging Dock",
            Language::Portuguese => "Base de Carga",
        },
        ConnectionType::Disconnected | ConnectionType::Unknown => i18n::conn_disconnected(language),
    }
    .to_string();

    let dpi_levels = [800u32, 1000, 1200, 1600, 2400, 3200, 4000];
    let hz_levels = [
        (125u32, "125 Hz"),
        (250, "250 Hz"),
        (500, "500 Hz"),
        (1000, "1000 Hz"),
    ];

    let mut dpi_row = h_flex().gap_2().flex_wrap();
    for (idx, dpi_val) in dpi_levels.iter().cloned().enumerate() {
        let is_active = active_profile.dpi == dpi_val;
        let cb = on_update_dpi.clone();
        dpi_row = dpi_row.child(dpi_chip(idx, dpi_val, is_active, move |_, _, cx| {
            cb(dpi_val, cx)
        }));
    }

    let mut hz_row = h_flex().gap_3().flex_wrap();
    for (idx, (hz_val, label)) in hz_levels.iter().cloned().enumerate() {
        let is_active = active_profile.polling_rate == hz_val;
        let cb = on_update_polling_rate.clone();
        hz_row = hz_row.child(hz_chip(idx, hz_val, label, is_active, move |_, _, cx| {
            cb(hz_val, cx)
        }));
    }

    // Hardware status badge determination
    let (status_bg, status_fg, status_border, status_text) = if is_bluetooth {
        (
            theme.panel_elevated,
            theme.warning,
            theme.warning,
            match language {
                Language::English => "⚠️ Not supported in Bluetooth mode",
                Language::Portuguese => "⚠️ Não suportado no modo Bluetooth",
            },
        )
    } else {
        (
            theme.panel_elevated,
            theme.accent_green,
            theme.accent_green,
            match language {
                Language::English => "🟢 Confirmed on mouse",
                Language::Portuguese => "🟢 Confirmado no mouse",
            },
        )
    };

    let subtitle = match language {
        Language::English => format!(
            "Sensitivity and polling rate settings for profile '{}'",
            active_profile.name
        ),
        Language::Portuguese => format!(
            "Ajustes de sensibilidade e amostragem do perfil '{}'",
            active_profile.name
        ),
    };

    let bt_warning_text = match language {
        Language::English => "⚠️ Bluetooth mode polling rate is managed by the Linux kernel (~90-133 Hz). For hardware EEPROM customization, connect via USB Cable or 2.4G Receiver.",
        Language::Portuguese => "⚠️ O modo Bluetooth gerencia a taxa de amostragem pelo kernel Linux (~90-133 Hz). Para gravar configurações diretamente na EEPROM do mouse, conecte por Cabo USB ou Dongle 2.4G.",
    };

    let dpi_title = match language {
        Language::English => "DPI Levels (Resolution)",
        Language::Portuguese => "Níveis de DPI (Resolução)",
    };

    let dpi_desc = match language {
        Language::English => "Select desired resolution. Active level is saved in profile.",
        Language::Portuguese => "Selecione a resolução desejada. Nível ativo é salvo no perfil.",
    };

    let hz_title = match language {
        Language::English => "Polling Rate (Sampling Frequency)",
        Language::Portuguese => "Taxa de Amostragem (Polling Rate)",
    };

    let hz_desc = match language {
        Language::English => "USB polling frequency (Recommended default: 1000 Hz).",
        Language::Portuguese => "Frequência de atualização USB (Padrão recomendado: 1000 Hz).",
    };

    let action_title = match language {
        Language::English => "Hardware Actions (/dev/hidraw ioctl report 0x07)",
        Language::Portuguese => "Ações do Hardware (/dev/hidraw ioctl report 0x07)",
    };

    let mut root = v_flex().flex_1().w_full().p_5().gap_4().child(
        // Compact Header + Active Summary Row + Status Badge
        h_flex()
            .w_full()
            .p_4()
            .rounded_xl()
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border)
            .justify_between()
            .items_center()
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_size(px(18.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child(i18n::pointer_page_title(language)),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(theme.text_muted)
                            .child(subtitle),
                    ),
            )
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_lg()
                            .bg(theme.panel_elevated)
                            .border_1()
                            .border_color(theme.accent_blue)
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(theme.text_muted)
                                            .child("DPI:"),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(14.0))
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(theme.accent_blue)
                                            .child(format!("{} DPI", active_profile.dpi)),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_lg()
                            .bg(theme.panel_elevated)
                            .border_1()
                            .border_color(theme.accent_green)
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(theme.text_muted)
                                            .child("Polling:"),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(14.0))
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(theme.accent_green)
                                            .child(format!("{} Hz", active_profile.polling_rate)),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_lg()
                            .bg(status_bg)
                            .border_1()
                            .border_color(status_border)
                            .text_size(px(12.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(status_fg)
                            .child(status_text),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_lg()
                            .bg(theme.panel_elevated)
                            .text_size(px(12.0))
                            .text_color(theme.text_muted)
                            .child(conn_str),
                    ),
            ),
    );

    if is_bluetooth {
        root = root.child(
            h_flex()
                .w_full()
                .p_3()
                .rounded_xl()
                .bg(theme.panel_elevated)
                .border_1()
                .border_color(theme.warning)
                .gap_3()
                .items_center()
                .child(
                    div()
                        .text_size(px(13.0))
                        .text_color(theme.warning)
                        .child(bt_warning_text),
                ),
        );
    }

    root = root.child(
        // 2-Column Controls (DPI Levels on Left, Polling Rate on Right)
        h_flex()
            .w_full()
            .gap_4()
            .items_stretch()
            .child(
                // Left Card: DPI Levels
                v_flex()
                    .flex_1()
                    .p_5()
                    .rounded_xl()
                    .bg(theme.panel)
                    .border_1()
                    .border_color(theme.border)
                    .gap_3()
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child(dpi_title),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(theme.text_muted)
                            .child(dpi_desc),
                    )
                    .child(dpi_row),
            )
            .child(
                // Right Card: Polling Rate
                v_flex()
                    .flex_1()
                    .p_5()
                    .rounded_xl()
                    .bg(theme.panel)
                    .border_1()
                    .border_color(theme.border)
                    .gap_3()
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme.text_primary)
                            .child(hz_title),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(theme.text_muted)
                            .child(hz_desc),
                    )
                    .child(hz_row),
            ),
    );

    let cb_apply = on_apply_hardware.clone();
    let cb_restore = on_restore_hardware.clone();
    let cb_read = on_read_hardware.clone();

    root.child(
        // Action Toolbar: Hardware Transaction Controls
        h_flex()
            .w_full()
            .p_4()
            .rounded_xl()
            .bg(theme.panel)
            .border_1()
            .border_color(theme.border)
            .justify_between()
            .items_center()
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(theme.text_muted)
                    .child(action_title),
            )
            .child(
                h_flex()
                    .gap_3()
                    .child(
                        div()
                            .id(ElementId::from(701))
                            .px_4()
                            .py_2()
                            .rounded_lg()
                            .bg(if is_bluetooth {
                                theme.panel_elevated
                            } else {
                                theme.accent_blue
                            })
                            .text_color(if is_bluetooth {
                                theme.text_muted
                            } else {
                                theme.text_primary
                            })
                            .text_size(px(13.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .cursor_pointer()
                            .on_click(move |_, _, cx| cb_apply(cx))
                            .child(i18n::apply_to_mouse(language)),
                    )
                    .child(
                        div()
                            .id(ElementId::from(702))
                            .px_4()
                            .py_2()
                            .rounded_lg()
                            .bg(theme.panel_elevated)
                            .text_color(theme.text_primary)
                            .text_size(px(13.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .cursor_pointer()
                            .on_click(move |_, _, cx| cb_restore(cx))
                            .child(i18n::restore_original(language)),
                    )
                    .child(
                        div()
                            .id(ElementId::from(703))
                            .px_4()
                            .py_2()
                            .rounded_lg()
                            .bg(theme.panel_elevated)
                            .text_color(theme.text_primary)
                            .text_size(px(13.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .cursor_pointer()
                            .on_click(move |_, _, cx| cb_read(cx))
                            .child(i18n::read_from_mouse(language)),
                    ),
            ),
    )
}

fn dpi_chip(
    idx: usize,
    dpi_val: u32,
    is_active: bool,
    on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    let theme = current_theme();
    let bg = if is_active {
        theme.accent_blue
    } else {
        theme.panel_elevated
    };
    let fg = if is_active {
        theme.text_primary
    } else {
        theme.text_muted
    };

    div()
        .id(ElementId::from(idx + 800))
        .px_3p5()
        .py_2()
        .rounded_lg()
        .bg(bg)
        .text_color(fg)
        .text_size(px(13.0))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .cursor_pointer()
        .hover(move |s| {
            if is_active {
                s
            } else {
                s.bg(theme.border).text_color(theme.text_primary)
            }
        })
        .on_click(on_click)
        .child(format!("{dpi_val} DPI"))
}

fn hz_chip(
    idx: usize,
    _hz_val: u32,
    label: &str,
    is_active: bool,
    on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) -> impl IntoElement {
    let theme = current_theme();
    let bg = if is_active {
        theme.accent_green
    } else {
        theme.panel_elevated
    };
    let fg = if is_active {
        theme.panel
    } else {
        theme.text_muted
    };

    div()
        .id(ElementId::from(idx + 900))
        .px_4()
        .py_2()
        .rounded_lg()
        .bg(bg)
        .text_color(fg)
        .text_size(px(13.0))
        .font_weight(gpui::FontWeight::BOLD)
        .cursor_pointer()
        .hover(move |s| {
            if is_active {
                s
            } else {
                s.bg(theme.border).text_color(theme.text_primary)
            }
        })
        .on_click(on_click)
        .child(label.to_string())
}
