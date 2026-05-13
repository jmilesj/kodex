# Kodex CLI Rename Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Install and expose this fork as `kodex`, continue sharing upstream Codex state under `~/.codex`, and disable telemetry plus update checks.

**Architecture:** Keep internal crate and storage names unchanged, and rename only the user-facing binary/package entry points. Disable telemetry and update behavior at initialization and command boundaries so internal types remain compile-compatible while outbound behavior is shut off.

**Tech Stack:** Rust workspace in `codex-rs`, clap CLI parsing, Bazel binary metadata, npm package launcher scripts, Python npm staging helpers.

---

## File Structure

- Modify `codex-rs/cli/Cargo.toml`: rename the top-level Rust binary from `codex` to `kodex`.
- Modify `codex-rs/cli/BUILD.bazel`: rename the Bazel multiplatform binary target from `codex` to `kodex`.
- Modify `codex-rs/cli/src/main.rs`: update clap help usage to `kodex`, force app-server analytics defaults off, and make `update` disabled.
- Modify `codex-rs/cli/tests/*.rs`: update tests that spawn the top-level CLI binary to use `kodex`.
- Modify `codex-cli/package.json`: expose the npm bin as `kodex` and rename the package to `kodex`.
- Rename `codex-cli/bin/codex.js` to `codex-cli/bin/kodex.js`: make the launcher resolve the `kodex` native binary and fork package names.
- Modify `codex-cli/scripts/build_npm_package.py`: stage `kodex` packages and native payload paths.
- Modify `codex-cli/scripts/install_native_deps.py`: install the native top-level CLI as `kodex`.
- Modify `codex-rs/core/src/otel_init.rs`: make OTEL provider initialization a no-op.
- Modify `codex-rs/analytics/src/client.rs` and `codex-rs/analytics/src/client_tests.rs`: make analytics clients disabled by construction and test that behavior.
- Modify `codex-rs/app-server/tests/suite/v2/analytics.rs`: update provider tests for globally disabled telemetry.
- Modify `codex-rs/tui/src/updates.rs` and `codex-rs/tui/src/update_action.rs`: make update checks return no version and update action detection return no action.
- Modify `codex-rs/app-server-daemon/src/update_loop.rs`, `codex-rs/app-server-daemon/src/update_loop_tests.rs`, and `codex-rs/app-server-daemon/src/lib.rs`: make the updater loop and updater re-exec paths inert.

### Task 1: Isolate Workspace

**Files:**
- Inspect: repository git metadata
- Possible modify: `.gitignore` if `.worktrees/` is not ignored

- [ ] **Step 1: Detect current workspace isolation**

Run:

```bash
GIT_DIR=$(cd "$(git rev-parse --git-dir)" 2>/dev/null && pwd -P)
GIT_COMMON=$(cd "$(git rev-parse --git-common-dir)" 2>/dev/null && pwd -P)
BRANCH=$(git branch --show-current)
git rev-parse --show-superproject-working-tree 2>/dev/null
printf 'git_dir=%s\ngit_common=%s\nbranch=%s\n' "$GIT_DIR" "$GIT_COMMON" "$BRANCH"
```

Expected: identify whether this checkout is already a linked worktree. If it is a normal checkout on `main`, do not modify Rust code in place without explicit consent.

- [ ] **Step 2: Prefer an isolated worktree**

If not already isolated, create a linked worktree from the current HEAD on a new branch named `kodex-cli-rename` after verifying the selected worktree directory is ignored:

```bash
git check-ignore -q .worktrees
git add .gitignore
git commit -m "Ignore local worktrees"
git worktree add .worktrees/kodex-cli-rename -b kodex-cli-rename
```

Expected: worktree exists at `.worktrees/kodex-cli-rename`. If `.gitignore` does not ignore `.worktrees`, first add this exact line with `apply_patch`, then run the `git add` and `git commit` commands:

```gitignore
.worktrees/
```

If `.gitignore` already ignores `.worktrees`, skip the `.gitignore` commit. If sandboxing blocks worktree creation, continue in the current checkout only after confirming this with the user.

### Task 2: Rename Rust CLI Entry Point

