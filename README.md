# egui-app

A cross-platform desktop application template built with [egui](https://github.com/emilk/egui) and [eframe](https://github.com/emilk/egui/tree/master/crates/eframe).

## Features


- Immediate-mode GUI + windowing | [`eframe`](https://crates.io/crates/eframe) / [`egui`](https://crates.io/crates/egui) | Whole app 
- Dockable, resizable panes | [`egui_tiles`](https://crates.io/crates/egui_tiles) | "Dock pane" tabs (horizontal splits, grids) 
- Line / scatter plots | [`egui_plot`](https://crates.io/crates/egui_plot) | Pane 1: `sin(x)` 
- Native file picker | [`egui-file-dialog`](https://crates.io/crates/egui-file-dialog) | Pane 0: folder-open button 
- Icon glyph font | [`egui-phosphor`](https://crates.io/crates/egui-phosphor) | Pane 0 buttons 
- In-app log viewer | [`egui_logger`](https://crates.io/crates/egui_logger) | Floating "Log" window 
- Structured logging | [`log`](https://crates.io/crates/log) | `log::info!` everywhere 

Platforms:

- **Windows 11 dark theme** — the app forces egui's dark theme on Windows 11 so it
  matches the OS default (see [`src/platform.rs`](src/platform.rs)).
- **Linux / macOS** — builds and runs unmodified with the same `cargo` commands.

## Prerequisites

- **Rust** (stable, edition 2024; tested with 1.94). Install via [rustup](https://rustup.rs/).
- **Linux only** — the system libraries required by `winit`/`wgpu`. On Debian/Ubuntu:

  ```sh
  sudo apt install libxcb1-dev libxkbcommon-dev libgtk-3-dev
  ```

  Other distributions have equivalent packages; see the
  [eframe template](https://github.com/emilk/egui_template) for details.

## Quick start

```sh
# Development build + run (fast, unoptimized) -> target/debug/
cargo run

# Distribution build -> target/release/
cargo build --release
```

### One-command scripts

The repository ships convenience scripts that build **and** run the app:

```sh
# Windows (cmd or PowerShell)
build_app.bat                # debug build + run
build_app.bat --release      # release build + run
build_app.bat --build-only   # build without launching

# Linux / macOS
./build_app.sh
./build_app.sh --release
./build_app.sh --build-only
```

## Repository

- `src/main.rs` — entry point: logging, fonts, theming, `eframe::run_native`.
- `src/app.rs` — `TemplateApp`: application state and the per-frame UI.
- `src/dock.rs` — the dockable workspace; add your own panes here.
- `src/platform.rs` — OS-specific detection (Windows 11 dark theme).
- `src/logging.rs` — logger installation.

See [ARCHITECTURE.md](ARCHITECTURE.md) for a full tour of the codebase.

## Development workflow

```sh
cargo run          # run in debug mode
cargo test         # unit tests
cargo clippy       # lint (no warnings expected)
cargo fmt          # format the code
cargo doc --open   # local documentation
```

Contributors and AI agents: please read [AGENTS.md](AGENTS.md) first.

## Security

See [SECURITY.md](SECURITY.md) for the supported-versions policy and how to report
a vulnerability.

## License

Licensed under [MIT](LICENSE), © 2026 vecnode.
