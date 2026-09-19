//! Battery Diagnostic section for the device detail view.

use crate::i18n::{self, Language};
use crate::theme::current_theme;
use gpui::{
    div, px, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex};
use openrapoo_core::battery::{BatteryDiagnosticReport, BatteryStatus};
use openrapoo_core::device::RapooDevice;
use std::rc::Rc;

pub fn render_battery_diag_tab(
    device: &RapooDevice,
    _report: &BatteryDiagnosticReport,
    language: Language,
    on_refresh_battery: impl Fn(&mut gpui::App) + 'static,
    on_open_modal: Rc<dyn Fn(&mut gpui::App)>,
) -> impl IntoElement {
    let theme = current_theme();
    let is_en = matches!(language, Language::English);

    let cb_modal = on_open_modal;

    let (badge_bg, badge_text, badge_fg) = match &device.battery_status {
        BatteryStatus::Available { percentage, charging, .. } => {
            if *charging {
                (theme.accent_green, if is_en { format!("{percentage}% · Charging ⚡") } else { format!("{percentage}% · Carregando ⚡") }, theme.panel)
            } else {
                (theme.accent_green, format!("{percentage}% 🟢"), theme.panel)
            }
        }
        BatteryStatus::Charging { percentage, source } => {
            let pct_str = percentage.map(|p| format!("{p}% · ")).unwrap_or_default();
            (theme.accent_blue, if is_en { format!("{pct_str}Charging ({source}) ⚡") } else { format!("{pct_str}Carregando ({source}) ⚡") }, theme.text_primary)
        }
        BatteryStatus::Full => (theme.accent_green, if is_en { "100% · Full 🟢".to_string() } else { "100% · Completa 🟢".to_string() }, theme.panel),
        BatteryStatus::Discharging { percentage } => (theme.panel_elevated, if is_en { format!("{percentage}% · In use 🔋") } else { format!("{percentage}% · Em uso 🔋") }, theme.text_primary),
        BatteryStatus::Stale { last_percentage, age_seconds } => {
            let pct_str = last_percentage.map(|p| format!("{p}%")).unwrap_or_else(|| "N/A".to_string());
            (theme.warning, if is_en { format!("Stale reading ({pct_str}, {age_seconds}s ago) ⚠️") } else { format!("Leitura antiga ({pct_str}, há {age_seconds}s) ⚠️") }, theme.panel)
        }
        BatteryStatus::Unavailable { reason, .. } => {
            (theme.panel_elevated, if is_en { format!("Unavailable ({reason}) ⚠️") } else { format!("Indisponível ({reason}) ⚠️") }, theme.text_muted)
        }
        BatteryStatus::Invalid { reason } => {
            (theme.warning, if is_en { format!("Invalid ({reason}) ❌") } else { format!("Inválida ({reason}) ❌") }, theme.panel)
        }
        BatteryStatus::Unknown => (theme.panel_elevated, if is_en { "No Info ❓".to_string() } else { "Sem Informação ❓".to_string() }, theme.text_muted),
    };

    let battery_pct_str = match &device.battery_status {
        BatteryStatus::Available { percentage, charging, .. } => {
            if *charging {
                format!("{percentage}% (⚡)")
            } else {
                format!("{percentage}%")
            }
        }
        BatteryStatus::Charging { percentage: Some(pct), .. } => format!("{pct}% (⚡)"),
        BatteryStatus::Charging { percentage: None, .. } => if is_en { "Charging".to_string() } else { "Carregando".to_string() },
        BatteryStatus::Full => "100%".to_string(),
        BatteryStatus::Discharging { percentage } => format!("{percentage}%"),
        BatteryStatus::Stale { last_percentage: Some(pct), .. } => format!("{pct}%"),
        _ => if is_en { "Unavailable".to_string() } else { "Indisponível".to_string() },
    };

    let (header_title, header_subtitle, detail_title, detail_subtitle) = match &device.battery_status {
        BatteryStatus::Available { percentage, source, .. } => (
            if is_en { "Battery Telemetry Status" } else { "Status de Telemetria da Bateria" },
            if is_en {
                format!("Reading confirmed ({percentage}%) via {}", source.as_ref().map(|s| s.to_string()).unwrap_or_else(|| "reliable provider".to_string()))
            } else {
                format!("Leitura confirmada ({percentage}%) via {}", source.as_ref().map(|s| s.to_string()).unwrap_or_else(|| "fonte confiável".to_string()))
            },
            if is_en { "Confirmed Battery Level" } else { "Nível de Bateria Confirmado" },
            if is_en { "The reading passed validation and is active in the interface." } else { "A leitura passou no pipeline de validação e está sendo transmitida à interface." },
        ),
        BatteryStatus::Charging { source, .. } => (
            if is_en { "Battery Telemetry Status" } else { "Status de Telemetria da Bateria" },
            if is_en { format!("Device charging via {source}") } else { format!("Dispositivo em carregamento via {source}") },
            if is_en { "Active Charging Mode" } else { "Modo de Carregamento Ativo" },
            if is_en { "Battery is receiving power and state is continuously monitored." } else { "A bateria está recebendo energia e o status é continuamente monitorado." },
        ),
        BatteryStatus::Full => (
            if is_en { "Battery Telemetry Status" } else { "Status de Telemetria da Bateria" },
            if is_en { "Battery at full capacity (100%)".to_string() } else { "Bateria com capacidade máxima (100%)".to_string() },
            if is_en { "Battery Fully Charged" } else { "Bateria Totalmente Carregada" },
            if is_en { "The device reached 100% full charge." } else { "O dispositivo atingiu a carga completa de 100%." },
        ),
        BatteryStatus::Discharging { percentage } => (
            if is_en { "Battery Telemetry Status" } else { "Status de Telemetria da Bateria" },
            if is_en { format!("Battery in normal use ({percentage}%)") } else { format!("Bateria em uso normal ({percentage}%)") },
            if is_en { "Wireless Mode Active" } else { "Modo Sem Fio Ativo" },
            if is_en { "Device powered by internal battery." } else { "Dispositivo alimentado pela bateria interna." },
        ),
        BatteryStatus::Stale { age_seconds, .. } => (
            if is_en { "Battery Telemetry Status" } else { "Status de Telemetria da Bateria" },
            if is_en { format!("Warning: last reading captured {age_seconds} seconds ago") } else { format!("Aviso: última leitura capturada há {age_seconds} segundos") },
            if is_en { "Stale Cached Reading" } else { "Leitura Antiga em Cache" },
            if is_en { "Click 'Refresh Battery Now' to scan via daemon." } else { "Pressione 'Atualizar Bateria Agora' para solicitar uma varredura ao daemon." },
        ),
        BatteryStatus::Unavailable { reason, .. } => (
            if is_en { "Battery Telemetry Status" } else { "Status de Telemetria da Bateria" },
            if is_en { format!("No reading confirmed: {reason}") } else { format!("Nenhuma leitura confirmada no momento: {reason}") },
            if is_en { "Battery Unavailable" } else { "Bateria Indisponível no Momento" },
            if is_en { "No dummy or unconfirmed 0% values are shown. Check technical log." } else { "A aplicação não exibe valores fictícios ou 0% não confirmados. Consulte o log abaixo." },
        ),
        BatteryStatus::Invalid { reason } => (
            if is_en { "Battery Telemetry Status" } else { "Status de Telemetria da Bateria" },
            if is_en { format!("Reading rejected during validation: {reason}") } else { format!("Leitura rejeitada durante a validação: {reason}") },
            if is_en { "Invalid Reading Discarded" } else { "Leitura Inválida Descartada" },
            if is_en { "Hardware reading violated security criteria and was discarded." } else { "A leitura do hardware violou os critérios de segurança e foi descartada." },
        ),
        BatteryStatus::Unknown => (
            if is_en { "Battery Telemetry Status" } else { "Status de Telemetria da Bateria" },
            if is_en { "Awaiting scan from hardware providers".to_string() } else { "Aguardando varredura dos provedores de hardware".to_string() },
            if is_en { "Telemetry Not Detected" } else { "Telemetria Não Detectada" },
            if is_en { "Click 'Refresh Battery Now' to trigger hardware bus scan." } else { "Clique em 'Atualizar Bateria Agora' para iniciar a busca nos barramentos." },
        ),
    };

    v_flex()
        .flex_1()
        .w_full()
        .p_6()
        .gap_6()
        .overflow_y_scrollbar()
        .child(
            // Summary Card
            v_flex()
                .w_full()
                .p_5()
                .rounded_xl()
                .bg(theme.panel)
                .border_1()
                .border_color(theme.border)
                .gap_4()
                .child(
                    h_flex()
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
                                        .child(header_title),
                                )
                                .child(
                                    div()
                                        .text_size(px(12.0))
                                        .text_color(theme.text_muted)
                                        .child(header_subtitle),
                                ),
                        )
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_full()
                                .bg(badge_bg)
                                .text_color(badge_fg)
                                .text_size(px(12.0))
                                .font_weight(gpui::FontWeight::BOLD)
                                .child(badge_text),
                        ),
                )
                .child(
                    h_flex()
                        .gap_6()
                        .items_center()
                        .child(
                            div()
                                .text_size(px(28.0))
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(if matches!(device.battery_status, BatteryStatus::Available { .. } | BatteryStatus::Full | BatteryStatus::Charging { .. }) {
                                    theme.text_primary
                                } else {
                                    theme.text_muted
                                })
                                .child(battery_pct_str),
                        )
                        .child(
                            v_flex()
                                .gap_1()
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme.text_primary)
                                        .child(detail_title),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .text_color(theme.text_muted)
                                        .child(detail_subtitle),
                                ),
                        ),
                )
                .child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .pt_2()
                        .border_t_1()
                        .border_color(theme.border)
                        .child(
                            div()
                                .id("btn-refresh-bat-diag")
                                .px_4()
                                .py_2()
                                .rounded_md()
                                .bg(theme.accent_blue)
                                .text_color(theme.text_primary)
                                .text_size(px(13.0))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .cursor_pointer()
                                .hover(|s| s.bg(theme.panel_elevated))
                                .on_click(move |_, _, cx| on_refresh_battery(cx))
                                .child(if is_en { "↻ Refresh Battery Now" } else { "↻ Atualizar Bateria Agora" }),
                        )
                        .child(
                            div()
                                .id("btn-trigger-diag-modal")
                                .px_4()
                                .py_2()
                                .rounded_md()
                                .bg(theme.panel_elevated)
                                .text_color(theme.text_primary)
                                .text_size(px(13.0))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .cursor_pointer()
                                .hover(|s| s.bg(theme.border))
                                .on_click(move |_, _, cx| cb_modal(cx))
                                .child(if is_en { "📄 Open Technical Report" } else { "📄 Abrir Relatório Técnico" }),
                        ),
                ),
        )
        .child(div().h(px(32.0)))
}