**Files:**
- Modify: `codex-rs/cli/Cargo.toml`
- Modify: `codex-rs/cli/BUILD.bazel`
- Modify: `codex-rs/cli/src/main.rs`
- Modify: `codex-rs/cli/tests/*.rs`

- [ ] **Step 1: Write failing CLI tests**

Add or update tests in `codex-rs/cli/src/main.rs` so help text and analytics flag behavior are covered:

```rust
#[test]
fn help_uses_kodex_command_name() {
    let help = help_from_args(&["kodex", "--help"]);
    assert!(help.contains("kodex [OPTIONS] [PROMPT]"));
    assert!(help.contains("kodex [OPTIONS] <COMMAND> [ARGS]"));
}

#[test]
fn app_server_analytics_default_enabled_flag_is_ignored() {
    let app_server =
        app_server_from_args(["kodex", "app-server", "--analytics-default-enabled"].as_ref());
    assert!(!app_server.analytics_default_enabled);
}
```

Update `codex-rs/cli/tests/update.rs`:

```rust
fn codex_command(codex_home: &Path) -> Result<assert_cmd::Command> {
    let mut cmd = assert_cmd::Command::new(codex_utils_cargo_bin::cargo_bin("kodex")?);
    cmd.env("CODEX_HOME", codex_home);
    Ok(cmd)
}

#[tokio::test]
async fn update_is_disabled() -> Result<()> {
    let codex_home = TempDir::new()?;

    codex_command(codex_home.path())?
        .arg("update")
        .assert()
        .failure()
        .stderr(contains("`kodex update` is disabled in this fork"));

    Ok(())
}
```

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
cd codex-rs
cargo test -p codex-cli help_uses_kodex_command_name
cargo test -p codex-cli app_server_analytics_default_enabled_flag_is_ignored
cargo test -p codex-cli update_is_disabled
```

Expected before implementation: failures because the binary is still named `codex`, help text still says `codex`, analytics flag is honored, and update command emits the old message.

- [ ] **Step 3: Implement Rust CLI rename and update disablement**

Make these concrete edits:

```toml
# codex-rs/cli/Cargo.toml
[[bin]]
name = "kodex"
path = "src/main.rs"
```

```starlark
# codex-rs/cli/BUILD.bazel
multiplatform_binaries(
    name = "kodex",
)
```

In `codex-rs/cli/src/main.rs`, introduce a single user-facing command constant near the parser:

```rust
const USER_FACING_CLI_NAME: &str = "kodex";
```

Use it in clap metadata:

```rust
bin_name = USER_FACING_CLI_NAME,
override_usage = "kodex [OPTIONS] [PROMPT]\n       kodex [OPTIONS] <COMMAND> [ARGS]"
```

Make the app-server analytics flag parse but never enable analytics:

```rust
let analytics_default_enabled = false;
```

after destructuring `AppServerCommand`, before calling `codex_app_server::run_main_with_transport_options`.

Replace `run_update_command()` with:

```rust
fn run_update_command() -> anyhow::Result<()> {
    anyhow::bail!("`kodex update` is disabled in this fork.");
}
```

Use `rg -n '"codex"' codex-rs/cli/src/main.rs` and change parser/test argv seeds such as `["codex", "exec"]`, `["codex", "app-server"]`, and `std::iter::once("codex")` to `kodex`. Leave non-command data such as `unix://codex.sock`, package names in comments, and internal crate names unchanged.

Update every `codex_utils_cargo_bin::cargo_bin("codex")` in `codex-rs/cli/tests/` to `codex_utils_cargo_bin::cargo_bin("kodex")`.

- [ ] **Step 4: Run focused CLI tests**

Run:

```bash
cd codex-rs
cargo test -p codex-cli
```

Expected: `codex-cli` tests pass.

### Task 3: Rename npm Launcher and Native Package Wiring

**Files:**
- Modify: `codex-cli/package.json`
- Rename: `codex-cli/bin/codex.js` to `codex-cli/bin/kodex.js`
- Modify: `codex-cli/bin/kodex.js`
- Modify: `codex-cli/scripts/build_npm_package.py`
- Modify: `codex-cli/scripts/install_native_deps.py`

- [ ] **Step 1: Write failing packaging checks**

Run these checks before implementation:

