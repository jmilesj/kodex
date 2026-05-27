# Fork Merge Audit

This file is append-only. Add one entry after each upstream merge.

Each entry should record the upstream merge, the fork features reviewed, the evidence used, and any follow-up work.

## 2026-05-24 - Upstream Merge `d1d1df1dd7`

- Merge commit: `d1d1df1dd71f187d8ed4fb95f4da6ab660614aea`
- Local pre-merge parent: `bd66099c4f1b8b088534caaa18a5accbd06c7ae0`
- Upstream parent: `7d47056ea42636271ac020b86347fbbef49490aa`
- Merge message: `Merge remote-tracking branch 'upstream/main'`

### Conflict Areas

The merge commit recorded conflicts in these areas that overlap fork-only behavior:

- CLI rename and package wiring: `README.md`, `codex-cli/bin/kodex.js`, `codex-cli/scripts/build_npm_package.py`, `codex-cli/scripts/install_native_deps.py`, `scripts/install/install.sh`, `scripts/install/install.ps1`, `scripts/stage_npm_packages.py`, `sdk/typescript/src/exec.ts`
- CLI runtime behavior: `codex-rs/cli/src/main.rs`
- Telemetry and analytics: `codex-rs/analytics/src/client_tests.rs`
- Project-local auth and config loading: `codex-rs/core/src/config/mod.rs`, `codex-rs/login/src/auth/manager.rs`, `codex-rs/tui/src/lib.rs`
- TUI and update behavior: `codex-rs/tui/src/update_action.rs`, `codex-rs/tui/src/chatwidget.rs`, `codex-rs/tui/src/app/tests.rs`

### Feature Review

| Feature | Result | Evidence | Follow-up |
| --- | --- | --- | --- |
| Kodex CLI rename, telemetry disablement, and update disablement | `needs review` | Merge conflicts overlapped `README.md`, `codex-cli/bin/kodex.js`, `codex-cli/scripts/build_npm_package.py`, `codex-cli/scripts/install_native_deps.py`, `scripts/install/install.sh`, `scripts/install/install.ps1`, `scripts/stage_npm_packages.py`, `sdk/typescript/src/exec.ts`, `codex-rs/cli/src/main.rs`, `codex-rs/analytics/src/client_tests.rs`, `codex-rs/tui/src/update_action.rs`, `codex-rs/tui/src/chatwidget.rs`, and `codex-rs/tui/src/app/tests.rs`. The shell, Python, and Node syntax checks exercised the packaging side of that overlap, but the feature still needs its own behavioral verification pass. | Run the verification steps in `features/kodex-cli-rename.md`, then update `registry.md` and this audit entry's follow-up status in a later entry. |
| Project-local auth files | `needs review` | Merge conflicts overlapped `codex-rs/core/src/config/mod.rs`, `codex-rs/login/src/auth/manager.rs`, and `codex-rs/tui/src/lib.rs`. The feature-specific auth behavior still needs its own verification pass, which is tracked in the feature note. | Run the verification steps in `features/project-local-auth.md`, then update `registry.md` and this audit entry's follow-up status in a later entry. |

### Supporting Checks

- `bash -n scripts/install/install.sh`: passed.
- `python3 -m py_compile codex-cli/scripts/build_npm_package.py scripts/stage_npm_packages.py`: passed.
- `node --check codex-cli/bin/kodex.js`: passed.
- `cargo fmt -- --config imports_granularity=Item`: passed after merge cleanup.
- TUI crate test run failed in `app::tests::discard_side_thread_removes_agent_navigation_entry` with a stack overflow. This failure was not attributed to a fork-tracked feature during the merge.

### Outcome

The upstream merge completed, but both tracked fork features remain `needs review` until their feature-specific verification steps are run and recorded.

## 2026-05-28 - Upstream Merge `090144e0ec`

- Merge commit: this merge resolution commit.
- Local pre-merge parent: `869118ed022b9c8ebb0028d71c2075a9f87a1ed6`
- Upstream parent: `090144e0eca3978b3ebf29bc376a48b3d37523c5`
- Merge message: `Merge remote-tracking branch 'upstream/main'`

### Conflict Areas

The merge recorded conflicts or fork-review decisions in these areas that overlap fork-only behavior:

- CLI rename, package wiring, and user-facing command text: `codex-rs/cli/`, `codex-rs/utils/cli/src/resume_command.rs`, `codex-rs/tui/`, `codex-cli/`, and `scripts/install/install.sh`
- Fork release and installer behavior: `.github/workflows/`, `.github/scripts/`, `scripts/install/`, `codex-rs/Cargo.toml`, and `codex-rs/Cargo.lock`
- Project-local auth and config loading: `codex-rs/core/src/config/`, `codex-rs/login/`, `codex-rs/cli/src/login.rs`, and `codex-rs/tui/src/lib.rs`
- Shared upstream changes in app-server v2, core turn state, MCP/rmcp handling, memories, web search, and Windows sandbox code

### Feature Review

| Feature | Result | Evidence | Follow-up |
| --- | --- | --- | --- |
| Kodex CLI rename, telemetry disablement, and update disablement | `needs review` | Preserved `kodex` binary/package paths, disabled update command behavior, fork installer target, and `kodex` user-facing CLI text. `just test -p codex-cli` passed after fixing stale `codex` binary invocations in integration tests and accepting the renamed doctor snapshot. Package/script checks listed below passed. | Run telemetry/updater scoped checks from the feature note: `just test -p codex-analytics`, `just test -p codex-app-server-daemon`, and `just test -p codex-tui`. |
| Project-local auth files | `needs review` | Merge kept project-local auth behavior and `just test -p codex-cli` covered the CLI login/logout precedence tests. | Run the remaining feature verification targets: `just test -p codex-core` and `just test -p codex-login`; consider TUI/exec/cloud requirements auth wiring tests because upstream touched adjacent areas. |
| Fork release pipeline, installer bootstrap, and release versioning | `active` | Release workflow, manifest, version, installer, and fork lockfile checks passed. The installer now accepts fork four-part versions like `x.y.z.build` and still targets `jmilesj/kodex`. | None. |

### Supporting Checks

- `cargo metadata --locked --format-version 1 --no-deps` in `codex-rs`: passed.
- `just bazel-lock-update`: passed.
- `just bazel-lock-check`: passed.
- `bash -n scripts/install/install.sh`: passed.
- `.github/scripts/test_cli_manifest.py`: passed.
- `.github/scripts/test_release_workflow.py`: passed.
- `.github/scripts/test_install_sh.py`: passed.
- `.github/scripts/test_kodex_release_version.py`: passed.
- `python3 -m py_compile ...`: passed for release, install, packaging, and staging scripts.
- `ruby -e 'require "yaml"; YAML.load_file(".github/workflows/release.yml"); puts "ok"'`: passed.
- `node --check codex-cli/bin/kodex.js`: passed.
- `KODEX_CLI_VERSION=0.133.0.1779638524 just test -p codex-cli version_uses_kodex_command_name version::tests`: passed.
- `just test -p codex-cli`: passed all 260 tests with loopback-bind sandbox escalation for doctor probe tests.
- `just fmt`: passed.
- `just fix -p codex-cli -p codex-tui -p codex-utils-cli`: passed.

### Outcome

The upstream merge was resolved with fork release behavior preserved. CLI rename behavior received partial verification, while telemetry/updater and project-local auth follow-up checks remain tracked as `needs review`.
