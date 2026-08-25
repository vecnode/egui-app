# Architecture

This document describes how `egui_app` is put together: the runtime lifecycle,
the module structure, the rendering model, and the extension points. It is the
authoritative complement to [AGENTS.md](AGENTS.md) (workflow) and
[README.md](README.md) (user guide).

## 1. Overview

`egui_app` is a single-binary desktop application written in Rust (edition
2024). It uses the **immediate-mode** GUI paradigm: every frame, the UI code
runs from scratch and egui diffs it against the previous frame to produce
minimal repaints. There is no retained widget tree and no widget callbacks —
state lives in the application and is read/written during each frame.

The stack:

```
┌──────────────────────────────────────────────────────────────┐
│ egui_app (binary)                                            │
│  main.rs → app.rs (TemplateApp) → dock.rs (egui_tiles)       │
│              │            │                                   │
│              ├─ icon.rs (bundled app icon)                   │
│              ├─ logging.rs (egui_logger + log)                │
│              └─ persist.rs (dock layout save/restore)         │
│  build.rs — embeds assets/icon.ico into the .exe (Windows)   │
├──────────────────────────────────────────────────────────────┤
│ eframe  — windowing + event loop + renderer wiring           │
│   ├─ egui — immediate-mode UI toolkit (theming included)     │
│   ├─ winit — OS window/input abstraction (OS theme events)   │
│   └─ wgpu — GPU renderer (Vulkan/Metal/DX12)                 │
└──────────────────────────────────────────────────────────────┘
```

Crates: `eframe`, `egui_file_dialog`, `egui_phosphor`, `egui_logger`,
`egui_plot`, `egui_tiles`, `log`, `image` (icon decode),
`embed-resource` (Windows icon, build-only), `serde` + `serde_json`
(layout persistence), `directories` (per-user config dir) — see
`Cargo.toml` and the README feature table.

## 2. Runtime lifecycle

1. **`main()`** (`src/main.rs`)
   1. Installs the global logger via `logging::init_logger()`
      (`egui_logger` → captures `log` records, `Debug` level).
   2. Builds `eframe::NativeOptions` with a 1024×720 viewport and, when
      `icon::load_app_icon()` succeeds, the application icon via
      `ViewportBuilder::with_icon`.
   3. Calls `eframe::run_native(APP_TITLE, options, app_creator)`.
      The creator closure runs once, before the first frame:
      1. `install_icon_font()` adds the egui-phosphor icon glyphs to the
         font atlas.
      2. Reads the OS theme once (`ctx.system_theme()`) and applies it as
         the initial light/dark preference. egui/winit read the OS setting
         in safe Rust (native APIs on each platform); the top bar can then
         switch between light and dark explicitly.
      3. `TemplateApp::new()` loads the persisted layout
         (`persist::load_layout()`, falling back to the default dock tree on
         first run) and constructs the file dialog.
2. **Event loop** (owned by eframe; the app never sees it directly). Each
   frame eframe calls the two `eframe::App` hooks on `TemplateApp`:
   - `logic()` — frame-independent bookkeeping (currently logs a one-shot
     startup message).
   - `ui()` — draws the central dockable workspace (the Log pane included),
     then calls `ctx.request_repaint()` implicitly as needed.
3. **Shutdown** — the window closes, the loop exits, and `run_native`
   returns `Ok(())` (or an error that `main` propagates).

## 3. Module breakdown

### `src/main.rs` — entry point

Thin by design. Owns `APP_TITLE`, the `NativeOptions` (viewport + app icon),
and the one-time startup wiring (logger, fonts, initial OS theme). No UI
logic lives here.

### `src/app.rs` — application state

`TemplateApp` is the root state object and the `eframe::App`
implementation:

| Field | Purpose |
| --- | --- |
| `tree: egui_tiles::Tree<DockPane>` | The dockable layout shown in the central panel |
| `file_dialog: FileDialog` | Shared native file picker, opened from dock pane 0 |
| `locked: bool` | Layout lock state, toggled from the top bar |
| `theme_pref: ThemePreference` | Light/dark preference (never `System`); initial value from the OS theme, toggled from the top bar |
| `logged_startup: bool` | One-shot flag for the startup log message |