```bash
node --check codex-cli/bin/codex.js
python3 codex-cli/scripts/build_npm_package.py --package kodex --version 0.0.0-test --staging-dir /tmp/kodex-npm-stage-check
```

Expected before implementation: `node --check` passes for the old launcher, but staging with `--package kodex` fails because the script only knows `codex`.

- [ ] **Step 2: Implement npm/package rename**

Update `codex-cli/package.json`:

```json
{
  "name": "kodex",
  "version": "0.0.0-dev",
  "license": "Apache-2.0",
  "bin": {
    "kodex": "bin/kodex.js"
  }
}
```

Preserve existing fields such as `type`, `engines`, `files`, `repository`, and `packageManager`.

Rename the launcher:

```bash
git mv codex-cli/bin/codex.js codex-cli/bin/kodex.js
```

In `codex-cli/bin/kodex.js`, update platform packages, binary names, vendor paths, and reinstall hints:

```js
const PLATFORM_PACKAGE_BY_TARGET = {
  "x86_64-unknown-linux-musl": "kodex-linux-x64",
  "aarch64-unknown-linux-musl": "kodex-linux-arm64",
  "x86_64-apple-darwin": "kodex-darwin-x64",
  "aarch64-apple-darwin": "kodex-darwin-arm64",
  "x86_64-pc-windows-msvc": "kodex-win32-x64",
  "aarch64-pc-windows-msvc": "kodex-win32-arm64",
};

const codexBinaryName = process.platform === "win32" ? "kodex.exe" : "kodex";
const updateCommand =
  packageManager === "bun"
    ? "bun install -g kodex@latest"
    : "npm install -g kodex@latest";
```

Update `codex-cli/scripts/build_npm_package.py` constants and package maps so `kodex` is the root package and native component:

```python
CODEX_NPM_NAME = "kodex"

CODEX_PLATFORM_PACKAGES = {
    "kodex-linux-x64": {
        "npm_name": "kodex-linux-x64",
        "npm_tag": "linux-x64",
        "target_triple": "x86_64-unknown-linux-musl",
        "os": "linux",
        "cpu": "x64",
    },
    "kodex-linux-arm64": {
        "npm_name": "kodex-linux-arm64",
        "npm_tag": "linux-arm64",
        "target_triple": "aarch64-unknown-linux-musl",
        "os": "linux",
        "cpu": "arm64",
    },
    "kodex-darwin-x64": {
        "npm_name": "kodex-darwin-x64",
        "npm_tag": "darwin-x64",
        "target_triple": "x86_64-apple-darwin",
        "os": "darwin",
        "cpu": "x64",
    },
    "kodex-darwin-arm64": {
        "npm_name": "kodex-darwin-arm64",
        "npm_tag": "darwin-arm64",
        "target_triple": "aarch64-apple-darwin",
        "os": "darwin",
        "cpu": "arm64",
    },
    "kodex-win32-x64": {
        "npm_name": "kodex-win32-x64",
        "npm_tag": "win32-x64",
        "target_triple": "x86_64-pc-windows-msvc",
        "os": "win32",
        "cpu": "x64",
    },
    "kodex-win32-arm64": {
        "npm_name": "kodex-win32-arm64",
        "npm_tag": "win32-arm64",
        "target_triple": "aarch64-pc-windows-msvc",
        "os": "win32",
        "cpu": "arm64",
    },
}

PACKAGE_EXPANSIONS = {"kodex": ["kodex", *CODEX_PLATFORM_PACKAGES]}
PACKAGE_NATIVE_COMPONENTS = {
    "kodex": [],
    "kodex-linux-x64": ["bwrap", "kodex", "rg"],
    "kodex-linux-arm64": ["bwrap", "kodex", "rg"],
    "kodex-darwin-x64": ["kodex", "rg"],
    "kodex-darwin-arm64": ["kodex", "rg"],
    "kodex-win32-x64": ["kodex", "rg", "codex-windows-sandbox-setup", "codex-command-runner"],
    "kodex-win32-arm64": ["kodex", "rg", "codex-windows-sandbox-setup", "codex-command-runner"],
}
COMPONENT_DEST_DIR["kodex"] = "kodex"
```

