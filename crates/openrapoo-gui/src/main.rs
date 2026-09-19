//! OpenRapoo GPUI Graphical User Interface Application Entry Point.
#![allow(clippy::type_complexity, clippy::too_many_arguments)]

mod actions;
mod app;
mod assets;
mod device_model;
mod i18n;
mod services;
mod settings;
mod state;
mod theme;
mod ui;

use app::AppView;
use gpui::{px, AppContext, Bounds, SharedString, Size, TitlebarOptions, WindowBounds, WindowOptions};
use tracing::info;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_env("OPENRAPOO_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Starting OpenRapoo GPUI v{}", env!("CARGO_PKG_VERSION"));

    let app = gpui_platform::application();

    app.run(move |cx| {
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, Size::new(px(1040.0), px(680.0)), cx);
        let window_options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some(SharedString::from("OpenRapoo")),
                appears_transparent: false,
                ..Default::default()
            }),
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_min_size: Some(Size::new(px(800.0), px(600.0))),
            app_id: Some("openrapoo-gui".into()),
            ..WindowOptions::default()
        };

        cx.open_window(window_options, |window, cx| {
            cx.new(|cx| AppView::new(window, cx))
        })
        .expect("Failed to open GPUI main window");
    });
}
