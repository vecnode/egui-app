//! Dock layout persistence.
//!
//! The [`egui_tiles`] tree (which panes exist and how they are arranged,
//! split and tabbed) is saved as pretty JSON to the platform's per-user
//! config directory, and loaded back at startup. Because the location is a
//! per-user directory rather than the working directory, persistence keeps
//! working in distributed builds, on every platform.
//!
//! Typical locations of the file:
//!
//! - Windows: `%APPDATA%\vecnode\egui-app\config\layout.json`
//! - Linux:   `~/.config/vecnode/egui-app/layout.json`
//! - macOS:   `~/Library/Application Support/com.vecnode.egui-app/layout.json`
//!
//! A corrupt or unreadable file is never fatal: the app falls back to the
//! default layout and logs a warning.

use std::path::PathBuf;

use crate::dock::{DockPane, create_dock_tree};

/// Qualifier / organization / application used to derive the config dir.
const APP_QUALIFIER: &str = "com";
const APP_ORG: &str = "vecnode";
const APP_NAME: &str = "egui-app";
/// File name of the persisted layout (JSON, human-readable).
const LAYOUT_FILE: &str = "layout.json";
/// Upper bound for the layout file size. The file is user-writable config,
/// so a corrupted or hostile file must never be able to exhaust memory.
/// A real layout is a few KB; 1 MiB is far more than enough.
const MAX_LAYOUT_SIZE: u64 = 1024 * 1024;

/// Returns the path of the layout file, or `None` when no per-user config
/// directory can be determined.
pub fn layout_file_path() -> Option<PathBuf> {
    directories::ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
        .map(|dirs| dirs.config_dir().join(LAYOUT_FILE))
}

/// Loads the saved layout, falling back to [`create_dock_tree`] when no file
/// exists, it is too large, or it cannot be parsed into a valid tree.
pub fn load_layout() -> egui_tiles::Tree<DockPane> {
    let Some(path) = layout_file_path() else {
        return create_dock_tree();
    };
    // Reject oversized files before reading them (see `MAX_LAYOUT_SIZE`).
    match std::fs::metadata(&path) {
        Ok(meta) if meta.len() > MAX_LAYOUT_SIZE => {
            log::warn!(
                "ignoring oversized layout file {} ({} bytes)",
                path.display(),
                meta.len()
            );
            return create_dock_tree();
        }
        Ok(_) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return create_dock_tree(),
        Err(err) => {
            log::warn!("could not stat layout file {}: {err}", path.display());
            return create_dock_tree();
        }
    }
    match std::fs::read(&path) {
        Ok(bytes) => match serde_json::from_slice::<egui_tiles::Tree<DockPane>>(&bytes) {
            Ok(tree) if tree.root.is_some() => tree,
            Ok(_) => {
                log::warn!("saved layout {} has no root; using default", path.display());
                create_dock_tree()
            }
            Err(err) => {
                log::warn!("could not parse saved layout {}: {err}", path.display());
                create_dock_tree()
            }
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => create_dock_tree(),
        Err(err) => {
            log::warn!("could not read layout file {}: {err}", path.display());
            create_dock_tree()
        }
    }
}

/// Saves the current layout to the per-user config directory. Failures are
/// logged and never fatal (persistence is a convenience, not a requirement).
pub fn save_layout(tree: &egui_tiles::Tree<DockPane>) {
    let Some(path) = layout_file_path() else {
        log::warn!("no per-user config directory available; layout not saved");
        return;
    };
    let Some(parent) = path.parent() else {
        return;
    };
    if let Err(err) = std::fs::create_dir_all(parent) {
        log::warn!(
            "could not create config directory {}: {err}",
            parent.display()
        );
        return;
    }
    match serde_json::to_vec_pretty(tree) {
        Ok(bytes) => {
            if let Err(err) = std::fs::write(&path, bytes) {
                log::warn!("could not save layout to {}: {err}", path.display());
            }
        }
        Err(err) => log::warn!("could not serialize layout: {err}"),
    }
}

#[cfg(test)]
mod tests {
    use super::save_layout;

    /// A layout must round-trip through JSON without losing panes or the
    /// root tile, which is what `load_layout` relies on.
    #[test]
    fn layout_round_trips_through_json() {
        let tree = super::load_layout();
        let bytes = serde_json::to_vec(&tree).expect("layout should serialize");
        let restored: egui_tiles::Tree<super::DockPane> =
            serde_json::from_slice(&bytes).expect("layout should deserialize");

        assert_eq!(tree.root, restored.root, "root tile should survive");
        assert_eq!(
            tree.tiles.len(),
            restored.tiles.len(),
            "panes should survive"
        );
    }

    /// Saving must never panic, even when the config directory is unusable
    /// (e.g. in a sandboxed test environment).
    #[test]
    fn save_layout_is_infallible() {
        save_layout(&super::load_layout());
    }
}
