//! `egui_app` — a cross-platform desktop application template built with
//! [eframe](https://docs.rs/eframe) / [egui](https://github.com/emilk/egui).
//!
//! This binary is intentionally thin: it installs logging and fonts, applies
//! platform theming, and hands control to [`eframe::run_native`]. All
//! application logic lives in the sibling modules:
//!
//! - [`app`] — application state and the [`eframe::App`] implementation
//! - [`dock`] — dockable workspace built on [`egui_tiles`]
//! - [`logging`] — in-app logging via [`egui_logger`]
//! - [`platform`] — OS-specific detection and theming

#![warn(missing_docs)]

mod app;
mod dock;
mod logging;
mod platform;

use eframe::egui;

/// Human-readable name shown in the window title bar.
const APP_TITLE: &str = "egui cross-platform template";

fn main() -> eframe::Result<()> {
    logging::init_logger().expect("global logger should be installed once at process startup");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(|cc| {
            install_icon_font(&cc.egui_ctx);

            if platform::force_dark_theme_on_windows() {
                cc.egui_ctx.set_theme(egui::Theme::Dark);
            }

            Ok(Box::new(app::TemplateApp::new()))
        }),
    )
}

/// Installs the egui-phosphor icon font so icon glyphs render as pictograms
/// instead of fallback boxes. Must run once, before the first frame.
fn install_icon_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    ctx.set_fonts(fonts);
}
