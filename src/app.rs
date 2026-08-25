//! Application state and the main [`eframe::App`] implementation.
//!
//! [`TemplateApp`] owns everything that lives for the whole process: the
//! dockable tile layout, the shared file dialog, the layout-lock flag, and a
//! one-shot flag used to log a startup message. The per-frame UI is built in
//! [`eframe::App::ui`], and cheap frame-independent bookkeeping happens in
//! [`eframe::App::logic`].

use eframe::egui;
use egui_file_dialog::FileDialog;
use egui_phosphor::regular;

use crate::dock::{DockBehavior, DockPane, create_dock_tree};

/// Label shown on the left side of the top bar.
const TOP_BAR_TITLE: &str = "egui-app";

/// Root application state, created once at startup by [`TemplateApp::new`].
pub struct TemplateApp {
    /// The dockable workspace shown in the central panel.
    tree: egui_tiles::Tree<DockPane>,
    /// Shared file dialog, opened from dock pane 0.
    file_dialog: FileDialog,
    /// When `true` the dock layout is locked (no dragging or resizing).
    locked: bool,
    /// Ensures the startup message is logged exactly once.
    logged_startup: bool,
}

impl TemplateApp {
    /// Creates the application with a fresh dock layout and file dialog.
    pub fn new() -> Self {
        Self {
            tree: create_dock_tree(),
            file_dialog: FileDialog::new(),
            locked: false,
            logged_startup: false,
        }
    }
}

impl eframe::App for TemplateApp {
    /// Frame-independent bookkeeping, called before [`Self::ui`] each frame.
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.logged_startup {
            log::info!("egui_app started; logs appear in the dockable Log tab");
            self.logged_startup = true;
        }
    }

    /// Draws the top bar (layout lock toggle) and the dockable workspace.
    /// The log viewer lives in its own dock pane (see [`crate::dock`]), so it
    /// can be dragged and docked like any other tab.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.top_bar(ui);

        egui::CentralPanel::default().show_inside(ui, |ui| {
            let mut behavior = DockBehavior {
                file_dialog: &mut self.file_dialog,
                locked: self.locked,
            };
            self.tree.ui(&mut behavior, ui);

            self.file_dialog.update(ui);
            if let Some(path) = self.file_dialog.take_picked() {
                log::info!("picked file: {}", path.display());
            }
        });
    }
}

impl TemplateApp {
    /// Renders the top bar: app label on the left, layout lock/unlock toggle
    /// on the right.
    fn top_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("top_bar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(TOP_BAR_TITLE);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    let (icon, tip) = if self.locked {
                        (regular::LOCK_SIMPLE, "Unlock the layout")
                    } else {
                        (regular::LOCK_SIMPLE_OPEN, "Lock the layout")
                    };
                    if ui
                        .add(egui::Button::new(egui::RichText::new(icon).size(18.0)))
                        .on_hover_text(tip)
                        .clicked()
                    {
                        self.locked = !self.locked;
                        log::info!("layout {}", if self.locked { "locked" } else { "unlocked" });
                    }
                });
            });
        });
    }
}