Keep helper components `codex-responses-api-proxy`, `codex-windows-sandbox-setup`, and `codex-command-runner` unchanged.

Update `codex-cli/scripts/install_native_deps.py` so the default top-level component is `kodex`:

```python
"kodex": BinaryComponent(
    artifact_prefix="kodex",
    dest_dir="kodex",
    binary_basename="kodex",
),
```

and the default component list uses `"kodex"` instead of `"codex"`.

- [ ] **Step 3: Verify npm staging**

Run:

```bash
node --check codex-cli/bin/kodex.js
python3 codex-cli/scripts/build_npm_package.py --package kodex --version 0.0.0-test --staging-dir /tmp/kodex-npm-stage-check
```

Expected: launcher syntax passes and staging creates `/tmp/kodex-npm-stage-check/package.json` with `"name": "kodex"` and `"bin": {"kodex": "bin/kodex.js"}`.

### Task 4: Disable Telemetry and Analytics

**Files:**
- Modify: `codex-rs/core/src/otel_init.rs`
- Modify: `codex-rs/analytics/src/client.rs`
- Modify: `codex-rs/analytics/src/client_tests.rs`
- Modify: `codex-rs/app-server/tests/suite/v2/analytics.rs`

- [ ] **Step 1: Write failing telemetry tests**

Add this test to `codex-rs/analytics/src/client_tests.rs`:

```rust
use codex_login::CodexAuth;
use codex_login::AuthManager;

#[test]
fn new_client_is_disabled_even_when_analytics_enabled() {
    let client = AnalyticsEventsClient::new(
        AuthManager::from_auth_for_testing(CodexAuth::from_api_key("test")),
        "https://example.test".to_string(),
        Some(true),
    );
    assert!(client.queue.is_none());
}
```

Update `codex-rs/app-server/tests/suite/v2/analytics.rs` so both provider tests assert no metrics are present when `default_analytics_enabled` is `false` or `true`.

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
cd codex-rs
cargo test -p codex-analytics new_client_is_disabled_even_when_analytics_enabled
cargo test -p codex-app-server app_server_default_analytics_enabled_with_flag
```

Expected before implementation: analytics client test fails because `Some(true)` creates a queue, and app-server provider test fails because metrics can be enabled.

- [ ] **Step 3: Implement no-op telemetry boundaries**

In `codex-rs/core/src/otel_init.rs`, make provider construction return no provider:

```rust
pub fn build_provider(
    _config: &Config,
    _service_version: &str,
    _service_name_override: Option<&str>,
    _default_analytics_enabled: bool,
) -> Result<Option<OtelProvider>, Box<dyn Error>> {
    Ok(None)
}
```

Leave `record_process_start` and `install_sqlite_telemetry` as no-ops when no provider exists.

In `codex-rs/analytics/src/client.rs`, make construction disabled:

```rust
pub fn new(
    _auth_manager: Arc<AuthManager>,
    _base_url: String,
    _analytics_enabled: Option<bool>,
) -> Self {
    Self::disabled()
}
```

- [ ] **Step 4: Run telemetry tests**

Run:

```bash
cd codex-rs
cargo test -p codex-analytics
cargo test -p codex-app-server app_server_default_analytics
```

Expected: analytics and focused app-server telemetry tests pass.

### Task 5: Disable Update Checks and Updater Loop

**Files:**
- Modify: `codex-rs/tui/src/updates.rs`
- Modify: `codex-rs/tui/src/update_action.rs`
- Modify: `codex-rs/cli/src/main.rs`
- Modify: `codex-rs/app-server-daemon/src/update_loop.rs`
- Modify: `codex-rs/app-server-daemon/src/update_loop_tests.rs`
- Modify: `codex-rs/app-server-daemon/src/lib.rs`

- [ ] **Step 1: Write failing update-disablement tests**

Update `codex-rs/app-server-daemon/src/update_loop_tests.rs`:

```rust
#[test]
fn changed_updater_does_not_request_refresh_when_updates_are_disabled() {
    assert_eq!(
        update_modes_for_identities(
            &executable_identity_from_bytes(b"old"),
            &executable_identity_from_bytes(b"new"),
        ),
        (RestartMode::IfVersionChanged, UpdaterRefreshMode::None)
    );
}
```

Update the `updater_reexec_waits_for_validated_restart` test in `codex-rs/app-server-daemon/src/lib.rs` to expect all `false` values for `UpdaterRefreshMode::ReexecIfManagedBinaryChanged`.

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
cd codex-rs
cargo test -p codex-app-server-daemon changed_updater_does_not_request_refresh_when_updates_are_disabled
cargo test -p codex-app-server-daemon updater_reexec_waits_for_validated_restart
```

