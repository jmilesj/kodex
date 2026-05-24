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
