# Security Policy

This document describes the security posture of the `egui-app` template and
how to report vulnerabilities. The project is a **template**: it ships as
source code and is intended to be copied and extended. Treat the guidance
below as the baseline you inherit when you fork it.

## Supported versions

| Version | Supported |
| --- | --- |
| `main` (development) | ✅ Bug and security fixes land here |
| Latest tagged release | ✅ Critical fixes only |
| Older releases | ❌ Not supported — upgrade |

There are no formal releases yet (`0.1.0`); until the first tag, `main` is the
only supported line.

## Reporting a vulnerability

**Do not open a public issue for security problems.** Instead:

1. Email the maintainers privately, or
2. Use the repository's private vulnerability reporting feature if enabled
   on your hosting platform (e.g. GitHub's *Security → Report a
   vulnerability*).

Include:

- Affected version(s) / commit hash,
- A minimal reproduction (steps or a small patch),
- Impact assessment and any suggested fix, if known.

The maintainers will acknowledge within **7 days** and will coordinate a
disclosure timeline with you (default: 90 days from confirmation, or sooner
if a fix is ready).

## Security considerations for this template

This is a local desktop GUI application. Its current attack surface is small,
but every fork changes that — re-read this section when you add features.

- **Memory safety.** The crate declares `#![forbid(unsafe_code)]` — it
  contains no `unsafe` blocks, and the lint guarantees that stays true for
  every future change. The Rust dependency tree still contains `unsafe`
  internally (that is normal), which is why keeping dependencies current
  matters (see "Dependency policy").
- **No network access.** The app makes no network calls. Any code that
  fetches or sends data (updaters, telemetry, remote content) is new attack
  surface: review it, restrict it, and make it opt-in.
- **File handling.** The only interactive file operation is the native file
  picker (`egui-file-dialog`); the app logs the picked path but does not read
  the file contents. When you start reading user files, treat their contents
  as **untrusted input** — never `include!` or execute them, and never format
  attacker-controlled strings with `log` macros that interpret `{}`
  placeholders as data, not code (the `log` crate is safe by construction;
  the risk is in downstream consumers).
- **Persisted layout is untrusted.** `layout.json` lives in the user-writable
  config directory, so it must never be trusted: reads are size-capped
  (1 MiB), the JSON is parsed by `serde_json` (which enforces a recursion
  limit), and the result is validated (a root tile must exist). Any failure
  falls back to the default layout — a corrupted or hostile file can at most
  reset the layout, never crash or exhaust memory.
- **Bounded logging.** The in-app log buffer retains at most 1000 entries
  (egui_logger `max_log_length`), so log spam cannot grow memory unbounded.
  Log records are in-memory only and never persisted.
- **No process spawning.** Nothing in the app shells out to external
  programs. The OS theme is read by egui/winit through the native APIs (no
  `reg.exe`, no `defaults`, no `gsettings`). Keep it that way: never
  interpolate user-controlled strings into any `Command` invocation.
- **Secrets.** None are stored or expected. If a fork adds credentials, store
  them via the OS keychain or a dedicated secrets API — never in source, logs,
  or the egui log buffer (the in-app "Log" window can capture them).
- **MIT license.** The license is not a security control, but it permits
  unrestricted use and proprietary derivatives — review what you distribute and
  keep the copyright notice intact when redistributing the code.

## Distribution hardening

- `distribute_app.bat` / `distribute_app.sh` produce archives that contain
  **only** the executable, the LICENSE and the README — no user data, no
  config files, no build artifacts.
- Every archive ships with a **SHA-256 checksum** (`.sha256`) so users can
  verify download integrity.
- **Binaries are unsigned.** The Windows `.exe` and the macOS `.app` have no
  code-signing certificate in this repository, so SmartScreen / Gatekeeper
  may warn on first run. Signing (Windows Authenticode; macOS codesign +
  notarization) is planned for CI and is a prerequisite for broad
  distribution. Linux binaries are not signed by default; prefer
  distribution via your distro's package manager for trusted updates.
- **Reproducible-ish builds.** `Cargo.lock` is committed and the release
  profile enables LTO + strip; cargo verifies crate integrity against the
  registry index at build time. Pin the toolchain in CI later for bit-exact
  reproducibility.
- **Release checklist** (before tagging): `cargo test`, `cargo clippy
  --all-targets` clean, `cargo fmt --check` clean, `cargo audit` reports no
  known vulnerabilities, and `distribute_app.*` succeeds with checksums.

## Dependency policy

- `Cargo.lock` is committed, so builds are reproducible and dependency
  versions are auditable in the diff history.
- The egui/eframe ecosystem shares types; bump all egui crates **together**
  to a coherent version set (see [AGENTS.md](AGENTS.md)).
- Run `cargo audit` against the RustSec advisory database before tagging a
  release (automate it in CI later), and keep dependencies current: track
  upstream security advisories and Dependabot-style notifications.
- **Audit status (current):** `cargo audit` reports **0 known
  vulnerabilities**. One maintenance advisory is tracked: `ttf-parser`
  (transitive via `ab_glyph`, used by egui-winit/egui-wgpu for font
  rasterization) is unmaintained but has **no known vulnerabilities**; it is
  re-checked on every audit and will be dropped when the egui stack moves
  off it.
- Before adding a dependency, prefer well-maintained crates with a clear
  maintenance status; each dependency expands the supply-chain and `unsafe`
  surface.

## Threat model (current template)

| Asset | Trust | Notes |
| --- | --- | --- |
| User files picked in the dialog | Read-only, not processed | No reads happen yet |
| OS theme setting | Read-only, via egui/winit | Native APIs; no process spawned |
| Persisted layout (`layout.json`) | Untrusted, user-writable | Size-capped, serde-parsed, validated; falls back on any error |
| Log records (in-memory) | Internal | Bounded buffer (1000); never persisted |
| Source code / build artifacts | Developer-controlled | `target/` and `dist/` are gitignored |

In short: the template processes **no meaningful untrusted data** today — the
only untrusted file is the user's own layout config, which is bounded and
validated at the boundary. The moment a fork ingests external data (files,
network, plugins), it must adopt an explicit trust model — treat all external
input as untrusted, validate at the boundary, and log nothing sensitive.

## Responsible disclosure

We ask reporters to give maintainers a reasonable window to fix and release
before public disclosure. We will credit reporters (unless anonymity is
requested) and keep them informed of the fix timeline.
