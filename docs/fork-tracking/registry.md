# Fork Feature Registry

This registry is the source of truth for fork-only behavior. Every fork-only feature should appear here exactly once.

| Feature | Status | Upstream anchor | Local paths | Verification target | Last verified against | Note |
| --- | --- | --- | --- | --- | --- | --- |
| Kodex CLI rename, telemetry disablement, and update disablement | `needs review` | Upstream top-level `codex` CLI, npm packaging, install scripts, telemetry initialization, analytics client, and update checks | `codex-rs/cli/`, `codex-cli/`, `scripts/install/`, `scripts/stage_npm_packages.py`, `codex-rs/analytics/`, `codex-rs/core/src/otel_init.rs`, `codex-rs/tui/src/updates.rs`, `codex-rs/tui/src/update_action.rs`, `codex-rs/app-server-daemon/src/update_loop.rs` | `just test -p codex-cli`; package script syntax checks; telemetry and updater crate tests when touched | Partial review after `090144e0ec`; telemetry/updater scoped tests pending | [`features/kodex-cli-rename.md`](features/kodex-cli-rename.md) |
| Project-local auth files | `needs review` | Upstream global auth/config loading and login/logout storage behavior | `codex-rs/core/src/config/`, `codex-rs/login/src/auth/`, `codex-rs/cli/src/login.rs`, `codex-rs/exec/src/lib.rs`, `codex-rs/tui/src/lib.rs`, `codex-rs/cloud-requirements/src/lib.rs` | `just test -p codex-core`; `just test -p codex-login`; `just test -p codex-cli` | Pending review after `090144e0ec` | [`features/project-local-auth.md`](features/project-local-auth.md) |
| Fork release pipeline, installer bootstrap, and release versioning | `active` | Upstream release workflow, GitHub Release publishing, installer bootstrap, CLI version stamping behavior, and shipped CLI dependency graph | `.github/workflows/release.yml`, `.github/actions/setup-rusty-v8/action.yml`, `.github/scripts/`, `scripts/install/install.sh`, `README.md`, `MODULE.bazel`, `defs.bzl`, `codex-rs/Cargo.toml`, `codex-rs/Cargo.lock`, `codex-rs/cli/src/main.rs`, `codex-rs/cli/src/version.rs`, `codex-rs/tui/` | `.github/scripts/test_cli_manifest.py`; `.github/scripts/test_release_workflow.py`; `.github/scripts/test_install_sh.py`; `bash -n scripts/install/install.sh`; `ruby -e 'require "yaml"; YAML.load_file(".github/workflows/release.yml"); puts "ok"'`; `cd codex-rs && KODEX_CLI_VERSION=0.133.0.1779638524 just test -p codex-cli version_uses_kodex_command_name version::tests` | `090144e0ec` | [`features/fork-release-pipeline.md`](features/fork-release-pipeline.md) |

## Update Rules

- Keep one row per feature.
- Use `needs review` when a feature has not been checked after an upstream merge.
- Use `active` only after the verification target has been run or manually checked and recorded in `merge-audit.md`.
- Use `degraded` when a feature still exists but has a known regression.
- Use `retired` when the fork no longer needs the behavior.
- Update `Last verified against` with the upstream merge commit or date from the audit entry.
