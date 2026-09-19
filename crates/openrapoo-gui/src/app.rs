//! Main GPUI application view and router.

use crate::state::{AppState, DetailTab};
use crate::theme::current_theme;
use crate::ui::device_page::render_device_page;
use crate::ui::devices_page::render_devices_page;
use crate::ui::header::render_header;
use crate::ui::status_bar::render_status_bar;
use gpui::{
    div, px, Context, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window,
};
use gpui_component::v_flex;
use openrapoo_core::config::ButtonAction;
use openrapoo_core::permissions::check_input_group_status;
use std::rc::Rc;

pub enum ViewMode {
    DevicesList,
    DeviceDetail,
}

pub struct AppView {
    state: AppState,
    mode: ViewMode,
    focus_handle: gpui::FocusHandle,
}

impl AppView {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            state: AppState::new(),
            mode: ViewMode::DevicesList,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.focus(&self.focus_handle, cx);
        let theme = current_theme();

        let entity_back = cx.entity();
        let on_back = move |cx: &mut gpui::App| {
            entity_back.update(cx, |this, cx| {
                this.mode = ViewMode::DevicesList;
                cx.notify();
            });
        };

        let entity_refresh = cx.entity();
        let on_refresh = move |cx: &mut gpui::App| {
            entity_refresh.update(cx, |this, cx| {
                this.state.refresh_devices();
                cx.notify();
            });
        };

        let active_device = if matches!(self.mode, ViewMode::DeviceDetail) {
            self.state.selected_device()
        } else {
            None
        };

        let language = self.state.language;
        let header = render_header(active_device, &self.state.active_profile.name, language, on_back);

