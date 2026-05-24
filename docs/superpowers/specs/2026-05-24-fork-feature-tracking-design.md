# Fork Feature Tracking and Merge Audit Design

## Objective

Create a lightweight document set that tracks fork-only features against upstream Codex so merge reviews can answer one question quickly: did each fork feature stay relevant, intact, and functional after the upstream changes landed?

## Scope

This design is documents only.

In scope:

- a canonical registry of all fork-only features
- optional per-feature notes for anything that needs its own verification history
- an append-only merge audit log for upstream merge reviews
- manual update rules for keeping the docs current after each upstream merge

Out of scope:

- automation, bots, or generated reports
- code changes
- CI checks
- issue tracker integration
- user-facing product documentation

## Architecture

The document set lives under `docs/fork-tracking/`:

- `README.md` explains the workflow and status vocabulary
- `registry.md` is the canonical index of fork-only features
- `features/<slug>.md` holds per-feature notes when a feature needs dedicated history or verification detail
- `merge-audit.md` records each upstream merge review and the result of rechecking fork features

The registry is the source of truth. Every fork-only feature appears there exactly once. Feature notes are used when a feature is nontrivial enough to need its own checklist, upstream anchor, or history. The merge audit is a chronological record of what was reviewed after each upstream merge.

## Registry Model

Each registry entry should capture:

- feature name
- status
- upstream anchor
- local paths
- verification target
- last verified against
- link to the feature note, if one exists

Recommended statuses:

- `active`: implemented and verified alive
- `needs review`: not yet rechecked after an upstream change
- `degraded`: known to be partially broken or behaviorally suspect
- `retired`: no longer needed in the fork, kept for history

The registry should stay short and scannable. If a feature starts needing more than one sentence to describe its risk or verification path, it gets a dedicated note.

## Feature Note Model

Each feature note should answer six questions:

- What is this feature for?
- Where does the upstream equivalent or conflict live?
- Which local files own it?
- How do we verify it still works?
- What is its current status?
- When was it last verified?

Recommended fields in each note:

- purpose
- upstream anchor
- local implementation paths
- verification steps
- failure modes or merge risks
- status
- last verified commit or merge date
- retirement condition

Feature notes are required for features that are:

- user-visible
- spread across multiple files or subsystems
- historically fragile during upstream merges
- difficult to verify from the registry row alone

## Merge Audit Model

`merge-audit.md` is an append-only log. Each merge entry should record:

- upstream merge commit or range
- merge date
- features rechecked
- outcomes for each feature
- follow-up work, if any

The audit does not replace the registry. It exists so a future merge can answer two questions fast: what changed upstream, and which fork features were revalidated because of it?

## Maintenance Workflow

1. Before an upstream merge, scan the registry for all `active` and `needs review` features.
2. After the merge, identify any features touched directly or indirectly by the upstream diff.
3. Recheck those features against their note or registry verification steps.
4. Update the registry status and `last verified` fields.
5. Append a merge entry to `merge-audit.md` with the review result.
6. If a feature is absorbed upstream or intentionally removed, mark it `retired` instead of deleting history immediately.

The rule is simple: no upstream merge is considered fully reviewed until the registry and audit log reflect the recheck.

## Error Handling

- If a fork-only feature is missing from the registry, add it before the merge review is considered complete.
- If a feature has no clear status, mark it `needs review` rather than guessing.
- If a feature regresses, record it in the feature note and the merge audit before attempting a fix.
- If upstream renames or moves the related code, update the upstream anchor and local paths in the same edit.

## Validation

Validation is human review, not automation. A good review answers:

- Is every fork-only feature represented in the registry?
- Does each nontrivial feature have a note with an upstream anchor and verification steps?
- Did the merge audit record what was checked after the latest upstream merge?
- Are the statuses current and unambiguous?

## Non-Goals

- automatic upstream diffing
- GitHub issue synchronization
- generated dashboards or summaries
- runtime tracking metadata
- replacing the normal merge review process
