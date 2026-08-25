//! Logging setup.
//!
//! Uses [`egui_logger`] to capture [`log`] records and display them inside an
//! egui window (the "Log" window rendered in [`crate::app::TemplateApp`]).
//! Because the logger is installed at `Debug` level, `log::debug!` and above
//! statements anywhere in the app appear in that window.

/// Installs the global logger at `Debug` level.
///
/// Must only be called once, at process startup. Returns the previous logger
/// error when a logger is already installed.
pub fn init_logger() -> Result<(), log::SetLoggerError> {
    egui_logger::builder()
        .max_level(log::LevelFilter::Debug)
        .init()
}
