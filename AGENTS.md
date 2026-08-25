# AGENTS.md

Guidance for AI coding agents and human contributors working in this repository.
Read this before making changes. It is intentionally short — the rest of the
context lives in [ARCHITECTURE.md](ARCHITECTURE.md).

## What this repository is

A cross-platform desktop application template built on **egui/eframe** (Rust,
edition 2024). The app is a small demo workspace: dockable panes, a plot, a
file picker, icon glyphs, and an in-app log window. It exists to be copied and
extended into real applications, so prefer clarity and idiomatic egui over
cleverness.

## Commands (run from the repository root)

| Command | Purpose |
| --- | --- |
| `cargo run` | Build (debug) and run the app |
| `cargo test` | Run unit tests (all pass, must stay green) |
| `cargo clippy --all-targets` | Lint — must produce **zero warnings** |
| `cargo fmt` / `cargo fmt --check` | Format — keep the tree formatted |
| `build_app.bat` / `build_app.sh` | One-command build + run wrappers |
| `cargo doc --open` | Local API documentation |

## Code layout

```
src/
  main.rs      # entry point: logger, fonts, icon, theme, eframe::run_native
  app.rs       # TemplateApp state + eframe::App impl (per-frame UI)
  dock.rs      # egui_tiles dockable workspace: demo panes + Log pane
  icon.rs      # bundled app icon (assets/icon.png) -> window/taskbar icon
  platform.rs  # OS detection (Windows 11 dark theme)
  logging.rs   # egui_logger installation
assets/        # icons (png/ico/icns), app.rc, Info.plist, .desktop, generator
build.rs       # embeds assets/icon.ico into the Windows .exe
```

`main.rs` is deliberately thin. Application logic lives in the modules, and the
`eframe::App` split is `logic` (frame-independent bookkeeping) vs `ui`
(drawing). New features should be added as new modules or new dock panes in
`dock.rs`, not by growing `main.rs`.

## Conventions

- **Formatting**: rustfmt defaults, enforced via `cargo fmt --check`.
- **Linting**: clippy must be clean (`cargo clippy --all-targets`).
- **Docs**: public items carry `///` doc comments; the crate enables
  `#![warn(missing_docs)]`. Document *why* a platform branch exists.
- **Naming**: follow egui conventions — `snake_case` for functions/variables,
  `CamelCase` for types. Window titles and log messages are plain English,
  lowercase after the first word ("picked file: ...").
- **Errors**: `eframe::Result` bubbles from `main`. Avoid panics; use
  `log::error!` and graceful fallbacks where reasonable.
- **Dependencies**: keep the dependency list small and explain new crates in
  the README feature table. Pin exact versions only when a newer one breaks the
  build; prefer caret requirements.
- **Tests**: put unit tests in a `#[cfg(test)] mod tests` at the bottom of the
  module they test. UI code is not unit-tested; keep pure helpers (tree
  construction, detection logic) testable.

## Constraints & cautions

- **Do not** restructure the module layout or rename the crate without updating
  `Cargo.toml`, `README.md`, `ARCHITECTURE.md`, and both `build_app.*` scripts
  (they hardcode the binary name `egui_app`).
- **Do not** bump `eframe`/`egui`-ecosystem versions casually: the egui crates
  share `egui` types, so a mixed-version dependency graph fails to compile.
  Bump the whole ecosystem together and run `cargo test` + `cargo clippy`.
- **Cargo.lock is committed** — this is an application, so lock the
  dependency graph. Regenerate it deliberately (`cargo update`) and review the
  diff.
- **Platform code**: `platform.rs` uses `#[cfg(target_os = "windows")]`
  heavily. When touching it, make sure non-Windows builds still compile — the
  `#[cfg(not(target_os = "windows"))]` fallback must always exist.
- **build.rs / icons**: `build.rs` embeds `assets/icon.ico` only for Windows
  targets (via `CARGO_CFG_TARGET_OS`) — never add unconditional resource
  compilation. Icon changes must keep the PNG/ICO/ICNS in sync; regenerate
  with `assets/generate_icons.ps1` and commit every output file.
- **Do not** commit secrets, user data, or build artifacts (`target/`).
- **Keep the demo honest**: panes exist to demonstrate a crate. When a pane's
  purpose is done, remove the demo, not the dependency list.

## Definition of done

A change is done when:

1. `cargo test` passes,
2. `cargo clippy --all-targets` reports no warnings,
3. `cargo fmt --check` is clean,
4. the app builds with `build_app.bat` / `build_app.sh` (or the equivalent
   `cargo` command on your host),
5. docs affected by the change (`README.md`, `ARCHITECTURE.md`, module doc
   comments) are updated.
