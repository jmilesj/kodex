# Kodex CLI Rename and Telemetry Disablement Design

## Objective

Rename this fork's user-facing Codex CLI command to `kodex` so it can be installed on machines that also have upstream `codex`. The fork must continue using the existing global Codex state, including `~/.codex`, `CODEX_HOME`, auth, settings, skills, plugins, and history. The fork must also disable telemetry, analytics, and update checking.

## Scope

The rename applies to installed entry points and packaging surfaces that would collide with upstream Codex:

- the Rust CLI binary name
- the Bazel CLI binary target
- the CLI help usage command name
- npm package `bin` mapping and launcher
- npm native packaging/install script references to the bundled top-level CLI binary
- tests and helpers that resolve the top-level CLI executable

The rename does not apply to shared storage or internal implementation names:

- `CODEX_HOME` remains supported
- `~/.codex` remains the default home
- Rust crate names remain `codex-*`
- helper binaries such as `codex-exec`, `codex-tui`, `codex-command-runner`, and `codex-windows-sandbox-setup` remain unchanged unless a packaging path requires invoking the top-level CLI
- config schema, auth files, skill folders, plugin folders, and session state remain compatible with upstream Codex

## Architecture

The top-level Rust executable is renamed from `codex` to `kodex` in `codex-rs/cli`. The clap parser uses `kodex` for help and usage text, while subcommands and internal APIs remain unchanged. Existing flows such as `kodex exec`, `kodex app-server`, `kodex login`, and `kodex mcp` should behave as the corresponding upstream `codex` commands, except for disabled telemetry and updates.

The npm launcher is renamed so global npm installs expose `kodex` instead of `codex`. Platform package metadata and native binary staging are updated so the launcher looks for a `kodex` native executable in the same vendor layout. Helper binaries that do not conflict with upstream's top-level command remain as they are.

The app-server daemon and related child-process launchers continue to invoke the same executable path they are given. Where they construct command arguments, only subcommands are passed, so no storage rename is needed.

## Telemetry and Analytics

Telemetry is disabled centrally instead of removing event types throughout the codebase. The design disables outbound analytics and OTEL provider construction at initialization boundaries:

- TUI startup does not build an OTEL provider, record process-start metrics, or install sqlite telemetry.
- app-server startup does not build an OTEL provider, record process-start metrics, or install sqlite telemetry.
- app-server analytics defaults are forced off even if `--analytics-default-enabled` is passed.
- analytics event clients use the disabled/no-op path for runtime events.

This keeps internal event plumbing intact for compile-time compatibility while preventing outbound telemetry behavior in the fork.

## Update Checks

Update behavior is disabled at the mechanisms that initiate network activity or prompt users:

- the TUI startup update prompt/check returns no update.
- background version refresh is not spawned.
- the CLI `update` command is hidden or returns a clear disabled message.
- the app-server daemon updater loop exits without fetching or installing standalone updates.
- managed app-server refresh paths do not re-exec an updater.

No replacement update mechanism is added.

## Error Handling

The `kodex` rename should not change existing command error behavior. Missing native npm artifacts should mention reinstalling the fork package rather than upstream `@openai/codex`. Disabled update commands should fail or exit with a clear local message and must not contact GitHub, npm, Homebrew, or `chatgpt.com`.

Telemetry disablement should be silent during normal startup. User-specified OTEL or analytics config may still parse, but it must not create exporters or send analytics.

## Testing

Focused verification should cover:

- `cargo test -p codex-cli` for CLI parsing, help usage, top-level binary resolution, and update command behavior.
- package metadata/script tests or direct script inspection for `kodex` npm bin/native binary wiring.
- tests for central telemetry disablement, preferably at provider/client initialization boundaries.
- tests for update-check disablement in the TUI and app-server daemon surfaces touched.

After Rust changes, run `just fmt` in `codex-rs`. Run scoped `just fix -p <crate>` for changed crates before finalizing. If changes touch shared crates such as `core`, ask before a complete workspace test run.

## Non-Goals

This change does not rebrand every visible `Codex` string. It does not introduce a new config home, new auth store, new skill store, or new plugin namespace. It does not delete telemetry crates or large internal telemetry data types.
