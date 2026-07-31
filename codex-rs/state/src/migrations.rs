use std::borrow::Cow;

use sqlx::Connection;
use sqlx::Executor;
use sqlx::SqliteConnection;
use sqlx::SqlitePool;
use sqlx::migrate::Migration;
use sqlx::migrate::Migrator;

pub(crate) static STATE_MIGRATOR: Migrator = sqlx::migrate!("./migrations");
pub(crate) static LOGS_MIGRATOR: Migrator = sqlx::migrate!("./logs_migrations");
pub(crate) static GOALS_MIGRATOR: Migrator = sqlx::migrate!("./goals_migrations");
pub(crate) static MEMORIES_MIGRATOR: Migrator = sqlx::migrate!("./memory_migrations");
pub(crate) static THREAD_HISTORY_MIGRATOR: Migrator = sqlx::migrate!("./thread_history_migrations");

// SHA-384 of the goals migration 2 shipped by Kodex before the upstream
// continuation-deferrals migration claimed the same version.
const LEGACY_KODEX_GOALS_MIGRATION_2_CHECKSUM: &[u8] = &[
    0x71, 0x9d, 0x75, 0x5c, 0xdf, 0x4e, 0x2a, 0x65, 0xec, 0x20, 0xff, 0x36, 0x6d, 0xe1, 0xb5, 0x09,
    0x66, 0x97, 0x09, 0xf3, 0x62, 0xcf, 0xb9, 0x19, 0xdd, 0xe7, 0xa4, 0x91, 0x59, 0x11, 0xa4, 0xea,
    0xf3, 0xe8, 0x5e, 0x16, 0x28, 0x61, 0x13, 0x28, 0x5d, 0x75, 0x14, 0x2b, 0x9a, 0x1e, 0x5f, 0x02,
];
const LEGACY_KODEX_DEFERRALS_MIGRATION_VERSION: i64 = 3;
const KODEX_DROP_USAGE_LIMITED_MIGRATION_VERSION: i64 = 20_260_525_033_110;
const THREAD_GOALS_SCHEMA_FINGERPRINT: &str = "CREATETABLEthread_goals(thread_idTEXTPRIMARYKEYNOTNULL,goal_idTEXTNOTNULL,objectiveTEXTNOTNULL,statusTEXTNOTNULLCHECK(statusIN('active','paused','blocked','budget_limited','complete')),token_budgetINTEGER,tokens_usedINTEGERNOTNULLDEFAULT0,time_used_secondsINTEGERNOTNULLDEFAULT0,created_at_msINTEGERNOTNULL,updated_at_msINTEGERNOTNULL)";
const THREAD_GOAL_DEFERRALS_SCHEMA_FINGERPRINT: &str = "CREATETABLEthread_goal_continuation_deferrals(thread_idTEXTPRIMARYKEYNOTNULLREFERENCESthread_goals(thread_id)ONDELETECASCADE)";

#[derive(Clone, Copy)]
enum LegacyGoalsMigrationLayout {
    DropUsageLimitedOnly,
    DropUsageLimitedThenDeferrals,
}

fn legacy_goals_migration_layout(
    applied_migrations: &[(i64, bool, Vec<u8>)],
    initial_migration: &Migration,
    deferrals_migration: &Migration,
) -> Option<LegacyGoalsMigrationLayout> {
    match applied_migrations {
        [
            (initial_version, true, initial_checksum),
            (2, true, legacy_checksum),
        ] if *initial_version == initial_migration.version
            && initial_checksum.as_slice() == initial_migration.checksum.as_ref()
            && legacy_checksum.as_slice() == LEGACY_KODEX_GOALS_MIGRATION_2_CHECKSUM =>
        {
            Some(LegacyGoalsMigrationLayout::DropUsageLimitedOnly)
        }
        [
            (initial_version, true, initial_checksum),
            (2, true, legacy_checksum),
            (LEGACY_KODEX_DEFERRALS_MIGRATION_VERSION, true, deferrals_checksum),
        ] if *initial_version == initial_migration.version
            && initial_checksum.as_slice() == initial_migration.checksum.as_ref()
            && legacy_checksum.as_slice() == LEGACY_KODEX_GOALS_MIGRATION_2_CHECKSUM
            && deferrals_checksum.as_slice() == deferrals_migration.checksum.as_ref() =>
        {
            Some(LegacyGoalsMigrationLayout::DropUsageLimitedThenDeferrals)
        }
        _ => None,
    }
}

