//! Custom CSS styling and GTK4 theme manager for OpenRapoo.

use gtk4::gdk::Display;
use gtk4::CssProvider;
use tracing::info;

pub const OPENRAPOO_CSS: &str = r#"
/* OpenRapoo — OpenLogi Minimal Dark Design System */

window {
    background-color: #121215;
    color: #ffffff;
    font-family: system-ui, -apple-system, Cantarell, "Segoe UI", Roboto, sans-serif;
}

headerbar {
    background-color: #121215;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    padding: 6px 12px;
    box-shadow: none;
}

.device-title {
    font-size: 16px;
    font-weight: 700;
    color: #ffffff;
}

.back-button {
    background: transparent;
    border: none;
    color: #8e8e93;
    font-weight: 600;
    font-size: 13px;
    padding: 6px 12px;
}

.back-button:hover {
    color: #ffffff;
    background-color: rgba(255, 255, 255, 0.06);
    border-radius: 6px;
}

/* Floating Pill Tab Switcher */
.pill-switcher {
    background-color: #1a1a1e;
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 20px;
    padding: 3px;
}

.pill-tab {
    background: transparent;
    border: none;
    color: #8e8e93;
    font-size: 13px;
    font-weight: 500;
    border-radius: 16px;
    padding: 5px 16px;
    transition: all 150ms ease;
}

.pill-tab:hover {
    color: #ffffff;
    background-color: rgba(255, 255, 255, 0.05);
}

.pill-tab.active {
    background-color: #2a2a34;
    color: #ffffff;
    font-weight: 600;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
}

/* Header Status Pill */
.status-pill {
    background-color: #1a1a1e;
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 16px;
    padding: 4px 12px;
}

.status-dot {
    background-color: #22c55e;
    border-radius: 50%;
    min-width: 8px;
    min-height: 8px;
    margin-right: 6px;
}

.status-dot.disconnected {
    background-color: #ef4444;
}

.status-text {
    font-size: 12px;
    font-weight: 600;
    color: #ffffff;
}

/* OpenLogi Action Cards */
.openlogi-card {
    background-color: #1a1a1e;
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 8px;
    padding: 10px 14px;
    min-width: 175px;
    transition: all 120ms ease;
}

.openlogi-card:hover {
    background-color: #24242d;
    border-color: rgba(255, 255, 255, 0.15);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
}

.openlogi-card.active {
    border-color: #3b82f6;
    background-color: #1e2538;
}

.card-label-sub {
    font-size: 11px;
    font-weight: 500;
    color: #8e8e93;
    margin-bottom: 3px;
}

.card-label-title {
    font-size: 13px;
    font-weight: 600;
    color: #ffffff;
}

.card-chevron {
    color: #555560;
    font-size: 13px;
    font-weight: bold;
}

/* Main Canvas Area */
.main-canvas {
    background-color: #121215;
    padding: 24px;
}

/* Bottom Status Footer */
.status-footer {
    background-color: #121215;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    padding: 10px 24px;
    font-size: 11px;
    color: #6e6e78;
}

/* Banner / Warning Styling */
.unsupported-banner {
    background-color: rgba(234, 179, 8, 0.1);
    border: 1px solid rgba(234, 179, 8, 0.25);
    color: #fef08a;
    border-radius: 8px;
    padding: 12px 16px;
    font-size: 12px;
}
"#;

pub fn load_custom_css() {
    if let Some(display) = Display::default() {
        let provider = CssProvider::new();
        provider.load_from_string(OPENRAPOO_CSS);
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        info!("Applied OpenLogi minimal dark CSS styling for OpenRapoo");
    }
}


