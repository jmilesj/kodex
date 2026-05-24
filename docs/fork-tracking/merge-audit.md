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
| Kodex CLI rename, telemetry disablement, and update disablement | `needs review` | Merge conflicts were resolved and syntax checks were run for shell, Python, and Node packaging paths. The dedicated fork tracking docs did not exist at merge time, so the feature still needs a tracked verification pass. | Run the verification steps in `features/kodex-cli-rename.md`, then update `registry.md` and this audit entry's follow-up status in a later entry. |
| Project-local auth files | `needs review` | Merge conflicts were resolved in config, login, and TUI auth paths. The dedicated fork tracking docs did not exist at merge time, so the feature still needs a tracked verification pass. | Run the verification steps in `features/project-local-auth.md`, then update `registry.md` and this audit entry's follow-up status in a later entry. |

### General Verification Evidence

- `bash -n scripts/install/install.sh`: passed.
- `python3 -m py_compile codex-cli/scripts/build_npm_package.py scripts/stage_npm_packages.py`: passed.
- `node --check codex-cli/bin/kodex.js`: passed.
- `cargo fmt -- --config imports_granularity=Item`: passed after merge cleanup.
- `git diff --cached --check`: reported trailing whitespace in snapshot or patch content that appeared unrelated to the merge resolution.
- TUI crate test run failed in `app::tests::discard_side_thread_removes_agent_navigation_entry` with a stack overflow. This failure was not attributed to a fork-tracked feature during the merge.

### Outcome

The upstream merge completed, but both tracked fork features remain `needs review` until their feature-specific verification steps are run and recorded.