`ui()` composes the frame: the top bar (`top_bar()`, app label on the left,
light/dark toggle and lock/unlock toggle on the right) is drawn first, then
the `CentralPanel` — filled with the theme-aware `extreme_bg_color` backdrop
so panes (`panel_fill`) are never flat black, framed by a 1px border stroke
that adapts to the current theme — hosts the dock tree (via
`DockBehavior`), and the file dialog is advanced and polled for results
(`take_picked()`). The log viewer is not a separate window — it lives in its
own dock pane (see below).

### `src/dock.rs` — dockable workspace (egui_tiles)

- `DockPaneKind` — `Demo(usize)` (template demos) or `Log` (in-app log
  viewer). A pane's tab title and content are derived from its kind.
- `DockPane` — a pane identified by its kind.
- `DockBehavior` — implements `egui_tiles::Behavior<DockPane>`, bridging the
  dock tree with shared app state (the `FileDialog`) and the layout lock
  flag. While locked, `is_tile_draggable` and `is_container_resizable`
  return `false`, which disables pane/tab dragging and splitter resizing
  (egui_tiles gates all of those through these hooks).
- `create_dock_tree()` — builds the initial layout:

  ```
  root (tab tile)
  ├── tab "horizontal" → 2 demo panes in a horizontal split
  └── tab "Log"        → the in-app log viewer (egui_logger)
  ```

  Every pane — including the Log — renders a small Phosphor "move" handle
  (`ARROWS_OUT_CARDINAL`) in its top-right corner, 6px from the top and right
  edges, drawn with `ui.put` over the pane content. Dragging it returns
  `UiResponse::DragStarted`, so panes can be torn out and re-docked at
  runtime. The handle is hidden while the layout is locked.

Demo panes: pane 0 shows a "File dialog:" label with the folder-open button
that opens the native file picker; pane 1 shows an `egui_plot` sine wave.

### `src/icon.rs` — application icon

`include_bytes!` embeds `assets/icon.png` (512×512) into the binary;
`load_app_icon()` decodes it with the `image` crate (PNG only) into
`egui::IconData` for `ViewportBuilder::with_icon`. The same icon therefore
drives the window title bar (small) and the taskbar/dock (large) on every
platform, with no runtime file I/O. On failure it returns `None` and the app
runs with the default icon.

### `src/logging.rs` — logging

Wraps `egui_logger::builder()`. The log records are captured into an
in-memory buffer rendered by the dockable "Log" pane; nothing is written to
disk or stdout by default, which keeps the template free of side effects.

### `src/persist.rs` — layout persistence

The `egui_tiles::Tree` (pane identities + arrangement) is serialized to
pretty JSON via `serde`/`serde_json` and stored in the per-user config
directory resolved by the `directories` crate (so it works in distributed
builds, on every platform). `load_layout()` runs at startup and falls back
to `create_dock_tree()` on any error; `save_layout()` runs after every
`EditAction::TileDropped` / `TileResized`, flagged through
`DockBehavior::layout_dirty`. Persistence is best-effort: failures are
logged, never fatal.

### Theming — no custom platform code

Theme handling needs no platform-specific module: at startup `main.rs` reads
the OS light/dark setting once via `ctx.system_theme()` (egui/winit use the
native APIs on each platform — no shelling out) and applies it as the
initial preference. The top bar then toggles between light and dark by
calling `Context::set_theme`. There is deliberately no `reg.exe`-spawning or
`#[cfg]`-laden platform code anymore.

### `build.rs` — Windows resource embedding

On Windows targets (`CARGO_CFG_TARGET_OS == "windows"`, so cross-compiles
stay correct) `embed-resource` compiles `assets/windows/app.rc`, which
binds `assets/icon.ico` to the executable. Explorer and the taskbar then
show the icon for the `.exe` itself, before any window exists. On other
platforms the script is a no-op.

### `assets/` — icons and packaging

| Path | Role |
| --- | --- |
| `icon.png`, `icon-{32,128,256}.png` | Source PNGs; `icon.png` is embedded at build time |
| `icon.ico` | Windows multi-size icon (`build.rs`) |
| `icon.icns` | macOS icon (`build_app.sh --bundle`) |
| `windows/app.rc` | Windows resource script |
| `macos/Info.plist` | `.app` bundle metadata template |
| `linux/egui-app.desktop` | Desktop-entry template (install steps in comments) |
| `generate_icons.ps1` | Regenerates every icon file (System.Drawing, Windows) |

