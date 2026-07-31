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

## 2026-07-27 - Upstream Sync `rust-v0.145.0`

- Merge commit: this merge resolution commit.
- Local pre-merge parent: `553b1e4eeba818c546fdc2df54915513e9e9eca4`
- Upstream tag: `rust-v0.145.0` (`1635de866c61d1b76e50b31928ee6d61482435a8`)
- Peeled upstream commit: `25af12f7e61572b0bc18ddb1008be543b91519b0`
- Merge target: upstream release version `0.145.0`

### Conflict Areas

The merge recorded conflicts or fork-review decisions in these areas that overlap fork-only behavior:

- CLI rename, package wiring, and user-facing command text: `codex-rs/cli/`, `codex-rs/utils/cargo-bin/`, `codex-cli/`, package scripts, installer scripts, and TUI snapshots.
- Telemetry, analytics, and update disablement: `codex-rs/analytics/`, analytics-dependent app-server/core tests, `codex-rs/tui/src/update_action.rs`, and `codex-rs/app-server-daemon/`.
- Project-local auth and config loading: `codex-rs/core/src/config/`, `codex-rs/login/src/auth/`, `codex-rs/tui/src/status/tests.rs`, and app-server websocket startup fixtures.
- Release and dependency wiring: `codex-rs/Cargo.toml`, `codex-rs/Cargo.lock`, `MODULE.bazel.lock`, and vendored `rama-*` crates.
- Upstream behavior changes adjacent to fork code: thread goal state migration ordering, `UsageLimited` to `BudgetLimited` goal status naming, unified exec lifecycle handling, network proxy local binding policy, app-server v2 analytics expectations, and protocol retry classification.

### Feature Review

| Feature | Result | Evidence | Follow-up |
| --- | --- | --- | --- |
| Kodex CLI rename, telemetry disablement, and update disablement | `active` | Preserved `kodex` CLI/package/install behavior, disabled updater output, disabled analytics delivery, and `kodex` user-facing text in CLI/TUI snapshots. `just test -p codex-cli`, `just test -p codex-analytics`, `just test -p codex-app-server-daemon`, `just test -p codex-tui`, package syntax checks, release script tests, and affected app-server/core tests passed. | None. |
| Project-local auth files | `active` | Preserved project-local auth precedence and isolated app-server/TUI fixtures from repository `.codex/auth.json` pollution. `just test -p codex-core`, `just test -p codex-login`, `just test -p codex-cli`, `just test -p codex-cloud-config`, `just test -p codex-tui`, and affected app-server tests passed. | None. |
| Fork release pipeline, installer bootstrap, and release versioning | `active` | Preserved fork release script behavior, installer bootstrap, lockfile state, vendored dependency overrides, `kodex` package manifest, and version/update snapshots for `0.145.0`. Release script tests, installer syntax checks, `just bazel-lock-update`, and `just bazel-lock-check` passed. | None. |

### Supporting Checks

- `python3 -m py_compile`: passed for release, install, package, and staging scripts.
- `bash -n scripts/install/install.sh`: passed.
- `node --check codex-cli/bin/kodex.js`: passed.
- `.github/scripts/test_release_workflow.py`: passed.
- `.github/scripts/test_install_sh.py`: passed.
- `.github/scripts/test_kodex_release_version.py`: passed.
- `.github/scripts/test_cli_manifest.py`: passed.
- `just bazel-lock-update`: passed.
- `just bazel-lock-check`: passed.
- `just test -p codex-login`: passed.
- `just test -p codex-cloud-config`: passed.
- `just test -p codex-goal-extension`: passed.
- `just test -p codex-app-server-daemon`: passed.
- `just test -p codex-network-proxy`: passed.
- `just test -p codex-tui`: passed, with intentional snapshot updates accepted.
- `just test -p codex-analytics`: passed.
- Affected core/CLI/support crates: `just test -p codex-core -p codex-cli -p codex-rmcp-client -p codex-exec -p codex-linux-sandbox -p codex-code-mode-host -p codex-shell-escalation -p codex-apply-patch`: passed.
- Affected app-server/protocol/state/network/api crates: `just test -p codex-analytics -p codex-app-server -p codex-app-server-protocol -p codex-state -p codex-network-proxy -p codex-protocol -p codex-api`: passed with one retried flaky `login_account_chatgpt_redirects_to_hosted_success_page`.

### Outcome

The upstream `rust-v0.145.0` sync was resolved with tracked fork behavior preserved and reverified. The full workspace `just test` suite was not run during this audit; scoped tests covered the crates touched by the merge resolution.

## 2026-07-31 - Upstream Sync `rust-v0.146.0`

- Merge commit: `84794a8b2512ec3d9c63ac15a06052a02eb577af`.
- Local pre-merge parent: `84f9334bb790fddc34b250efeb9e67e878c6545c`
- Upstream tag: `rust-v0.146.0` (`be449751a978f02e5bbba886999662956c7f38f5`)
- Peeled upstream commit: `e363b08c9175ac1cbe5893615dd2cb9ddf95043b`
- Merge target: upstream release version `0.146.0`