async fn legacy_goals_schema_matches(
    connection: &mut SqliteConnection,
    layout: LegacyGoalsMigrationLayout,
) -> anyhow::Result<bool> {
    let thread_goals_schema = sqlx::query_scalar::<_, String>(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'thread_goals'",
    )
    .fetch_optional(&mut *connection)
    .await?;
    if thread_goals_schema
        .as_deref()
        .map(sqlite_schema_fingerprint)
        != Some(THREAD_GOALS_SCHEMA_FINGERPRINT.to_string())
    {
        return Ok(false);
    }
    let deferrals_schema = sqlx::query_scalar::<_, String>(
        "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'thread_goal_continuation_deferrals'",
    )
    .fetch_optional(&mut *connection)
    .await?;
    if matches!(
        layout,
        LegacyGoalsMigrationLayout::DropUsageLimitedThenDeferrals
    ) && deferrals_schema.as_deref().map(sqlite_schema_fingerprint)
        != Some(THREAD_GOAL_DEFERRALS_SCHEMA_FINGERPRINT.to_string())
    {
        return Ok(false);
    }
    if matches!(layout, LegacyGoalsMigrationLayout::DropUsageLimitedOnly)
        && deferrals_schema.is_some()
    {
        return Ok(false);
    }
    let integrity_check = sqlx::query_scalar::<_, String>("PRAGMA integrity_check")
        .fetch_all(&mut *connection)
        .await?;
    if integrity_check.len() != 1 || integrity_check[0] != "ok" {
        return Ok(false);
    }
    let foreign_key_violations =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM pragma_foreign_key_check")
            .fetch_one(&mut *connection)
            .await?;
    if foreign_key_violations != 0 {
        return Ok(false);
    }
    let unexpected_statuses = sqlx::query_scalar::<_, i64>(
        r#"
SELECT COUNT(*)
FROM thread_goals
WHERE status NOT IN ('active', 'paused', 'blocked', 'budget_limited', 'complete')
        "#,
    )
    .fetch_one(&mut *connection)
    .await?;
    if unexpected_statuses != 0 {
        return Ok(false);
    }
    let leftover_tables = sqlx::query_scalar::<_, i64>(
        r#"
SELECT COUNT(*)
FROM sqlite_master
WHERE type = 'table' AND name = 'thread_goals_new'
        "#,
    )
    .fetch_one(&mut *connection)
    .await?;
    if leftover_tables != 0 {
        return Ok(false);
    }
    let deferrals_table_exists = sqlx::query_scalar::<_, i64>(
        r#"
SELECT 1
FROM sqlite_master
WHERE type = 'table' AND name = 'thread_goal_continuation_deferrals'
        "#,
    )
    .fetch_optional(&mut *connection)
    .await?
    .is_some();
    Ok(deferrals_table_exists
        == matches!(
            layout,
            LegacyGoalsMigrationLayout::DropUsageLimitedThenDeferrals
        ))
}

fn sqlite_schema_fingerprint(schema: &str) -> String {
    schema
        .chars()
        .filter(|character| !character.is_ascii_whitespace() && *character != '"')
        .collect()
}

/// Allow an older Codex binary to open a database that has already been
/// migrated by a newer binary running in parallel.
///
/// We intentionally ignore applied migration versions that are newer than the
/// embedded migration set. Known migration versions are still validated by
/// checksum, so this only relaxes the "database is ahead of me" case.
fn runtime_migrator(base: &'static Migrator) -> Migrator {
    Migrator {
        migrations: Cow::Borrowed(base.migrations.as_ref()),
        ignore_missing: true,
        locking: base.locking,
        no_tx: base.no_tx,
        table_name: base.table_name.clone(),
        create_schemas: base.create_schemas.clone(),
    }
}

