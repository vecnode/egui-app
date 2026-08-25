//! Dockable workspace built with [`egui_tiles`].
//!
//! The template demonstrates the three tile containers offered by
//! `egui_tiles`: horizontal splits, grids, and tabs. Panes can be dragged
//! between tabs and splitters can be resized at runtime, so this module is a
//! good starting point for building a full editor-style layout.

use eframe::egui;
use egui_file_dialog::FileDialog;
use egui_phosphor::regular;
use egui_plot::{Line, Plot, PlotPoints};

/// A single dock pane, identified by its number.
pub struct DockPane {
    nr: usize,
}

/// Bridges the dock tree with the rest of the application state, currently
/// the shared [`FileDialog`] that pane 0 opens.
pub struct DockBehavior<'a> {
    pub file_dialog: &'a mut FileDialog,
}

impl egui_tiles::Behavior<DockPane> for DockBehavior<'_> {
    fn tab_title_for_pane(&mut self, pane: &DockPane) -> egui::WidgetText {
        format!("Pane {}", pane.nr).into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut DockPane,
    ) -> egui_tiles::UiResponse {
        ui.vertical(|ui| {
            ui.heading(format!("Dock pane {}", pane.nr));

            match pane.nr {
                0 => icons_pane(ui, &mut *self.file_dialog),
                1 => plot_pane(ui),
                _ => {
                    ui.label("Resize the splitters or drag tabs to dock tiles.");
                }
            }
        });

        // The bottom button makes every pane draggable so it can be torn out
        // of its tab and docked elsewhere.
        if ui
            .add(egui::Button::new("Drag to dock").sense(egui::Sense::drag()))
            .drag_started()
        {
            egui_tiles::UiResponse::DragStarted
        } else {
            egui_tiles::UiResponse::None
        }
    }
}

/// Pane 0: demonstrates egui-phosphor icon glyphs and the file dialog.
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

/// Pane 1: demonstrates egui-plot with a continuously sampled sine wave.
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

/// Builds the initial dock tree: three tabs holding a horizontal split, a
/// grid, and a single pane, all under a shared root tab tile.
pub fn create_dock_tree() -> egui_tiles::Tree<DockPane> {
    let mut next_view_nr = 0;
    let mut gen_pane = || {
        let pane = DockPane { nr: next_view_nr };
        next_view_nr += 1;
        pane
    };

    let mut tiles = egui_tiles::Tiles::default();

    let mut tabs = vec![];
    tabs.push({
        let children = (0..3).map(|_| tiles.insert_pane(gen_pane())).collect();
        tiles.insert_horizontal_tile(children)
    });
    tabs.push({
        let cells = (0..4).map(|_| tiles.insert_pane(gen_pane())).collect();
        tiles.insert_grid_tile(cells)
    });
    tabs.push(tiles.insert_pane(gen_pane()));

    let root = tiles.insert_tab_tile(tabs);

    egui_tiles::Tree::new("main_dock", root, tiles)
}

#[cfg(test)]
mod tests {
    use super::create_dock_tree;

    #[test]
    fn dock_tree_has_multiple_tiles_and_root() {
        let tree = create_dock_tree();
        assert!(tree.root.is_some(), "dock tree should have a root tile");
        assert!(tree.tiles.len() > 1, "dock should contain multiple tiles");
    }
}
