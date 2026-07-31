# Goals Database Migrations

Kodex and upstream Codex share `goals_1.sqlite`, but Kodex normalizes and
rejects the legacy `usage_limited` goal status. The first implementation used
goals migration 2
for that fork-only schema change. Upstream later assigned its own migration 2
to continuation deferrals, so a database written by either binary failed the
other binary's checksum validation.

## Invariants

- Keep upstream sequential goals migrations at their canonical versions and
  byte-for-byte checksums.
- Use timestamped versions for fork-only goals migrations so future upstream
  sequential versions do not collide.
- Repair only exact migration histories shipped by known Kodex releases.
  Unknown checksums, dirty migrations, unexpected histories, integrity errors,
  and foreign-key violations must not be rewritten by the compatibility path;
  normal SQLx migration policy still applies.
- Preserve goal rows and continuation-deferral rows while normalizing
  `thread_goals` in place. The timestamped migration does not rebuild the
  table, so future upstream columns remain intact.

The compatibility repair recognizes the two legacy Kodex histories:

- versions 1 and 2, where legacy version 2 removed `usage_limited`;
- versions 1, 2, and 3, where legacy version 3 added continuation deferrals.

It moves the fork migration to its timestamped version and restores upstream's
continuation-deferrals migration to version 2. Older upstream binaries can
then validate version 2 and ignore the newer fork migration, although binaries
that still write `usage_limited` are not fully schema-compatible. The current
fork migration also installs insert/update guards so a future upstream
migration that adds columns cannot reintroduce that status or discard those
columns. A future upstream table-rebuild migration would need to recreate the
guards. Older Kodex
binaries that expect the legacy version 2 checksum cannot open a repaired
database; one SQLx migration row cannot satisfy both checksums.

## Verification

Run `just test -p codex-state`. The migration tests cover upstream databases
with nonempty deferrals, both released Kodex layouts, retained goal data,
foreign-key integrity, final checksums, and rejection of unknown checksums.