        let body = match self.mode {
            ViewMode::DevicesList => {
                let entity_hl = cx.entity();
                let on_highlight = Rc::new(move |idx: usize, cx: &mut gpui::App| {
                    entity_hl.update(cx, |this, cx| {
                        this.state.select_device(idx);
                        cx.notify();
                    });
                });

                let entity_cf = cx.entity();
                let on_confirm = Rc::new(move |idx: usize, cx: &mut gpui::App| {
                    entity_cf.update(cx, |this, cx| {
                        this.state.select_device(idx);
                        this.mode = ViewMode::DeviceDetail;
                        cx.notify();
                    });
                });

                let entity_prev = cx.entity();
                let on_prev = Rc::new(move |cx: &mut gpui::App| {
                    entity_prev.update(cx, |this, cx| {
                        let len = this.state.devices.len();
                        if len > 0 {
                            let curr = this.state.selected_device_index.unwrap_or(0);
                            let new_idx = (curr + len - 1) % len;
                            this.state.select_device(new_idx);
                            cx.notify();
                        }
                    });
                });

                let entity_next = cx.entity();
                let on_next = Rc::new(move |cx: &mut gpui::App| {
                    entity_next.update(cx, |this, cx| {
                        let len = this.state.devices.len();
                        if len > 0 {
                            let curr = this.state.selected_device_index.unwrap_or(0);
                            let new_idx = (curr + 1) % len;
                            this.state.select_device(new_idx);
                            cx.notify();
                        }
                    });
                });

                let entity_ref = cx.entity();
                let on_ref = move |cx: &mut gpui::App| {
                    entity_ref.update(cx, |this, cx| {
                        this.state.refresh_devices();
                        cx.notify();
                    });
                };

                v_flex().flex_1().w_full().overflow_hidden().child(render_devices_page(
                    &self.state.devices,
                    self.state.selected_device_index,
                    language,
                    on_highlight,
                    on_confirm,
                    on_prev,
                    on_next,
                    on_ref,
                ))
            }
            ViewMode::DeviceDetail => {
                let Some(dev) = self.state.selected_device() else {
                    let entity_ref = cx.entity();
                    return div()
                        .id("disconnected-view")
                        .flex_1()
                        .w_full()
                        .h_full()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap_4()
                        .child(
                            div()
                                .text_size(px(16.0))
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(theme.warning)
                                .child(crate::i18n::disconnected_title(language)),
                        )
                        .child(
                            div()
                                .text_size(px(13.0))
                                .text_color(theme.text_muted)
                                .child(crate::i18n::disconnected_desc(language)),
                        )
                        .child(
                            div()
                                .id("btn-reconnect")
                                .px_4()
                                .py_2()
                                .rounded_md()
                                .bg(theme.accent_blue)
                                .text_color(theme.text_primary)
                                .text_size(px(13.0))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .cursor_pointer()
                                .on_click(move |_, _, cx| {
                                    entity_ref.update(cx, |this, cx| {
                                        this.state.refresh_devices();
                                        cx.notify();
                                    });
                                })
                                .child(crate::i18n::reconnect_button(language)),
                        );
                };

                let active_tab = self.state.active_tab;
                let active_profile = self.state.active_profile.clone();
                let selected_hotspot_hex = self.state.selected_hotspot_hex.clone();

                let entity_tab = cx.entity();
                let on_select_tab: Rc<dyn Fn(DetailTab, &mut gpui::App)> =
                    Rc::new(move |tab: DetailTab, cx: &mut gpui::App| {
                        entity_tab.update(cx, |this, cx| {
                            this.state.active_tab = tab;
                            cx.notify();
                        });
                    });

                let entity_hotspot = cx.entity();
                let on_select_hotspot: Rc<dyn Fn(String, &mut gpui::App)> =
                    Rc::new(move |hex: String, cx: &mut gpui::App| {
                        entity_hotspot.update(cx, |this, cx| {
                            this.state.select_hotspot(Some(hex));
                            cx.notify();
                        });
                    });

                let entity_update = cx.entity();
                let on_update_action: Rc<dyn Fn(String, ButtonAction, &mut gpui::App)> = Rc::new(
                    move |hex: String, action: ButtonAction, cx: &mut gpui::App| {
                        entity_update.update(cx, |this, cx| {
                            this.state.update_button_action(&hex, action);
                            cx.notify();
                        });
                    },
                );

                let entity_reset = cx.entity();
                let on_reset_action: Rc<dyn Fn(String, &mut gpui::App)> =
                    Rc::new(move |hex: String, cx: &mut gpui::App| {
                        entity_reset.update(cx, |this, cx| {
                            this.state.reset_button_action(&hex);
                            cx.notify();
                        });
                    });

                let entity_dpi = cx.entity();
                let on_update_dpi: Rc<dyn Fn(u32, &mut gpui::App)> =
                    Rc::new(move |dpi: u32, cx: &mut gpui::App| {
                        entity_dpi.update(cx, |this, cx| {
                            this.state.update_dpi(dpi);
                            cx.notify();
                        });
                    });

                let entity_rate = cx.entity();
                let on_update_polling_rate: Rc<dyn Fn(u32, &mut gpui::App)> =
                    Rc::new(move |rate: u32, cx: &mut gpui::App| {
                        entity_rate.update(cx, |this, cx| {
                            this.state.update_polling_rate(rate);
                            cx.notify();
                        });
                    });

                let entity_apply = cx.entity();
                let on_apply_hardware: Rc<dyn Fn(&mut gpui::App)> =
                    Rc::new(move |cx: &mut gpui::App| {
                        entity_apply.update(cx, |this, cx| {
                            this.state.apply_hardware_settings();
                            cx.notify();
                        });
                    });

                let entity_restore = cx.entity();
                let on_restore_hardware: Rc<dyn Fn(&mut gpui::App)> =
                    Rc::new(move |cx: &mut gpui::App| {
                        entity_restore.update(cx, |this, cx| {
                            this.state.restore_hardware_snapshot();
                            cx.notify();
                        });
                    });

                let entity_read = cx.entity();
                let on_read_hardware: Rc<dyn Fn(&mut gpui::App)> =
                    Rc::new(move |cx: &mut gpui::App| {
                        entity_read.update(cx, |this, cx| {
                            this.state.read_hardware_state();
                            cx.notify();
                        });
                    });

                let entity_ref_bat = cx.entity();
                let on_refresh_battery: Rc<dyn Fn(&mut gpui::App)> =
                    Rc::new(move |cx: &mut gpui::App| {
                        entity_ref_bat.update(cx, |this, cx| {
                            this.state.refresh_selected_battery();
                            cx.notify();
                        });
                    });

                let entity_diag = cx.entity();
                let on_open_diag_modal: Rc<dyn Fn(&mut gpui::App)> =
                    Rc::new(move |cx: &mut gpui::App| {
                        entity_diag.update(cx, |this, cx| {
                            this.state.open_diag_modal();
                            cx.notify();
                        });
                    });

                v_flex().flex_1().w_full().overflow_hidden().child(render_device_page(
                    dev,
                    active_tab,
                    &active_profile,
                    selected_hotspot_hex.as_deref(),
                    language,
                    on_select_tab,
                    on_select_hotspot,
                    on_update_action,
                    on_reset_action,
                    on_update_dpi,
                    on_update_polling_rate,
                    on_apply_hardware,
                    on_restore_hardware,
                    on_read_hardware,
                    on_refresh_battery,
                    on_open_diag_modal,
                ))
            }
        };