pub(crate) fn runtime_state_migrator() -> Migrator {
    runtime_migrator(&STATE_MIGRATOR)
}

pub(crate) fn runtime_logs_migrator() -> Migrator {
    runtime_migrator(&LOGS_MIGRATOR)
}

pub(crate) fn runtime_goals_migrator() -> Migrator {
    runtime_migrator(&GOALS_MIGRATOR)
}

pub(crate) fn runtime_memories_migrator() -> Migrator {
    runtime_migrator(&MEMORIES_MIGRATOR)
}

// The paginated history projector will call this when it takes ownership of opening the database.
#[allow(dead_code)]
pub(crate) fn runtime_thread_history_migrator() -> Migrator {
    runtime_migrator(&THREAD_HISTORY_MIGRATOR)
}

async fn migrations_table_exists(pool: &SqlitePool) -> anyhow::Result<bool> {
    Ok(sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_optional(pool)
    .await?
    .is_some())
}

/// Repair migration history written by released Kodex builds before the
/// upstream goals migration was restored to its canonical version 2.
pub(crate) async fn repair_legacy_goals_migration_versions(
    pool: &SqlitePool,
    migrator: &Migrator,
) -> anyhow::Result<()> {
    let Some(deferrals_migration) = migrator
        .migrations
        .iter()
        .find(|migration| migration.version == 2)
    else {
        return Ok(());
    };
    let Some(drop_usage_limited_migration) = migrator
        .migrations
        .iter()
        .find(|migration| migration.version == KODEX_DROP_USAGE_LIMITED_MIGRATION_VERSION)
    else {
        return Ok(());
    };
    if !migrations_table_exists(pool).await? {
        return Ok(());
    }

    let Some(initial_migration) = migrator
        .migrations
        .iter()
        .find(|migration| migration.version == 1)
    else {
        return Ok(());
    };
    let applied_migrations = sqlx::query_as::<_, (i64, bool, Vec<u8>)>(
        "SELECT version, success, checksum FROM _sqlx_migrations ORDER BY version",
    )
    .fetch_all(pool)
    .await?;
    if legacy_goals_migration_layout(
        applied_migrations.as_slice(),
        initial_migration,
        deferrals_migration,
    )
    .is_none()
    {
        return Ok(());
    };

    let mut connection = pool.acquire().await?;
    let mut transaction = connection.begin_with("BEGIN IMMEDIATE").await?;
    let applied_migrations = sqlx::query_as::<_, (i64, bool, Vec<u8>)>(
        "SELECT version, success, checksum FROM _sqlx_migrations ORDER BY version",
    )
    .fetch_all(&mut *transaction)
    .await?;
    let Some(legacy_layout) = legacy_goals_migration_layout(
        applied_migrations.as_slice(),
        initial_migration,
        deferrals_migration,
    ) else {
        transaction.rollback().await?;
        return Ok(());
    };
    if !legacy_goals_schema_matches(&mut transaction, legacy_layout).await? {
        transaction.rollback().await?;
        return Ok(());
    }
    match legacy_layout {
        LegacyGoalsMigrationLayout::DropUsageLimitedOnly => {
            let result = sqlx::query(
                r#"
UPDATE _sqlx_migrations
SET version = ?, description = ?, checksum = ?
WHERE version = 2
  AND success = TRUE
  AND checksum = ?
  AND NOT EXISTS (
      SELECT 1 FROM _sqlx_migrations WHERE version = ?
  )
                "#,
            )
            .bind(drop_usage_limited_migration.version)
            .bind(drop_usage_limited_migration.description.as_ref())
            .bind(drop_usage_limited_migration.checksum.as_ref())
            .bind(LEGACY_KODEX_GOALS_MIGRATION_2_CHECKSUM)
            .bind(drop_usage_limited_migration.version)
            .execute(&mut *transaction)
            .await?;
            if result.rows_affected() == 0 {
                transaction.rollback().await?;
                return Ok(());
            }

            transaction.execute(deferrals_migration.sql.clone()).await?;
            sqlx::query(
                r#"
INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
VALUES (?, ?, TRUE, ?, -1)
                "#,
            )
            .bind(deferrals_migration.version)
            .bind(deferrals_migration.description.as_ref())
            .bind(deferrals_migration.checksum.as_ref())
            .execute(&mut *transaction)
            .await?;
        }
        LegacyGoalsMigrationLayout::DropUsageLimitedThenDeferrals => {
            let version_2_result = sqlx::query(
                r#"
UPDATE _sqlx_migrations
SET version = ?, description = ?, checksum = ?
WHERE version = 2
  AND success = TRUE
  AND checksum = ?
  AND EXISTS (
      SELECT 1
      FROM _sqlx_migrations
      WHERE version = ? AND success = TRUE AND checksum = ?
  )
  AND NOT EXISTS (
      SELECT 1 FROM _sqlx_migrations WHERE version = ?
  )
                "#,
            )
            .bind(drop_usage_limited_migration.version)
            .bind(drop_usage_limited_migration.description.as_ref())
            .bind(drop_usage_limited_migration.checksum.as_ref())
            .bind(LEGACY_KODEX_GOALS_MIGRATION_2_CHECKSUM)
            .bind(LEGACY_KODEX_DEFERRALS_MIGRATION_VERSION)
            .bind(deferrals_migration.checksum.as_ref())
            .bind(drop_usage_limited_migration.version)
            .execute(&mut *transaction)
            .await?;
            if version_2_result.rows_affected() == 0 {
                transaction.rollback().await?;
                return Ok(());
            }

            let version_3_result = sqlx::query(
                r#"
UPDATE _sqlx_migrations
SET version = ?, description = ?, checksum = ?
WHERE version = ?
  AND success = TRUE
  AND checksum = ?
  AND EXISTS (
      SELECT 1
      FROM _sqlx_migrations
      WHERE version = ? AND success = TRUE AND checksum = ?
  )
                "#,
            )
            .bind(deferrals_migration.version)
            .bind(deferrals_migration.description.as_ref())
            .bind(deferrals_migration.checksum.as_ref())
            .bind(LEGACY_KODEX_DEFERRALS_MIGRATION_VERSION)
            .bind(deferrals_migration.checksum.as_ref())
            .bind(drop_usage_limited_migration.version)
            .bind(drop_usage_limited_migration.checksum.as_ref())
            .execute(&mut *transaction)
            .await?;
            if version_3_result.rows_affected() != 1 {
                transaction.rollback().await?;
                anyhow::bail!("failed to repair legacy goals migration metadata");
            }
        }
    }
    transaction.commit().await?;
    Ok(())
}