Expected before implementation: tests fail because changed updater identities still request updater refresh and re-exec.

- [ ] **Step 3: Implement update shutdown**

In `codex-rs/tui/src/updates.rs`:

```rust
pub fn get_upgrade_version(_config: &Config) -> Option<String> {
    None
}

pub fn get_upgrade_version_for_popup(_config: &Config) -> Option<String> {
    None
}

pub async fn dismiss_version(_config: &Config, _version: &str) -> anyhow::Result<()> {
    Ok(())
}
```

In `codex-rs/tui/src/update_action.rs`:

```rust
#[cfg(not(debug_assertions))]
pub fn get_update_action() -> Option<UpdateAction> {
    None
}
```

In `codex-rs/app-server-daemon/src/update_loop.rs`, make the updater loop return immediately:

```rust
#[cfg(unix)]
pub(crate) async fn run() -> Result<()> {
    Ok(())
}
```

and make identity comparison never request updater re-exec:

```rust
fn update_modes_for_identities(
    _running_updater_identity: &ExecutableIdentity,
    _managed_identity: &ExecutableIdentity,
) -> (RestartMode, UpdaterRefreshMode) {
    (RestartMode::IfVersionChanged, UpdaterRefreshMode::None)
}
```

In `codex-rs/app-server-daemon/src/lib.rs`, make:

```rust
fn should_reexec_updater(
    _updater_refresh_mode: UpdaterRefreshMode,
    _outcome: RestartIfRunningOutcome,
) -> bool {
    false
}
```

- [ ] **Step 4: Run update tests**

Run:

```bash
cd codex-rs
cargo test -p codex-app-server-daemon
cargo test -p codex-cli update_is_disabled
```

Expected: app-server daemon and CLI update tests pass.

### Task 6: Final Verification and Formatting

**Files:**
- Verify all files changed by previous tasks

- [ ] **Step 1: Run focused test suites**

Run:

```bash
cd codex-rs
cargo test -p codex-cli
cargo test -p codex-analytics
cargo test -p codex-app-server-daemon
cargo test -p codex-app-server app_server_default_analytics
```

Expected: all focused tests pass.

- [ ] **Step 2: Run npm/package checks**

Run:

```bash
node --check codex-cli/bin/kodex.js
python3 codex-cli/scripts/build_npm_package.py --package kodex --version 0.0.0-test --staging-dir /tmp/kodex-npm-stage-final
```

Expected: both commands succeed and staged package exposes `kodex`.

- [ ] **Step 3: Run Rust formatting**

Run:

```bash
cd codex-rs
just fmt
```

Expected: formatting completes successfully.

- [ ] **Step 4: Run scoped lint fix**

Run:

```bash
cd codex-rs
just fix -p codex-cli
just fix -p codex-analytics
just fix -p codex-app-server-daemon
just fix -p codex-app-server
```

Expected: scoped fix commands complete. Per repo instructions, do not rerun tests after `fix` or `fmt`.

- [ ] **Step 5: Audit objective coverage**

Run:

```bash
rg -n 'cargo_bin\("codex"\)|bin_name = "codex"|override_usage = "codex|bin/codex\.js|"codex": "bin/codex\.js"|@openai/codex@latest|run_pid_update_loop|build_provider\(' codex-rs/cli codex-cli codex-rs/core/src/otel_init.rs codex-rs/tui/src codex-rs/app-server-daemon/src codex-rs/analytics/src
```

Expected: no remaining top-level CLI spawn/package references that would install or invoke `codex`; any remaining `codex` strings are internal crate/storage/helper names or intentionally shared `CODEX_HOME` compatibility.

Run:

```bash
git status --short
```

Expected: only files from this plan plus pre-existing unrelated user changes are modified.
