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

- **No network access.** The app makes no network calls. Any code that
  fetches or sends data (updaters, telemetry, remote content) is new attack
  surface: review it, restrict it, and make it opt-in.
- **File handling.** The only file interaction is the native file picker
  (`egui-file-dialog`); the app logs the picked path but does not read the
  file contents. When you start reading user files, treat their contents as
  **untrusted input** — never `include!` or execute them, and never format
  attacker-controlled strings with `log` macros that interpret `{}`
  placeholders as data, not code (the `log` crate is safe by construction;
  the risk is in downstream consumers).
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

## Dependency policy

- `Cargo.lock` is committed, so builds are reproducible and dependency
  versions are auditable in the diff history.
- The egui/eframe ecosystem shares types; bump all egui crates **together**
  to a coherent version set (see [AGENTS.md](AGENTS.md)).
- Keep dependencies current: track upstream security advisories
  (`cargo audit`, GitHub Dependabot) once the project grows past the template
  stage.
- Before adding a dependency, prefer well-maintained crates with a clear
  maintenance status; each dependency expands the supply-chain surface.

## Threat model (current template)

| Asset | Trust | Notes |
| --- | --- | --- |
| User files picked in the dialog | Read-only, not processed | No reads happen yet |
| OS theme setting | Read-only, via egui/winit | Native APIs; no process spawned |
| Log records (in-memory) | Internal | Buffered in-process only; never persisted |
| Source code / build artifacts | Developer-controlled | `target/` is gitignored |

In short: the template processes **no untrusted data** today. The moment a
fork ingests external data (files, network, plugins), it must adopt an
explicit trust model — treat all external input as untrusted, validate at the
boundary, and log nothing sensitive.

## Responsible disclosure

We ask reporters to give maintainers a reasonable window to fix and release
before public disclosure. We will credit reporters (unless anonymity is
requested) and keep them informed of the fix timeline.
