//! Dockable workspace built with [`egui_tiles`].
//!
//! The template demonstrates the `egui_tiles` containers: a horizontal split
//! and a tab strip. Panes can be dragged between tabs and splitters can be
//! resized at runtime, so this module is a good starting point for building a
//! full editor-style layout.
//!
//! The in-app log viewer ([`egui_logger`]) is also a dock pane here, which
//! makes it draggable and dockable exactly like every other tab.
//!
//! Each pane shows a small Phosphor "move" handle in its top-right corner
//! (6px inset) that starts a tile drag. The whole layout can be locked from
//! the top bar: while locked, dragging and resizing are disabled (see
//! [`DockBehavior::is_tile_draggable`] and
//! [`DockBehavior::is_container_resizable`]).

use eframe::egui;
use egui_file_dialog::FileDialog;
use egui_phosphor::regular;
use egui_plot::{Line, Plot, PlotPoints};

/// Inset of the pane move handle from the top-right corner, in points.
const MOVE_HANDLE_MARGIN: f32 = 6.0;
/// Edge length of the square move handle, in points.
const MOVE_HANDLE_SIZE: f32 = 22.0;

/// The kind of content a [`DockPane`] hosts.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DockPaneKind {
    /// One of the template demo panes (icons / plot / placeholder).
    Demo(usize),
    /// The in-app log viewer backed by [`egui_logger`].
    Log,
}

/// A single dock pane.
pub struct DockPane {
    kind: DockPaneKind,
}

/// Bridges the dock tree with the rest of the application state: the shared
/// [`FileDialog`] that the icon demo pane opens, and the layout lock flag
/// toggled from the top bar.
pub struct DockBehavior<'a> {
    pub file_dialog: &'a mut FileDialog,
    /// When `true` the layout is locked: panes cannot be dragged, tabs
    /// cannot be re-docked, and splitters cannot be resized.
    pub locked: bool,
}

impl egui_tiles::Behavior<DockPane> for DockBehavior<'_> {
    fn tab_title_for_pane(&mut self, pane: &DockPane) -> egui::WidgetText {
        match pane.kind {
            DockPaneKind::Demo(nr) => format!("Pane {nr}").into(),
            DockPaneKind::Log => "Log".into(),
        }
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut DockPane,
    ) -> egui_tiles::UiResponse {
        ui.vertical(|ui| match pane.kind {
            DockPaneKind::Demo(nr) => demo_pane_ui(ui, nr, &mut *self.file_dialog),
            DockPaneKind::Log => log_pane_ui(ui),
        });

        if !self.locked {
            // Floating Phosphor "move" handle in the top-right corner, 6px
            // from the top and right edges of the pane. Dragging it starts
            // the egui_tiles tile drag.
            let top_right = ui.max_rect().right_top();
            let rect = egui::Rect::from_min_size(
                top_right + egui::vec2(-MOVE_HANDLE_SIZE - MOVE_HANDLE_MARGIN, MOVE_HANDLE_MARGIN),
                egui::Vec2::splat(MOVE_HANDLE_SIZE),
            );
            let response = ui.put(
                rect,
                egui::Button::new(egui::RichText::new(regular::ARROWS_OUT_CARDINAL).size(16.0))
                    .sense(egui::Sense::drag()),
            );
            if response.drag_started() {
                return egui_tiles::UiResponse::DragStarted;
            }
        }

        egui_tiles::UiResponse::None
    }

    /// Locked layouts cannot be rearranged by dragging panes or tabs.
    fn is_tile_draggable(
        &self,
        _tiles: &egui_tiles::Tiles<DockPane>,
        _tile_id: egui_tiles::TileId,
    ) -> bool {
        !self.locked
    }

    /// Locked layouts cannot be resized by dragging the splitters.
    fn is_container_resizable(
        &self,
        _tiles: &egui_tiles::Tiles<DockPane>,
        _tile_id: egui_tiles::TileId,
    ) -> bool {
        !self.locked
    }
}

