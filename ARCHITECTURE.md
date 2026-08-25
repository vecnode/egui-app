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
│              └─ platform.rs (Windows 11 dark theme)           │
│  build.rs — embeds assets/icon.ico into the .exe (Windows)   │
├──────────────────────────────────────────────────────────────┤
│ eframe  — windowing + event loop + renderer wiring           │
│   ├─ egui — immediate-mode UI toolkit                        │
│   ├─ winit — OS window/input abstraction                     │
│   └─ wgpu — GPU renderer (Vulkan/Metal/DX12)                 │
└──────────────────────────────────────────────────────────────┘
```

Crates: `eframe`, `egui_file_dialog`, `egui_phosphor`, `egui_logger`,
`egui_plot`, `egui_tiles`, `log`, `image` (icon decode),
`embed-resource` (Windows icon, build-only) — see `Cargo.toml` and the
README feature table.

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
      2. `platform::force_dark_theme_on_windows()` decides the theme; the
         dark theme is forced on Windows 11.
      3. `TemplateApp::new()` constructs the dock tree and file dialog.
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
and the one-time startup wiring (logger, fonts, theme). No UI logic lives
here.

### `src/app.rs` — application state

`TemplateApp` is the root state object and the `eframe::App`
implementation:

| Field | Purpose |
| --- | --- |
| `tree: egui_tiles::Tree<DockPane>` | The dockable layout shown in the central panel |
| `file_dialog: FileDialog` | Shared native file picker, opened from dock pane 0 |
| `logged_startup: bool` | One-shot flag for the startup log message |

`ui()` composes the frame: the `CentralPanel` hosts the dock tree (via
`DockBehavior`), and the file dialog is advanced and polled for results
(`take_picked()`). The log viewer is not a separate window — it lives in its
own dock pane (see below).

### `src/dock.rs` — dockable workspace (egui_tiles)

- `DockPaneKind` — `Demo(usize)` (template demos) or `Log` (in-app log
  viewer). A pane's tab title and content are derived from its kind.
- `DockPane` — a pane identified by its kind.
- `DockBehavior` — implements `egui_tiles::Behavior<DockPane>`, bridging the
  dock tree with shared app state (the `FileDialog`).
- `create_dock_tree()` — builds the initial layout:

  ```
  root (tab tile)
  ├── tab "horizontal" → 3 demo panes in a horizontal split
  ├── tab "grid"       → 4 demo panes in a grid
  ├── tab (single)     → 1 demo pane
  └── tab "Log"        → the in-app log viewer (egui_logger)
  ```

  Every pane gets a "Drag to dock" button (via `egui::Sense::drag()`), so
  panes — including the Log — can be torn out and re-docked at runtime.

Demo panes: pane 0 shows egui-phosphor icon buttons + file dialog trigger;
pane 1 shows an `egui_plot` sine wave.

### `src/icon.rs` — application icon

`include_bytes!` embeds `assets/icon.png` (512×512) into the binary;
`load_app_icon()` decodes it with the `image` crate (PNG only) into
`egui::IconData` for `ViewportBuilder::with_icon`. The same icon therefore
drives the window title bar (small) and the taskbar/dock (large) on every
platform, with no runtime file I/O. On failure it returns `None` and the app
runs with the default icon.

### `src/platform.rs` — OS-specific behavior

`force_dark_theme_on_windows()` is the only public entry point. On
non-Windows targets it is a compile-time `false` (the `#[cfg(not(target_os
= "windows"))]` arm). On Windows it reads the build number from the registry
with `reg.exe` (`HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion`
→ `CurrentBuild`); build ≥ 22000 means Windows 11, and the dark theme is
forced. An unreadable build **fails closed to dark**, matching the Windows 11
default.

This module is the template's pattern for platform-specific code: isolate it
behind a small pure-ish function with `#[cfg]` arms so the rest of the app
stays platform-agnostic.

### `src/logging.rs` — logging

Wraps `egui_logger::builder()`. The log records are captured into an
in-memory buffer rendered by the dockable "Log" pane; nothing is written to
disk or stdout by default, which keeps the template free of side effects.

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
   │   CentralPanel
   │     └─ tree.ui(&mut DockBehavior, ui)   ← dock panes render,
   │     │                                     incl. the Log pane
   │     │                                     (egui_logger::logger_ui)
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
- Platform detection spawns `reg.exe` **once per call**; it is currently
  called once at startup. If it is ever called per-frame, cache the result —
  spawning a process every frame is expensive.
- Rendering uses wgpu, so the app is portable across Vulkan (Linux/Windows),
  Metal (macOS), and DX12 (Windows). eframe also falls back to OpenGL where
  needed.

## 6. Testing strategy

- Unit tests live in `#[cfg(test)] mod tests` inside each module
  (`dock.rs`, `platform.rs`).
- Testable seams: `create_dock_tree()` is a pure constructor — the dock test
  asserts a root tile exists and multiple tiles are present. Platform code
  exposes a total function that never panics.
- UI rendering is intentionally **not** unit-tested (immediate-mode code is
  exercised by running the app); keep helpers pure so the important logic is
  covered.

## 7. Extension points

- **New dock pane**: add a variant to `DockPaneKind` and a branch in
  `pane_ui` (or restructure `create_dock_tree`), keeping pane content in
  small helper functions like `icons_pane`/`plot_pane`.
- **New app-wide state**: add fields to `TemplateApp`; read/write them from
  `ui()`.
- **New platform behavior**: add `#[cfg(...)]` functions to `platform.rs`
  behind a small public API.
- **New logging sink**: extend `logging.rs` (e.g. `env_logger` for stdout)
  without touching the app code.
- **New icon**: replace the PNGs/ICO/ICNS in `assets/` (or edit
  `generate_icons.ps1` and re-run it), then commit all regenerated files.
- **Distribution**: `cargo build --release` (LTO + strip enabled in
  `Cargo.toml`); `build_app.bat` / `build_app.sh` wrap build + run;
  `build_app.sh --bundle` produces a macOS `.app` with the icon.

## 8. Reproducibility

`Cargo.lock` is committed (application, not library), so dependency versions
are pinned for every build. `rust-toolchain` is not pinned; the crate targets
stable Rust with edition 2024 (minimum 1.85), tested on 1.94.