### Conflict Areas

The merge recorded conflicts or fork-review decisions in these areas:

- CLI rename, installer bootstrap, release workflow, and version/dependency wiring: `README.md`, `.github/workflows/`, `scripts/install/`, `codex-rs/Cargo.toml`, `codex-rs/Cargo.lock`, and `MODULE.bazel.lock`.
- Telemetry, analytics, announcements, and update disablement: `codex-rs/analytics/`, `codex-rs/otel/`, `codex-rs/http-client/`, `codex-rs/tui/`, `codex-rs/cli/src/doctor/updates.rs`, and `codex-rs/app-server-daemon/`.
- Project-local auth and route configuration: `codex-rs/core/src/config/`, `codex-rs/login/src/auth/`, `codex-rs/cli/src/login.rs`, `codex-rs/cloud-config/`, and `codex-rs/tui/src/onboarding/auth.rs`.
- Shared upstream API and runtime changes: app-server config/MCP tests, protocol error handling, unified exec watchers, exec-server transports, and Windows sandbox setup.

### Feature Review

| Feature | Result | Evidence | Follow-up |
| --- | --- | --- | --- |
| Kodex CLI rename, telemetry disablement, and update disablement | `active` | Preserved `kodex` packaging and installer behavior, kept analytics/OTEL exporters inert, removed the upstream announcement fetch, disabled the doctor’s remote version probe, and corrected newly exposed `kodex` help/examples. Full CLI and TUI suites, analytics, and release-script checks passed. | None. |
| Project-local auth files | `active` | Preserved nearest-first project auth loading and project-aware login/logout across core, login, CLI, cloud-config, exec, and TUI route changes. Full CLI/login and app-server auth coverage passed. | None. |
| Fork release pipeline, installer bootstrap, and release versioning | `active` | Preserved the fork release workflow, `jmilesj/kodex` installer/release URLs, version stamping, musl target graph, and removal of upstream cloud-task/WebRTC release dependencies. Cargo and Bazel lockfiles were regenerated from the merged manifests. | None. |
| Goals database migration compatibility | `active` | Restored upstream's canonical goals migration 2, moved the fork-only status migration to a timestamped version, and added exact-history repair for released Kodex databases. Upgrade coverage preserves goals and continuation deferrals. | None. |

### Supporting Checks

- `cargo metadata --locked --format-version 1 --no-deps`: passed.
- `cargo generate-lockfile`: passed with network access; cloud-task packages and WebRTC crates remain absent from the shipped graph.
- `just bazel-lock-update`: passed with Bazel cache access.
- `just bazel-lock-check`: passed with Bazel cache access.
- `bash -n scripts/install/install.sh`: passed.
- `.github/scripts/test_cli_manifest.py`, `.github/scripts/test_release_workflow.py`, `.github/scripts/test_install_sh.py`, and `.github/scripts/test_kodex_release_version.py`: passed.
- Python compilation, Node syntax, and release workflow YAML checks: passed.
- `just test -p codex-cli`: 327 passed, 1 skipped.
- `just test -p codex-tui --retries 3`: 3,251 passed, 4 skipped (one retry-marked flaky test).
- `just test -p codex-app-server --retries 3`: 1,003 passed, 17 skipped (one retry-marked flaky test).
- `just test -p codex-core --retries 2`: 3,074 passed, 22 skipped; four remote/sandbox-dependent tests could not run under this host's enforced sandbox environment.
- `just test -p codex-protocol`: 265 passed; `codex-app-server-protocol`: 274 passed; `codex-mcp`: 122 passed; `codex-analytics`: 3 passed.
- Grouped login/config/state/network/identity suites: 914 passed; `codex-http-client`: 67 passed; `codex-app-server-transport --retries 3`: 142 passed (six retry-marked flaky tests); serial `codex-exec-server --test-threads 1`: 359 passed, 3 skipped.
- `just test -p codex-state`: 165 passed after correcting the goals migration checksum collision; focused coverage includes upstream and both released Kodex layouts that contain the colliding migration histories.
- `codex-windows-sandbox`, installer scripts, and release scripts passed on the host; Windows-only behavior still requires Windows CI.
- The shell-network environment test and three remote-environment core tests are documented as sandbox-limited; no `CODEX_SANDBOX_*` code or test behavior was changed.
- Upstream app-server exporter tests were removed because this fork’s `codex-otel` implementation is intentionally exporter-free; production app-list behavior remains in place.
- The complete workspace `just test` suite was not run; scoped tests cover the changed crates and a full run requires explicit approval under the repository instructions.

### Outcome

The `rust-v0.146.0` upstream sync is resolved with the tracked fork features preserved and reverified. The post-sync goals database checksum collision is repaired without discarding goal state. At audit time, `origin/main` points to the merge commit.