        let group_ok = check_input_group_status().is_active();
        let daemon_running = self.state.is_daemon_running();
        let active_profile_name = self.state.active_profile.name.clone();

        let entity_lang = cx.entity();
        let on_toggle_language = move |cx: &mut gpui::App| {
            entity_lang.update(cx, |this, cx| {
                this.state.toggle_language();
                cx.notify();
            });
        };

        let status_bar =
            render_status_bar(group_ok, daemon_running, &active_profile_name, language, on_refresh, on_toggle_language);

        let entity_prev = cx.entity();
        let entity_next = cx.entity();
        let entity_cf = cx.entity();

        let mut root = v_flex()
            .id("app-root")
            .track_focus(&self.focus_handle)
            .on_key_down(move |event, _window, cx| {
                let key = event.keystroke.key.to_lowercase();
                match key.as_str() {
                    "arrowleft" | "left" | "<" | "," | "h" | "arrowup" | "up" => {
                        entity_prev.update(cx, |this, cx| {
                            if matches!(this.mode, ViewMode::DevicesList) {
                                let len = this.state.devices.len();
                                if len > 0 {
                                    let curr = this.state.selected_device_index.unwrap_or(0);
                                    let new_idx = (curr + len - 1) % len;
                                    this.state.select_device(new_idx);
                                    cx.notify();
                                }
                            }
                        });
                    }
                    "arrowright" | "right" | ">" | "." | "l" | "arrowdown" | "down" => {
                        entity_next.update(cx, |this, cx| {
                            if matches!(this.mode, ViewMode::DevicesList) {
                                let len = this.state.devices.len();
                                if len > 0 {
                                    let curr = this.state.selected_device_index.unwrap_or(0);
                                    let new_idx = (curr + 1) % len;
                                    this.state.select_device(new_idx);
                                    cx.notify();
                                }
                            }
                        });
                    }
                    "enter" | "return" | "space" => {
                        entity_cf.update(cx, |this, cx| {
                            if matches!(this.mode, ViewMode::DevicesList) {
                                let curr = this.state.selected_device_index.unwrap_or(0);
                                if !this.state.devices.is_empty() {
                                    this.state.select_device(curr);
                                    this.mode = ViewMode::DeviceDetail;
                                    cx.notify();
                                }
                            }
                        });
                    }
                    _ => {}
                }
            })
            .flex_1()
            .w_full()
            .h_full()
            .overflow_hidden()
            .bg(theme.bg)
            .child(header)
            .child(body)
            .child(status_bar);

        if self.state.show_diag_modal {
            let report = self
                .state
                .selected_device()
                .map(|dev| {
                    crate::services::BatteryService::get_diagnostic_report(dev).to_markdown()
                })
                .unwrap_or_default();

            let entity_modal_copy = cx.entity();
            let on_modal_copy: Rc<dyn Fn(String, &mut gpui::App)> =
                Rc::new(move |content: String, cx: &mut gpui::App| {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(content));
                    entity_modal_copy.update(cx, |this, cx| {
                        this.state.export_toast_message =
                            Some("Conteúdo copiado para a área de transferência!".into());
                        cx.notify();
                    });
                });

            let entity_modal_export = cx.entity();
            let on_modal_export: Rc<dyn Fn(String, &mut gpui::App)> =
                Rc::new(move |content: String, cx: &mut gpui::App| {
                    entity_modal_export.update(cx, |this, cx| {
                        this.state.export_diag_report(&content);
                        cx.notify();
                    });
                });

            let entity_modal_close = cx.entity();
            let on_modal_close: Rc<dyn Fn(&mut gpui::App)> = Rc::new(move |cx: &mut gpui::App| {
                entity_modal_close.update(cx, |this, cx| {
                    this.state.close_diag_modal();
                    cx.notify();
                });
            });

            let modal_overlay = crate::ui::diag_modal::render_diag_modal(
                &report,
                self.state.export_toast_message.as_deref(),
                self.state.language,
                on_modal_copy,
                on_modal_export,
                on_modal_close,
            );

            root = root.child(modal_overlay);
        }

        root
    }
}