## 4. Data flow per frame

```
input (winit) → egui (immediate mode) → TemplateApp::ui()
   │                                      │
   │          ┌───────────────────────────┘
   │          ▼
   │   Panel::top "top_bar"               ← label + light/dark toggle +
   │     └─ toggles TemplateApp::theme_pref (→ ctx.set_theme) and
   │        toggles TemplateApp::locked    (drives DockBehavior::locked)
   │   CentralPanel (theme backdrop + 1px border stroke)
   │     └─ tree.ui(&mut DockBehavior, ui) ← dock panes render, incl.
   │     │     │                              the Log pane
   │     │     │                              (egui_logger::logger_ui)
   │     │     │                              move handle → drag when
   │     │     │                              unlocked
   │     │     └─ EditAction::TileDropped / TileResized → layout_dirty
   │     │          → persist::save_layout(&tree)        (JSON, config dir)
   │     └─ file_dialog.update(ui)           ← picker advances/renders
   │     └─ take_picked() → log::info!(path)
   │          │
   └──────────┘ egui outputs a paint list → wgpu draws → present
```

Because egui is immediate mode, "state" is just fields on `TemplateApp`
read/written during the frame; there are no event handlers to wire up.

## 5. Threading & platform notes

- Single-threaded UI: all rendering and logic runs on the main thread inside
  the eframe event loop. The `FileDialog` is UI-only (no background threads).
- The OS theme is delivered by winit as `ThemeChanged` events; no process is
  spawned and no registry is queried directly.
- Rendering uses wgpu, so the app is portable across Vulkan (Linux/Windows),
  Metal (macOS), and DX12 (Windows). eframe also falls back to OpenGL where
  needed.

## 6. Testing strategy

- Unit tests live in `#[cfg(test)] mod tests` inside each module
  (`dock.rs`, `persist.rs`).
- Testable seams: `create_dock_tree()` is a pure constructor — the dock test
  asserts a root tile exists and multiple tiles are present. The layout lock
  is tested through the `Behavior` hooks (`is_tile_draggable` /
  `is_container_resizable`). Persistence is tested by round-tripping a tree
  through JSON and by asserting `save_layout` never panics.
- UI rendering is intentionally **not** unit-tested (immediate-mode code is
  exercised by running the app); keep helpers pure so the important logic is
  covered.

## 7. Extension points

- **New dock pane**: add a variant to `DockPaneKind` and a branch in
  `pane_ui` (or restructure `create_dock_tree`), keeping pane content in
  small helper functions like `icons_pane`/`plot_pane`. New pane kinds are
  automatically persisted once they derive `serde`.
- **New app-wide state**: add fields to `TemplateApp`; read/write them from
  `ui()`.
- **Theme behavior**: everything goes through `egui::ThemePreference` — no
  custom platform code. Extend the light/dark toggle in `top_bar()` (e.g. a
  persistent setting loaded from disk).
- **New logging sink**: extend `logging.rs` (e.g. `env_logger` for stdout)
  without touching the app code.
- **New icon**: replace the PNGs/ICO/ICNS in `assets/` (or edit
  `generate_icons.ps1` and re-run it), then commit all regenerated files.
- **Persisted settings**: `persist.rs` is the pattern — a per-user config
  file via `directories`; add more files (e.g. `settings.json`) the same way.
- **Distribution**: `cargo build --release` (LTO + strip enabled in
  `Cargo.toml`); `build_app.bat` / `build_app.sh` wrap build + run;
  `distribute_app.bat` / `distribute_app.sh` package release artifacts with
  SHA-256 checksums (Linux tarball, macOS `.app` zip, Windows portable zip);
  signing/notarization is deferred to CI.

## 8. Reproducibility

`Cargo.lock` is committed (application, not library), so dependency versions
are pinned for every build. `rust-toolchain` is not pinned; the crate targets
stable Rust with edition 2024 (minimum 1.85), tested on 1.94.
