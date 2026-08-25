//! `egui_app` — a cross-platform desktop application template built with
//! [eframe](https://docs.rs/eframe) / [egui](https://github.com/emilk/egui).
//!
//! This binary is intentionally thin: it installs logging, fonts and the
//! application icon, lets the theme follow the operating system, and hands
//! control to [`eframe::run_native`]. All application logic lives in the
//! sibling modules:
//!
//! - [`app`] — application state and the [`eframe::App`] implementation
//! - [`dock`] — dockable workspace (including the Log pane) built on [`egui_tiles`]
//! - [`icon`] — bundled application icon (window title bar + taskbar/dock)
//! - [`logging`] — in-app logging via [`egui_logger`]

#![warn(missing_docs)]

mod app;
mod dock;
mod icon;
mod logging;

use eframe::egui;

/// Human-readable name shown in the window title bar.
const APP_TITLE: &str = "egui cross-platform template";

fn main() -> eframe::Result<()> {
    logging::init_logger().expect("global logger should be installed once at process startup");

    let mut viewport = egui::ViewportBuilder::default().with_inner_size([1024.0, 720.0]);
    if let Some(icon) = icon::load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        APP_TITLE,
        options,
        Box::new(|cc| {
            install_icon_font(&cc.egui_ctx);

            // Follow the operating system's light/dark theme automatically.
            // egui/eframe reads the OS setting in safe Rust (via winit, which
            // uses the native APIs on each platform) and keeps it updated when
            // the OS theme changes. The user can override it from the top bar.
            cc.egui_ctx.set_theme(egui::ThemePreference::System);

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
