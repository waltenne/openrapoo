//! Custom CSS styling and GTK4 theme manager for OpenRapoo.

use gtk4::gdk::Display;
use gtk4::CssProvider;
use tracing::info;

pub const OPENRAPOO_CSS: &str = r#"
/* OpenRapoo Dark Modern Minimalist Theme */

.openrapoo-header {
    padding: 8px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.openrapoo-title {
    font-weight: bold;
    font-size: 15px;
}

.connection-badge {
    background-color: rgba(46, 194, 126, 0.2);
    color: #2ec4b6;
    border-radius: 12px;
    padding: 3px 10px;
    font-size: 12px;
    font-weight: 600;
}

.connection-badge.disconnected {
    background-color: rgba(224, 27, 36, 0.2);
    color: #f66151;
}

.card-frame {
    background-color: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    padding: 12px;
}

.hotspot-pin {
    background-color: #3584e4;
    color: #ffffff;
    border-radius: 50%;
    min-width: 24px;
    min-height: 24px;
    font-weight: bold;
    font-size: 11px;
    border: 2px solid #ffffff;
}

.hotspot-pin:hover {
    background-color: #62a0ea;
    transform: scale(1.1);
}

.action-card {
    background-color: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 10px;
    padding: 10px 14px;
}

.action-card:hover {
    background-color: rgba(255, 255, 255, 0.09);
    border-color: #3584e4;
}

.action-title {
    font-weight: bold;
    font-size: 13px;
    color: #ffffff;
}

.action-subtitle {
    font-size: 11px;
    color: #9a9996;
}

.status-bar {
    background-color: rgba(0, 0, 0, 0.2);
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    padding: 6px 16px;
    font-size: 11px;
    color: #9a9996;
}

.unsupported-banner {
    background-color: rgba(222, 126, 44, 0.15);
    border: 1px solid rgba(222, 126, 44, 0.3);
    color: #f5c2e7;
    border-radius: 8px;
    padding: 10px 14px;
    font-size: 12px;
}
"#;

pub fn load_custom_css() {
    if let Some(display) = Display::default() {
        let provider = CssProvider::new();
        provider.load_from_data(OPENRAPOO_CSS);
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        info!("Applied custom GTK4 CSS styling for OpenRapoo");
    }
}
