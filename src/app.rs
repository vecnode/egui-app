//! Application state and the main [`eframe::App`] implementation.
//!
//! [`TemplateApp`] owns everything that lives for the whole process: the
//! dockable tile layout, the shared file dialog, and a one-shot flag used to
//! log a startup message. The per-frame UI is built in
//! [`eframe::App::ui`], and cheap frame-independent bookkeeping happens in
//! [`eframe::App::logic`].

use eframe::egui;
use egui_file_dialog::FileDialog;

use crate::dock::{DockBehavior, DockPane, create_dock_tree};

/// Root application state, created once at startup by [`TemplateApp::new`].
pub struct TemplateApp {
    /// The dockable workspace shown in the central panel.
    tree: egui_tiles::Tree<DockPane>,
    /// Shared file dialog, opened from dock pane 0.
    file_dialog: FileDialog,
    /// Ensures the startup message is logged exactly once.
    logged_startup: bool,
}

impl TemplateApp {
    /// Creates the application with a fresh dock layout and file dialog.
    pub fn new() -> Self {
        Self {
            tree: create_dock_tree(),
            file_dialog: FileDialog::new(),
            logged_startup: false,
        }
    }
}

impl eframe::App for TemplateApp {
    /// Frame-independent bookkeeping, called before [`Self::ui`] each frame.
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.logged_startup {
            log::info!("egui_app started; open the Log window for captured output");
            self.logged_startup = true;
        }
    }

    /// Draws the dockable workspace and the floating "Log" window.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let mut behavior = DockBehavior {
                file_dialog: &mut self.file_dialog,
            };
            self.tree.ui(&mut behavior, ui);

            self.file_dialog.update(ui);
            if let Some(path) = self.file_dialog.take_picked() {
                log::info!("picked file: {}", path.display());
            }
        });

        egui::Window::new("Log")
            .default_size([1024.0, 360.0])
            .show(ui.ctx(), |ui| {
                egui_logger::logger_ui().show(ui);
            });
    }
}
