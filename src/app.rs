//! Application state and the main [`eframe::App`] implementation.
//!
//! [`TemplateApp`] owns everything that lives for the whole process: the
//! dockable tile layout, the shared file dialog, the layout-lock flag, the
//! theme preference, and a one-shot flag used to log a startup message. The
//! per-frame UI is built in [`eframe::App::ui`], and cheap frame-independent
//! bookkeeping happens in [`eframe::App::logic`].

use eframe::egui;
use egui_file_dialog::FileDialog;
use egui_phosphor::regular;

use crate::dock::{DockBehavior, DockPane};
use crate::persist;

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
    /// Light/dark theme preference (never `System`), toggled from the top
    /// bar; the initial value follows the OS theme at startup.
    theme_pref: egui::ThemePreference,
    /// Ensures the startup message is logged exactly once.
    logged_startup: bool,
}

impl TemplateApp {
    /// Creates the application with the persisted dock layout (or the default
    /// one on first run), a fresh file dialog, and the initial theme
    /// preference (derived from the OS theme at startup).
    pub fn new(theme_pref: egui::ThemePreference) -> Self {
        Self {
            tree: persist::load_layout(),
            file_dialog: FileDialog::new(),
            locked: false,
            theme_pref,
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

    /// Draws the top bar (theme + layout lock) and the dockable workspace,
    /// which is framed by a border. The log viewer lives in its own dock pane
    /// (see [`crate::dock`]), so it can be dragged and docked like any other
    /// tab.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.top_bar(ui);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    // Theme-aware backdrop: panes (panel_fill) sit on the
                    // stronger extreme_bg_color, so they never look flat
                    // black and the whole app changes with the theme.
                    .fill(ui.visuals().extreme_bg_color)
                    .inner_margin(4.0)
                    .stroke(egui::Stroke::new(
                        1.0,
                        ui.visuals().widgets.noninteractive.bg_stroke.color,
                    )),
            )
            .show_inside(ui, |ui| {
                // The behavior flags the layout as dirty whenever a drag or
                // resize changed the tree; persist it right after the frame.
                let mut layout_dirty = false;
                let mut behavior = DockBehavior {
                    file_dialog: &mut self.file_dialog,
                    locked: self.locked,
                    layout_dirty: &mut layout_dirty,
                };
                self.tree.ui(&mut behavior, ui);
                if layout_dirty {
                    persist::save_layout(&self.tree);
                }

                self.file_dialog.update(ui);
                if let Some(path) = self.file_dialog.take_picked() {
                    log::info!("picked file: {}", path.display());
                }
            });
    }
}

impl TemplateApp {
    /// Renders the top bar: app label on the left; on the right the theme
    /// toggle and the layout lock/unlock toggle (lock is the rightmost).
    fn top_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("top_bar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(TOP_BAR_TITLE);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);

                    // Rightmost: layout lock/unlock.
                    let (lock_icon, lock_tip) = if self.locked {
                        (regular::LOCK_SIMPLE, "Unlock the layout")
                    } else {
                        (regular::LOCK_SIMPLE_OPEN, "Lock the layout")
                    };
                    if ui
                        .add(egui::Button::new(egui::RichText::new(lock_icon).size(18.0)))
                        .on_hover_text(lock_tip)
                        .clicked()
                    {
                        self.locked = !self.locked;
                        log::info!("layout {}", if self.locked { "locked" } else { "unlocked" });
                    }

                    // Just left of the lock: light/dark theme toggle.
                    ui.add_space(4.0);
                    let (theme_icon, theme_tip) = if self.theme_pref == egui::ThemePreference::Light
                    {
                        (regular::SUN, "Theme: light — click for dark")
                    } else {
                        (regular::MOON, "Theme: dark — click for light")
                    };
                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new(theme_icon).size(18.0),
                        ))
                        .on_hover_text(theme_tip)
                        .clicked()
                    {
                        self.theme_pref = if self.theme_pref == egui::ThemePreference::Light {
                            egui::ThemePreference::Dark
                        } else {
                            egui::ThemePreference::Light
                        };
                        ui.ctx().set_theme(self.theme_pref);
                        log::info!(
                            "theme: {}",
                            if self.theme_pref == egui::ThemePreference::Light {
                                "light"
                            } else {
                                "dark"
                            }
                        );
                    }
                });
            });
        });
    }
}
