//! Device info & UDev permissions tab page.

use crate::device_model::{DeviceUiExt, RapooDevice};
use crate::i18n::{self, Language};
use crate::theme::current_theme;
use gpui::{div, px, IntoElement, ParentElement, Styled, StyledImage};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex};
use openrapoo_core::battery::BatteryStatus;
use openrapoo_core::permissions::check_input_group_status;

pub fn render_device_info_page(device: Option<&RapooDevice>, language: Language) -> impl IntoElement {
    let theme = current_theme();

    use openrapoo_core::device::DeviceType;

    let is_en = matches!(language, Language::English);

    let dev_name = device.map(|d| d.name.clone()).unwrap_or_else(|| "Rapoo MT760 Pro".to_string());
    let is_keyboard = device
        .map(|d| d.device_type == DeviceType::Keyboard)
        .unwrap_or(false);
    let vid = device.map(|d| d.vendor_id).unwrap_or(0x24AE);
    let pid = device.map(|d| d.product_id).unwrap_or(0x186A);
    let conn = device
        .map(|d| d.connection_badge(language))
        .unwrap_or_else(|| i18n::conn_24ghz(language).to_string());
    let fw_version = device
        .map(|d| d.firmware_version().to_string())
        .unwrap_or_else(|| {
            if is_en {
                "v1.0.4 (Default Firmware)".to_string()
            } else {
                "v1.0.4 (Firmware Padrão)".to_string()
            }
        });
    let evdev = device
        .and_then(|d| d.evdev_path.as_ref())
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "/dev/input/event5".to_string());
    let hidraw = device
        .and_then(|d| d.hidraw_path.as_ref())
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "/dev/hidraw2".to_string());

    let group_status = check_input_group_status();

    let hero_image = if is_keyboard {
        if let Some(png_path) = crate::assets::get_e9050l_png_path() {
            div()
                .w_full()
                .h(px(140.0))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    gpui::img(png_path)
                        .w(px(320.0))
                        .h(px(130.0))
                        .object_fit(gpui::ObjectFit::Contain),
                )
        } else {
            div()
                .w_full()
                .h(px(90.0))
                .flex()
                .items_center()
                .justify_center()
                .child(div().text_size(px(40.0)).child("⌨️"))
        }
    } else if let Some(png_path) = crate::assets::get_mt760_pro_png_path() {
        div()
            .w_full()
            .h(px(140.0))
            .flex()
            .items_center()
            .justify_center()
            .child(
                gpui::img(png_path)
                    .w(px(140.0))
                    .h(px(130.0))
                    .object_fit(gpui::ObjectFit::Contain),
            )
    } else {
        div()
            .w_full()
            .h(px(90.0))
            .flex()
            .items_center()
            .justify_center()
            .child(div().text_size(px(40.0)).child("🖱️"))
    };

    let battery_summary = device.and_then(|d| match &d.battery_status {
        BatteryStatus::Available { percentage, charging, .. } => {
            if *charging {
                Some(format!("{percentage}% (⚡)"))
            } else {
                Some(format!("{percentage}%"))
            }
        }
        BatteryStatus::Charging { percentage: Some(pct), .. } => Some(format!("{pct}% (⚡)")),
        BatteryStatus::Full => Some(if is_en { "100% (Full) 🟢".to_string() } else { "100% (Completa) 🟢".to_string() }),
        BatteryStatus::Discharging { percentage } => Some(format!("{percentage}%")),
        _ => None,
    }).unwrap_or_else(|| if is_en { "Unavailable / Not exposed".to_string() } else { "Indisponível / Não exposta".to_string() });

    let specs_content = if is_keyboard {
        v_flex()
            .flex_1()
            .justify_between()
            .gap_1p5()
            .child(info_row(if is_en { "Firmware Version" } else { "Versão do Firmware" }, &fw_version))
            .child(info_row(if is_en { "Keyboard Type" } else { "Tipo de Teclado" }, if is_en { "E9050L Ultra-Slim Keyboard" } else { "Teclado Ultra-Fino E9050L" }))
            .child(info_row(if is_en { "Remapping Software" } else { "Software de Remapeamento" }, "Standard Input"))
            .child(info_row(if is_en { "Key Layout" } else { "Mapeamento de Teclas" }, "Chiclet Slim (ABNT2 / ANSI)"))
            .child(info_row(if is_en { "Media Shortcuts" } else { "Atalhos de Mídia" }, "Fn + F1-F12 · 2.4G / BT1 / BT2"))
    } else {
        v_flex()
            .flex_1()
            .justify_between()
            .gap_1p5()
            .child(info_row(if is_en { "Firmware Version" } else { "Versão do Firmware" }, &fw_version))
            .child(info_row(if is_en { "Optical Sensor" } else { "Sensor Óptico" }, "PixArt PAW3395 (4000 DPI)"))
            .child(info_row(if is_en { "Controller / MCU" } else { "Controlador / MCU" }, "ITON Rapoo Dual-Mode"))
            .child(info_row(if is_en { "Hardware Revision" } else { "Revisão de Hardware" }, "Rev 2.0 (2.4GHz + BT 5.0)"))
            .child(info_row(if is_en { "Polling Rate" } else { "Taxa de Amostragem" }, "1000 Hz (1ms)"))
    };

    let ident_content = v_flex()
        .flex_1()
        .justify_between()
        .gap_1p5()
        .child(info_row(if is_en { "Model Name" } else { "Nome do Modelo" }, &dev_name))
        .child(info_row(if is_en { "Manufacturer" } else { "Fabricante" }, "ITON Corp. / Rapoo"))
        .child(info_row("Vendor ID (VID)", &format!("0x{vid:04X}")))
        .child(info_row("Product ID (PID)", &format!("0x{pid:04X}")));

    let conn_status_badge = if device.map(|d| d.is_connected()).unwrap_or(true) {
        if is_en { "Connected 🟢" } else { "Conectado 🟢" }
    } else {
        if is_en { "Disconnected 🔴" } else { "Desconectado 🔴" }
    };

    v_flex()
        .flex_1()
        .w_full()
        .p_6()
        .gap_6()
        .overflow_y_scrollbar()
        .child(
            v_flex()
                .gap_1()
                .items_center()
                .w_full()
                .child(
                    div()
                        .text_size(px(20.0))
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.text_primary)
                        .child(if is_en { "Device Information" } else { "Informações do Dispositivo" }),
                )
                .child(
                    div()
                        .text_size(px(13.0))
                        .text_color(theme.text_muted)
                        .child(if is_en { "Linux system hardware, identification, and permissions summary." } else { "Resumo de hardware, identificação e permissões do sistema Linux." }),
                ),
        )
        .child(
            // Summary Card (Centralized Content)
            h_flex()
                .w_full()
                .p_6()
                .rounded_xl()
                .bg(theme.panel)
                .border_1()
                .border_color(theme.border)
                .justify_center()
                .items_center()
                .gap_8()
                .child(div().w(px(220.0)).flex().justify_center().child(hero_image))
                .child(
                    v_flex()
                        .gap_2p5()
                        .justify_center()
                        .child(
                            div()
                                .text_size(px(22.0))
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child(dev_name.clone()),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(
                                    div()
                                        .px_2p5()
                                        .py_1()
                                        .rounded_md()
                                        .bg(theme.panel_elevated)
                                        .text_color(theme.accent_blue)
                                        .text_size(px(12.0))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .child(conn),
                                )
                                .child(
                                    div()
                                        .px_2p5()
                                        .py_1()
                                        .rounded_md()
                                        .bg(theme.panel_elevated)
                                        .text_color(theme.accent_green)
                                        .text_size(px(12.0))
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .child(conn_status_badge),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(theme.text_muted)
                                .child(format!("{}: {battery_summary}", if is_en { "Battery" } else { "Bateria" })),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(theme.text_muted)
                                .child(format!("Firmware: {fw_version}")),
                        ),
                ),
        )
        .child(
            // 2-Column Row for Identificação & Especificações (Equal Heights)
            h_flex()
                .w_full()
                .gap_6()
                .items_stretch()
                .child(
                    // Left Card: Identificação de Hardware
                    v_flex()
                        .flex_1()
                        .h_full()
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
                                .child(if is_en { "Hardware Identification" } else { "Identificação de Hardware" }),
                        )
                        .child(ident_content),
                )
                .child(
                    // Right Card: Especificações de Hardware
                    v_flex()
                        .flex_1()
                        .h_full()
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
                                .child(if is_en { "Hardware Specifications" } else { "Especificações de Hardware" }),
                        )
                        .child(specs_content),
                ),
        )
        .child(
            // System Integration
            v_flex()
                .w_full()
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
                        .child(if is_en { "Linux System Integration" } else { "Integração com o Sistema Linux" }),
                )
                .child(info_row(if is_en { "`input` Group" } else { "Grupo `input`" }, if is_en { group_status.display_message_en() } else { group_status.display_message_pt() }))
                .child(info_row(if is_en { "UDev Rule (99-openrapoo.rules)" } else { "Regra UDev (99-openrapoo.rules)" }, if is_en { "Installed 🟢" } else { "Instalada 🟢" }))
                .child(info_row(if is_en { "Evdev Node" } else { "Nó Evdev" }, &evdev))
                .child(info_row(if is_en { "HIDRAW Node" } else { "Nó HIDRAW" }, &hidraw)),
        )
        .child(div().h(px(32.0)))
}

fn info_row(label: &str, value: &str) -> impl IntoElement {
    let theme = current_theme();

    h_flex()
        .w_full()
        .justify_between()
        .items_center()
        .gap_3()
        .py_2()
        .border_b_1()
        .border_color(theme.border)
        .child(
            div()
                .flex_shrink_0()
                .max_w(px(260.0))
                .text_size(px(13.0))
                .text_color(theme.text_muted)
                .child(label.to_string()),
        )
        .child(
            div()
                .flex_1()
                .text_size(px(13.0))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme.text_primary)
                .text_align(gpui::TextAlign::Right)
                .child(value.to_string()),
        )
}
