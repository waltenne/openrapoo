//! System diagnostics tab page.

use crate::theme::current_theme;
use gpui::{div, px, IntoElement, ParentElement, Styled};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex};

#[allow(dead_code)]
pub fn render_diagnostics_page() -> impl IntoElement {
    let theme = current_theme();

    v_flex()
        .flex_1()
        .w_full()
        .h_full()
        .p_6()
        .gap_6()
        .overflow_y_scrollbar()
        .child(
            v_flex()
                .gap_1()
                .child(
                    div()
                        .text_size(px(18.0))
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme.text_primary)
                        .child("Diagnóstico do Sistema"),
                )
                .child(
                    div()
                        .text_size(px(13.0))
                        .text_color(theme.text_muted)
                        .child("Logs de eventos em tempo real e verificação do subsistema HID."),
                ),
        )
        .child(
            v_flex()
                .flex_1()
                .min_h(px(240.0))
                .p_4()
                .rounded_lg()
                .bg(theme.panel)
                .border_1()
                .border_color(theme.border)
                .gap_3()
                .child(
                    h_flex()
                        .justify_between()
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(theme.text_primary)
                                .child("Console de Eventos Evdev / HIDRAW"),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(theme.accent_green)
                                .child("Escutando eventos em tempo real..."),
                        ),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .p_3()
                        .rounded_md()
                        .bg(theme.bg)
                        .text_size(px(11.0))
                        .text_color(theme.text_muted)
                        .gap_1()
                        .child("[SYS] OpenRapoo GPUI inicializado com sucesso (v0.1.0).")
                        .child("[HID] Scanner de entrada conectado ao nó /dev/input/event5.")
                        .child("[DEV] Rapoo MT760 Pro (0x24AE:0x186A) pronto.")
                        .child("[BAT] Bateria detectada via sysfs: 85% (Sem carregar).")
                        .child("[IPC] Conexão com daemon /tmp/openrapoo.sock estabelecida."),
                ),
        )
}
