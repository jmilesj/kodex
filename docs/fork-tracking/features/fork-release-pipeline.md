# Fork Release Pipeline, Installer Bootstrap, And Release Versioning

## Purpose

This fork publishes its own binary release artifacts, installs them through a stable bootstrap script, and displays a fork-specific release version without rewriting the workspace manifest during CI.

The release pipeline also keeps Linux and macOS release targets aligned with the fork's supported binaries and uses cache reuse to keep repeated release builds from recompiling the whole workspace. Linux release binaries use musl targets built through `cargo-zigbuild` so the published artifacts do not inherit the GitHub runner's glibc floor. The shipped CLI intentionally avoids the native realtime WebRTC dependency path (`libwebrtc` / `webrtc-sys`) so release builds do not spend time compiling native WebRTC glue.

## Upstream Anchor

Upstream owns the release workflow, GitHub Release publishing, installer bootstrap path, and CLI version stamping behavior that this fork diverges from.

Primary upstream areas to inspect after merges:

- `.github/workflows/release.yml`
- `scripts/install/`
- `README.md`
- `codex-rs/cli/src/main.rs`
- `codex-rs/cli/src/version.rs`

## Local Implementation Paths

- `.github/workflows/release.yml`
- `.github/actions/setup-rusty-v8/action.yml`
- `.github/scripts/kodex_release_version.py`
- `.github/scripts/test_cli_manifest.py`
- `.github/scripts/test_release_workflow.py`
- `.github/scripts/test_install_sh.py`
- `.github/scripts/test_kodex_release_version.py`
- `MODULE.bazel`
- `defs.bzl`
- `codex-rs/Cargo.toml`
- `codex-rs/Cargo.lock`
- `scripts/install/install.sh`
- `README.md`
- `codex-rs/cli/src/main.rs`
- `codex-rs/cli/src/version.rs`
- `codex-rs/tui/`

## Verification Steps

Run these to check the fork release pipeline and installer behavior:

```bash
python3 .github/scripts/test_cli_manifest.py
python3 .github/scripts/test_release_workflow.py
python3 .github/scripts/test_install_sh.py
python3 .github/scripts/test_kodex_release_version.py
python3 -m py_compile .github/scripts/kodex_release_version.py .github/scripts/test_cli_manifest.py .github/scripts/test_release_workflow.py .github/scripts/test_install_sh.py .github/scripts/test_kodex_release_version.py
bash -n scripts/install/install.sh
ruby -e 'require "yaml"; YAML.load_file(".github/workflows/release.yml"); puts "ok"'
cd codex-rs
KODEX_CLI_VERSION=0.133.0.1779638524 just test -p codex-cli version_uses_kodex_command_name version::tests
```

Manual checks:

- The release workflow publishes `kodex` binaries for `aarch64-apple-darwin`, `x86_64-unknown-linux-musl`, and `aarch64-unknown-linux-musl`.
- Linux release workflow builds musl targets with `cargo-zigbuild`, preloaded rusty_v8 artifacts, release `opt-level=2`, and cargo timing artifact upload.
- The workflow passes `KODEX_CLI_VERSION` instead of rewriting `Cargo.toml`.
- `kodex --version` reports the fork release version, not the workspace package version.
- The release `codex-cli` build does not depend on `codex-app-server-test-client` or native realtime WebRTC crates.
- The TUI starts realtime voice without supplying a WebRTC SDP offer, so core uses the websocket/audio-chunk transport.
- `scripts/install/install.sh` resolves the latest fork release from GitHub, selects musl Linux release assets, skips reinstall when the local version is current, and configures `PATH`.
- The README points users at the stable installer URL and the fork release binaries.

## Recent Fork Commits

- `ed08976002` `Use fork-only binary releases`
- `6ee88de836` `Fix release build lockfile mismatch`
- `6e8b9d232b` `Reduce fork release workflow scope`
- `6c31a98937` `Fix release targets and cache`
- `29c441edff` `Restore release workflow push trigger`
- `a18f8b2d2c` `Fix fork release versioning and installer`
- `95edff9679` `Improve release build cache reuse`
- `477b094d6c` `Document fork tracking in README and agents`

## Merge Risks

- Upstream can change release publishing or installer assumptions and reintroduce a need to stamp `Cargo.toml`.
- Upstream can change CLI version handling and break the fork-specific release version display.
- Upstream can change binary target support or release asset naming.
- Upstream can change release workflow caching behavior and make repeated release builds slow again.
- Upstream can reintroduce native realtime WebRTC into the shipped CLI dependency graph through the TUI.

## Status

`active`

## Last Verified

2026-05-25

## Retirement Condition

Retire this feature only if the fork stops shipping its own binary releases, stops using the stable installer bootstrap, or no longer needs fork-specific release versioning and cache reuse.
