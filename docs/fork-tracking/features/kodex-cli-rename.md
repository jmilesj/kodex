# Kodex CLI Rename, Telemetry Disablement, And Update Disablement

## Purpose

This fork installs and exposes the top-level CLI as `kodex` so it can coexist with upstream `codex` on the same machine. It still shares upstream Codex state such as `~/.codex`, `CODEX_HOME`, auth, settings, skills, plugins, and history.

The fork also disables telemetry, analytics, and update checks so it does not send analytics or prompt for upstream updates.

## Upstream Anchor

Upstream owns the corresponding `codex` command, npm package wiring, native binary packaging, install scripts, telemetry initialization, analytics event client, TUI update prompt, and app-server daemon update loop.

Primary upstream areas to inspect after merges:

- `codex-rs/cli/`
- `codex-cli/`
- `scripts/install/`
- `scripts/stage_npm_packages.py`
- `codex-rs/analytics/`
- `codex-rs/core/src/otel_init.rs`
- `codex-rs/tui/src/updates.rs`
- `codex-rs/tui/src/update_action.rs`
- `codex-rs/app-server-daemon/src/update_loop.rs`

## Local Implementation Paths

- `codex-rs/cli/Cargo.toml`
- `codex-rs/cli/BUILD.bazel`
- `codex-rs/cli/src/main.rs`
- `codex-rs/cli/tests/`
- `codex-cli/package.json`
- `codex-cli/bin/kodex.js`
- `codex-cli/scripts/build_npm_package.py`
- `codex-cli/scripts/install_native_deps.py`
- `scripts/install/install.sh`
- `scripts/install/install.ps1`
- `scripts/stage_npm_packages.py`
- `codex-rs/analytics/src/client.rs`
- `codex-rs/core/src/otel_init.rs`
- `codex-rs/tui/src/updates.rs`
- `codex-rs/tui/src/update_action.rs`
- `codex-rs/app-server-daemon/src/update_loop.rs`

## Verification Steps

Run these after any upstream merge that touches the upstream anchor or local implementation paths:

```bash
node --check codex-cli/bin/kodex.js
python3 -m py_compile codex-cli/scripts/build_npm_package.py scripts/stage_npm_packages.py
bash -n scripts/install/install.sh
cd codex-rs
just test -p codex-cli
```

When telemetry, analytics, or updater code changed upstream, also run:

```bash
cd codex-rs
just test -p codex-analytics
just test -p codex-app-server-daemon
just test -p codex-tui
```

Manual checks:

- CLI help and examples use `kodex` for the user-facing command.
- npm package `bin` exposes `kodex`, not `codex`.
- package staging expects a `kodex` native executable.
- `scripts/install/install.ps1` syntax and rename references still target `kodex`.
- update commands, background update checks, and app-server updater loops do not contact upstream update services.
- analytics and OTEL initialization stay disabled.

## Merge Risks

- Upstream can add new `codex` command references in packaging or docs that need to stay renamed in this fork.
- Upstream can add new telemetry or analytics entry points outside the existing disabled paths.
- Upstream can change updater code in the TUI or app-server daemon and reintroduce network checks.
- Upstream can add native package artifacts or install scripts that assume the binary name is `codex`.

## Status

`needs review`

## Last Verified

Partial review after upstream merge `090144e0ec`; telemetry/updater scoped tests are still pending.

## Retirement Condition

Retire this feature only if the fork intentionally stops using the `kodex` command name and stops disabling telemetry/update behavior, or if upstream provides an accepted mechanism that fully replaces these fork changes.
