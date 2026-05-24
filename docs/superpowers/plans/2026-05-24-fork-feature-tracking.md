# Fork Feature Tracking Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create the fork tracking document set described in the design spec, seeded with the current fork-only features and latest upstream merge audit.

**Architecture:** Add a docs-only `docs/fork-tracking/` area. `registry.md` is the source of truth, `features/*.md` holds detailed verification notes, and `merge-audit.md` records upstream merge reviews chronologically.

**Tech Stack:** Markdown documents, git history, existing Rust and packaging verification commands referenced as manual checks.

---

## File Structure

- Create `docs/fork-tracking/README.md`: workflow, status vocabulary, and update rules.
- Create `docs/fork-tracking/registry.md`: canonical fork-only feature registry.
- Create `docs/fork-tracking/features/kodex-cli-rename.md`: detailed note for the CLI rename, telemetry disablement, and update disablement feature.
- Create `docs/fork-tracking/features/project-local-auth.md`: detailed note for project-local auth file support.
- Create `docs/fork-tracking/merge-audit.md`: append-only log seeded with the latest upstream merge.

### Task 1: Add Tracking Workflow README

**Files:**
- Create: `docs/fork-tracking/README.md`

- [ ] **Step 1: Create the fork tracking directory and README**

Create `docs/fork-tracking/README.md` with this content:

```markdown
# Fork Tracking

This directory tracks fork-only behavior that must survive upstream merges.

The goal is not to document upstream Codex. The goal is to keep a small, current record of what this fork intentionally changes, where those changes live, and how to verify they are still alive after upstream changes land.

## Files

- `registry.md` is the canonical list of fork-only features.
- `features/` contains detailed notes for features that need their own history or verification checklist.
- `merge-audit.md` records upstream merge reviews and the feature checks performed after each merge.

## Statuses

- `active`: implemented and verified alive against the recorded upstream merge.
- `needs review`: not yet rechecked after an upstream change, or newly added without a full verification pass.
- `degraded`: known to be partially broken or behaviorally suspect.
- `retired`: no longer needed in the fork, kept for history.

## Merge Review Rule

After every upstream merge:

1. Scan `registry.md` for all `active` and `needs review` features.
2. Compare the upstream diff and merge conflicts against each feature's local paths and upstream anchor.
3. Run or manually perform the verification steps listed in the registry or feature note.
4. Update each affected feature's status and last verified field.
5. Append an entry to `merge-audit.md`.

An upstream merge is not fully reviewed until the registry and merge audit reflect the recheck.

## Adding A Feature

Add every fork-only feature to `registry.md` once. Create a file in `features/` when the feature is user-visible, touches multiple subsystems, has merge risk, or needs verification detail that does not fit cleanly in the registry row.

Keep the registry short. Put nuance in the feature note.
```

- [ ] **Step 2: Verify the README renders as a focused workflow doc**

Run:

```bash
sed -n '1,220p' docs/fork-tracking/README.md
```

Expected: the output shows the `Fork Tracking`, `Files`, `Statuses`, `Merge Review Rule`, and `Adding A Feature` sections.

- [ ] **Step 3: Commit the README**

Run:

```bash
git add docs/fork-tracking/README.md
git commit -m "Add fork tracking workflow"
```

Expected: git creates one commit containing only `docs/fork-tracking/README.md`.

### Task 2: Add The Canonical Feature Registry

**Files:**
- Create: `docs/fork-tracking/registry.md`

- [ ] **Step 1: Create the registry with the current fork-only features**

Create `docs/fork-tracking/registry.md` with this content:

```markdown
# Fork Feature Registry

This registry is the source of truth for fork-only behavior. Every fork-only feature should appear here exactly once.

| Feature | Status | Upstream anchor | Local paths | Verification target | Last verified against | Note |
| --- | --- | --- | --- | --- | --- | --- |
| Kodex CLI rename, telemetry disablement, and update disablement | `needs review` | Upstream top-level `codex` CLI, npm packaging, install scripts, telemetry initialization, analytics client, and update checks | `codex-rs/cli/`, `codex-cli/`, `scripts/install/`, `scripts/stage_npm_packages.py`, `codex-rs/analytics/`, `codex-rs/core/src/otel_init.rs`, `codex-rs/tui/src/updates.rs`, `codex-rs/tui/src/update_action.rs`, `codex-rs/app-server-daemon/src/update_loop.rs` | `just test -p codex-cli`; package script syntax checks; telemetry and updater crate tests when touched | Pending review after `d1d1df1dd7` | [`features/kodex-cli-rename.md`](features/kodex-cli-rename.md) |
| Project-local auth files | `needs review` | Upstream global auth/config loading and login/logout storage behavior | `codex-rs/core/src/config/`, `codex-rs/login/src/auth/`, `codex-rs/cli/src/login.rs`, `codex-rs/exec/src/lib.rs`, `codex-rs/tui/src/lib.rs`, `codex-rs/cloud-requirements/src/lib.rs`, `codex-rs/cloud-tasks/src/util.rs` | `just test -p codex-core`; `just test -p codex-login`; `just test -p codex-cli` | Pending review after `d1d1df1dd7` | [`features/project-local-auth.md`](features/project-local-auth.md) |

## Update Rules

- Keep one row per feature.
- Use `needs review` when a feature has not been checked after an upstream merge.
- Use `active` only after the verification target has been run or manually checked and recorded in `merge-audit.md`.
- Use `degraded` when a feature still exists but has a known regression.
- Use `retired` when the fork no longer needs the behavior.
- Update `Last verified against` with the upstream merge commit or date from the audit entry.
```

