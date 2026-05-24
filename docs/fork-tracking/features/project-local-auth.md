# Project-Local Auth Files

## Purpose

This fork supports project-local `.codex/auth.json` files that take precedence over the configured global persistent auth store. That lets a project carry explicit local credentials without replacing the user's global Codex auth.

The feature threads project auth directories through config loading, login status, logout behavior, TUI startup, exec startup, cloud requirements, and auth manager refresh behavior.

## Upstream Anchor

Upstream owns the global auth store, config layer stack, login/logout commands, TUI startup auth wiring, exec startup auth wiring, and cloud auth requirements.

Primary upstream areas to inspect after merges:

- `codex-rs/core/src/config/`
- `codex-rs/login/src/auth/`
- `codex-rs/cli/src/login.rs`
- `codex-rs/exec/src/lib.rs`
- `codex-rs/tui/src/lib.rs`
- `codex-rs/cloud-requirements/src/lib.rs`
- `codex-rs/cloud-tasks/src/util.rs`

## Local Implementation Paths

- `codex-rs/core/src/config/mod.rs`
- `codex-rs/core/src/config/config_tests.rs`
- `codex-rs/login/src/auth/manager.rs`
- `codex-rs/login/src/auth/storage.rs`
- `codex-rs/login/src/auth/auth_tests.rs`
- `codex-rs/login/src/auth/storage_tests.rs`
- `codex-rs/cli/src/login.rs`
- `codex-rs/cli/tests/login.rs`
- `codex-rs/exec/src/lib.rs`
- `codex-rs/tui/src/lib.rs`
- `codex-rs/tui/src/onboarding/auth.rs`
- `codex-rs/cloud-requirements/src/lib.rs`
- `codex-rs/cloud-tasks/src/util.rs`

## Verification Steps

Run these after any upstream merge that touches the upstream anchor or local implementation paths:

```bash
cd codex-rs
just test -p codex-core
just test -p codex-login
just test -p codex-cli
```

When TUI, exec, cloud requirements, or cloud task auth wiring changed upstream, also run the matching scoped tests:

```bash
cd codex-rs
just test -p codex-tui
just test -p codex-exec
just test -p codex-cloud-requirements
just test -p codex-cloud-tasks
```

Targeted tests to look for in the output:

- `project_auth_dirs_include_project_codex_dirs_nearest_first`
- `config_toml_load_returns_project_auth_dirs`
- `load_persistent_auth_prefers_first_project_auth_file`
- `load_persistent_auth_falls_back_to_global_when_project_auth_is_missing`
- `project_chatgpt_auth_refresh_persists_to_project_auth_json`
- `login_status_prefers_project_auth_json_over_global_auth_json`
- `logout_removes_project_auth_json_without_removing_global_auth_json`

Manual checks:

- Project auth directories are derived from project config layers nearest-first.
- Project-local auth takes precedence over global auth when present.
- Missing project auth falls back to the configured global store.
- Logout removes the active project auth file without deleting unrelated global auth.
- TUI and exec startup pass the same project auth directories into auth loading.

## Merge Risks

- Upstream can change config layer ordering and break nearest-first project auth precedence.
- Upstream can change auth manager loading or caching and bypass project auth directories.
- Upstream can change login/logout behavior and remove the project-local deletion path.
- Upstream can add new auth consumers that still read only the global auth store.

## Status

`needs review`

## Last Verified

Pending review after upstream merge `d1d1df1dd7`.

## Retirement Condition

Retire this feature only if the fork intentionally stops supporting project-local auth files, or if upstream adds equivalent project-scoped auth behavior and this fork can delete its custom implementation.
