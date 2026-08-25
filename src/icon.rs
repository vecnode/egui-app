//! Application icon handling.
//!
//! The icon PNG is embedded into the binary at compile time
//! ([`include_bytes!`]) and decoded into [`egui::IconData`] at startup, so the
//! window title-bar icon (small) and the taskbar/dock icon (large) come from
//! the same source on Windows, Linux and macOS — no file I/O at runtime.
//!
//! Platform extras live in `assets/`:
//!
//! - `assets/icon.ico` — embedded into `egui_app.exe` by `build.rs`, so
//!   Explorer shows the icon for the executable itself.
//! - `assets/icon.icns` — used by the macOS `.app` bundle (see
//!   `build_app.sh --bundle`).
//! - `assets/linux/egui-app.desktop` — desktop-entry template for Linux.

use eframe::egui;

/// The bundled icon, embedded at compile time (512×512 PNG).
const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

/// Decodes the bundled icon into [`egui::IconData`] for
/// [`egui::ViewportBuilder::with_icon`].
///
/// Returns `None` only if the embedded PNG is malformed, in which case the
/// app runs with the default (empty) icon.
pub fn load_app_icon() -> Option<egui::IconData> {
    let image = image::ImageReader::new(std::io::Cursor::new(ICON_PNG))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?
        .to_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}