- [ ] **Step 2: Verify the registry has exactly the seeded feature rows**

Run:

```bash
rg -n "Kodex CLI rename|Project-local auth files" docs/fork-tracking/registry.md
```

Expected: two matches, one for each seeded feature row.

- [ ] **Step 3: Commit the registry**

Run:

```bash
git add docs/fork-tracking/registry.md
git commit -m "Add fork feature registry"
```

Expected: git creates one commit containing only `docs/fork-tracking/registry.md`.

### Task 3: Add Per-Feature Notes

**Files:**
- Create: `docs/fork-tracking/features/kodex-cli-rename.md`
- Create: `docs/fork-tracking/features/project-local-auth.md`

- [ ] **Step 1: Add the Kodex CLI rename feature note**

Create `docs/fork-tracking/features/kodex-cli-rename.md` with this content:

```markdown
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

Pending review after upstream merge `d1d1df1dd7`.

## Retirement Condition

Retire this feature only if the fork intentionally stops using the `kodex` command name and stops disabling telemetry/update behavior, or if upstream provides an accepted mechanism that fully replaces these fork changes.
```

- [ ] **Step 2: Add the project-local auth feature note**

Create `docs/fork-tracking/features/project-local-auth.md` with this content:

```markdown
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
```

- [ ] **Step 3: Verify registry links resolve to the feature note paths**

Run:

```bash
test -f docs/fork-tracking/features/kodex-cli-rename.md
test -f docs/fork-tracking/features/project-local-auth.md
rg -n "features/kodex-cli-rename.md|features/project-local-auth.md" docs/fork-tracking/registry.md
```

Expected: both `test` commands succeed, and `rg` prints both feature note links from the registry.

- [ ] **Step 4: Commit the feature notes**

Run:

```bash
git add docs/fork-tracking/features/kodex-cli-rename.md docs/fork-tracking/features/project-local-auth.md
git commit -m "Add fork feature notes"
```

Expected: git creates one commit containing both feature note files.

### Task 4: Add The Initial Merge Audit

**Files:**
- Create: `docs/fork-tracking/merge-audit.md`

- [ ] **Step 1: Create the merge audit with the latest upstream merge**

Create `docs/fork-tracking/merge-audit.md` with this content:

```markdown
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
```

- [ ] **Step 2: Verify the audit names the merge and both feature rows**

Run:

```bash
rg -n "d1d1df1dd7|Kodex CLI rename|Project-local auth files" docs/fork-tracking/merge-audit.md
```

Expected: matches for the merge commit and both feature review rows.

- [ ] **Step 3: Commit the merge audit**

Run:

```bash
git add docs/fork-tracking/merge-audit.md
git commit -m "Add initial fork merge audit"
```

Expected: git creates one commit containing only `docs/fork-tracking/merge-audit.md`.

### Task 5: Validate The Document Set

**Files:**
- Inspect: `docs/fork-tracking/README.md`
- Inspect: `docs/fork-tracking/registry.md`
- Inspect: `docs/fork-tracking/features/kodex-cli-rename.md`
- Inspect: `docs/fork-tracking/features/project-local-auth.md`
- Inspect: `docs/fork-tracking/merge-audit.md`

- [ ] **Step 1: Verify all expected tracking files exist**

Run:

```bash
find docs/fork-tracking -type f | sort
```

Expected output:

```text
docs/fork-tracking/README.md
docs/fork-tracking/features/kodex-cli-rename.md
docs/fork-tracking/features/project-local-auth.md
docs/fork-tracking/merge-audit.md
docs/fork-tracking/registry.md
```

- [ ] **Step 2: Check for unresolved placeholder language in the tracking docs**

Run:

```bash
rg -n "fill me|later placeholder|unresolved marker" docs/fork-tracking
```

Expected: no matches.

- [ ] **Step 3: Check markdown whitespace**

Run:

```bash
git diff --check -- docs/fork-tracking
```

Expected: no whitespace errors for the tracking docs.

- [ ] **Step 4: Confirm git status contains only intentional work**

Run:

```bash
git status --short
```

Expected: the tracking docs are committed. Any unrelated untracked local files, such as `.codex/auth.json`, remain untouched.