/// Renders one of the numbered demo panes.
fn demo_pane_ui(ui: &mut egui::Ui, nr: usize, file_dialog: &mut FileDialog) {
    ui.heading(format!("Dock pane {nr}"));
    match nr {
        0 => icons_pane(ui, file_dialog),
        1 => plot_pane(ui),
        _ => {
            ui.label("Resize the splitters or drag tabs to dock tiles.");
        }
    }
}

/// Renders the in-app log viewer as a regular dock pane.
fn log_pane_ui(ui: &mut egui::Ui) {
    egui_logger::logger_ui().show(ui);
}

/// Demo pane 0: demonstrates egui-phosphor icon glyphs and the file dialog.
fn icons_pane(ui: &mut egui::Ui, file_dialog: &mut FileDialog) {
    ui.label(format!(
        "Phosphor icons: {} {} {}",
        regular::ALARM,
        regular::AIRPLANE,
        regular::FOLDER_OPEN
    ));
    ui.horizontal(|ui| {
        let _ = ui.button(regular::ALARM);
        let _ = ui.button(regular::AIRPLANE);
        if ui.button(regular::FOLDER_OPEN).clicked() {
            file_dialog.pick_file();
        }
    });
}

/// Demo pane 1: demonstrates egui-plot with a continuously sampled sine wave.
fn plot_pane(ui: &mut egui::Ui) {
    Plot::new("demo_plot")
        .height(ui.available_height() - 30.0)
        .show(ui, |plot_ui| {
            plot_ui.line(Line::new(
                "sin(x)",
                PlotPoints::from_explicit_callback(|x| x.sin(), -10.0..=10.0, 256),
            ));
        });
}

/// Builds the initial dock tree: a horizontal split with the two demo panes
/// plus a dockable "Log" tab, all under a shared root tab tile.
pub fn create_dock_tree() -> egui_tiles::Tree<DockPane> {
    let mut next_view_nr = 0;
    let mut gen_demo = || {
        let pane = DockPane {
            kind: DockPaneKind::Demo(next_view_nr),
        };
        next_view_nr += 1;
        pane
    };

    let mut tiles = egui_tiles::Tiles::default();

    let mut tabs = vec![];
    tabs.push({
        let children = (0..2).map(|_| tiles.insert_pane(gen_demo())).collect();
        tiles.insert_horizontal_tile(children)
    });
    tabs.push(tiles.insert_pane(DockPane {
        kind: DockPaneKind::Log,
    }));

    let root = tiles.insert_tab_tile(tabs);

    egui_tiles::Tree::new("main_dock", root, tiles)
}

#[cfg(test)]
mod tests {
    use super::{DockBehavior, DockPane, DockPaneKind, create_dock_tree};
    use egui_file_dialog::FileDialog;
    use egui_tiles::Behavior;

    #[test]
    fn dock_tree_has_multiple_tiles_and_root() {
        let tree = create_dock_tree();
        assert!(tree.root.is_some(), "dock tree should have a root tile");
        assert!(tree.tiles.len() > 1, "dock should contain multiple tiles");
    }

    #[test]
    fn locked_layout_disables_dragging_and_resizing() {
        let mut file_dialog = FileDialog::new();
        let mut tiles = egui_tiles::Tiles::default();
        let pane_id = tiles.insert_pane(DockPane {
            kind: DockPaneKind::Demo(0),
        });

        let unlocked = DockBehavior {
            file_dialog: &mut file_dialog,
            locked: false,
        };
        assert!(unlocked.is_tile_draggable(&tiles, pane_id));
        assert!(unlocked.is_container_resizable(&tiles, pane_id));

        let locked = DockBehavior {
            file_dialog: &mut file_dialog,
            locked: true,
        };
        assert!(!locked.is_tile_draggable(&tiles, pane_id));
        assert!(!locked.is_container_resizable(&tiles, pane_id));
    }
}
