# egui-app

A cross-platform desktop application template built with [egui](https://github.com/emilk/egui) and [eframe](https://github.com/emilk/egui/tree/master/crates/eframe).

## Features


- Immediate-mode GUI + windowing - [`eframe`](https://crates.io/crates/eframe) / [`egui`](https://crates.io/crates/egui)
- Dockable, resizable panes - [`egui_tiles`](https://crates.io/crates/egui_tiles) (Phosphor move handle in each pane; splitters, tabs)
- Layout lock — top-bar lock/unlock button (right side) freezes pane dragging and resizing
- Automatic OS theme — light/dark follows the system setting (safe Rust via egui/winit); top-bar toggle cycles system → light → dark
- Line / scatter plots - [`egui_plot`](https://crates.io/crates/egui_plot)
- Native file picker - [`egui-file-dialog`](https://crates.io/crates/egui-file-dialog) 
- Icon glyph font - [`egui-phosphor`](https://crates.io/crates/egui-phosphor)
- In-app log viewer - [`egui_logger`](https://crates.io/crates/egui_logger) 
- Structured logging - [`log`](https://crates.io/crates/log) 
- Bundled app icon - [`image`](https://crates.io/crates/image) + [`embed-resource`](https://crates.io/crates/embed-resource)

Platforms:

- **Theme** — the app follows the OS light/dark setting automatically, live
  (egui [`ThemePreference::System`](https://docs.rs/egui/latest/egui/enum.ThemePreference.html);
  egui/winit read it through the native APIs on each platform — no shelling
  out). Override anytime from the top bar.
- **Windows** — `build.rs` embeds `assets/icon.ico` into the `.exe`.
- **Linux** — window/taskbar icon via the bundled PNG; desktop-entry template in
  `assets/linux/`.
- **macOS** — window/dock icon at runtime; `build_app.sh --bundle` produces a
  proper `.app` bundle with `assets/icon.icns` and `Info.plist`.
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
./build_app.sh --bundle         # macOS only: also create a .app bundle with the icon
```

## Repository

- `src/main.rs` — entry point: logging, fonts, icon, OS-theme preference, `eframe::run_native`.
- `src/app.rs` — `TemplateApp`: application state, top bar (theme + lock), per-frame UI.
- `src/dock.rs` — the dockable workspace (demo panes + the Log pane); add your own panes here.
- `src/icon.rs` — decodes the embedded `assets/icon.png` for the window icon.
- `src/logging.rs` — logger installation.
- `build.rs` — embeds `assets/icon.ico` into the Windows executable.

See [ARCHITECTURE.md](ARCHITECTURE.md) for a full tour of the codebase.


## Assets & icons

`assets/` holds the application icon and platform packaging files:

| File | Purpose |
| --- | --- |
| `icon.png` (512), `icon-32/128/256.png` | Master PNGs; the 512 is embedded in the binary and used as the window/taskbar icon on Linux |
| `icon.ico` | Windows multi-size icon, embedded into `egui_app.exe` by `build.rs` |
| `icon.icns` | macOS icon, used by the `.app` bundle (`build_app.sh --bundle`) |
| `windows/app.rc` | Windows resource script referenced by `build.rs` |
| `macos/Info.plist` | Bundle metadata template for macOS |
| `linux/egui-app.desktop` | Desktop-entry template (see comments inside for install steps) |
| `generate_icons.ps1` | Regenerates the whole icon set — run it and commit the new PNGs/ICO/ICNS |

The icon is deliberately a solid black square (one color, no transparency) —
swap the files and re-run the generator if you want a different look.

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