pub(crate) async fn repair_legacy_recency_migration_version(
    pool: &SqlitePool,
    migrator: &Migrator,
) -> anyhow::Result<()> {
    let Some(recency_migration) = migrator
        .migrations
        .iter()
        .find(|migration| migration.version == 39)
    else {
        return Ok(());
    };
    if !migrations_table_exists(pool).await? {
        return Ok(());
    }

    let legacy_recency_needs_repair = sqlx::query_scalar::<_, i64>(
        r#"
SELECT 1
FROM _sqlx_migrations
WHERE version = ?
  AND checksum = ?
  AND NOT EXISTS (
      SELECT 1 FROM _sqlx_migrations WHERE version = ?
  )
        "#,
    )
    .bind(38_i64)
    .bind(recency_migration.checksum.as_ref())
    .bind(recency_migration.version)
    .fetch_optional(pool)
    .await?
    .is_some();
    if !legacy_recency_needs_repair {
        return Ok(());
    }

    sqlx::query(
        r#"
UPDATE _sqlx_migrations
SET version = ?, description = ?
WHERE version = ?
  AND checksum = ?
  AND NOT EXISTS (
      SELECT 1 FROM _sqlx_migrations WHERE version = ?
  )
        "#,
    )
    .bind(recency_migration.version)
    .bind(recency_migration.description.as_ref())
    .bind(38_i64)
    .bind(recency_migration.checksum.as_ref())
    .bind(recency_migration.version)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
#[path = "migrations_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "goals_migrations_tests.rs"]
mod goals_tests;
