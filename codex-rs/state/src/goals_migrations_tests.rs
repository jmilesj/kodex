use std::borrow::Cow;
use std::path::PathBuf;

use codex_utils_absolute_path::test_support::PathExt;
use pretty_assertions::assert_eq;
use sqlx::Row;
use sqlx::SqlitePool;
use sqlx::migrate::MigrateError;
use sqlx::migrate::Migrator;

use super::GOALS_MIGRATOR;
use super::LEGACY_KODEX_GOALS_MIGRATION_2_CHECKSUM;
use super::repair_legacy_goals_migration_versions;
use super::runtime_goals_migrator;

type GoalRow = (
    String,
    String,
    String,
    String,
    Option<i64>,
    i64,
    i64,
    i64,
    i64,
);

struct GoalsDbFixture {
    sqlite_home: PathBuf,
    sqlite: crate::SqliteConfig,
}

impl GoalsDbFixture {
    async fn new() -> Self {
        let sqlite_home = crate::runtime::test_support::unique_temp_dir();
        tokio::fs::create_dir_all(&sqlite_home)
            .await
            .expect("sqlite home should be created");
        let sqlite = crate::SqliteConfig::new_for_testing(sqlite_home.as_path().abs());
        Self {
            sqlite_home,
            sqlite,
        }
    }

    async fn open(&self) -> SqlitePool {
        self.sqlite
            .open_read_write_pool(&self.sqlite.goals_db_path())
            .await
            .expect("goals database should open")
    }

    async fn open_current(&self) -> SqlitePool {
        let migrator = super::runtime_goals_migrator();
        self.sqlite
            .open_goals_db(&migrator, /*telemetry_override*/ None)
            .await
            .expect("goals database should upgrade")
    }
}

impl Drop for GoalsDbFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.sqlite_home);
    }
}

fn goals_migrator_through(version: i64) -> Migrator {
    Migrator {
        migrations: Cow::Owned(
            GOALS_MIGRATOR
                .migrations
                .iter()
                .filter(|migration| migration.version <= version)
                .cloned()
                .collect(),
        ),
        ignore_missing: GOALS_MIGRATOR.ignore_missing,
        locking: GOALS_MIGRATOR.locking,
        table_name: GOALS_MIGRATOR.table_name.clone(),
        create_schemas: GOALS_MIGRATOR.create_schemas.clone(),
        no_tx: GOALS_MIGRATOR.no_tx,
    }
}

async fn assert_current_goals_migration_history(pool: &SqlitePool) {
    let applied = sqlx::query("SELECT version, checksum FROM _sqlx_migrations ORDER BY version")
        .fetch_all(pool)
        .await
        .expect("applied goals migrations should load")
        .into_iter()
        .map(|row| {
            (
                row.get::<i64, _>("version"),
                row.get::<Vec<u8>, _>("checksum"),
            )
        })
        .collect::<Vec<_>>();
    let expected = GOALS_MIGRATOR
        .migrations
        .iter()
        .map(|migration| (migration.version, migration.checksum.to_vec()))
        .collect::<Vec<_>>();
    assert_eq!(applied, expected);
}

async fn insert_preserved_goal(pool: &SqlitePool) {
    sqlx::query(
        r#"
INSERT INTO thread_goals (
    thread_id,
    goal_id,
    objective,
    status,
    token_budget,
    tokens_used,
    time_used_seconds,
    created_at_ms,
    updated_at_ms
) VALUES ('thread-1', 'goal-1', 'objective', 'active', 100, 10, 20, 1000, 2000)
        "#,
    )
    .execute(pool)
    .await
    .expect("goal fixture should insert");
}

async fn record_migration(pool: &SqlitePool, version: i64, description: &str, checksum: &[u8]) {
    sqlx::query(
        r#"
INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
VALUES (?, ?, TRUE, ?, -1)
        "#,
    )
    .bind(version)
    .bind(description)
    .bind(checksum)
    .execute(pool)
    .await
    .expect("migration fixture row should be recorded");
}

async fn apply_legacy_status_schema(pool: &SqlitePool) {
    let mut transaction = pool
        .begin()
        .await
        .expect("fixture transaction should begin");
    sqlx::query("DROP TABLE IF EXISTS thread_goal_continuation_deferrals")
        .execute(&mut *transaction)
        .await
        .expect("deferrals table should be dropped before rebuilding goals");
    sqlx::query(
        r#"
CREATE TABLE thread_goals_new (
    thread_id TEXT PRIMARY KEY NOT NULL,
    goal_id TEXT NOT NULL,
    objective TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN (
        'active',
        'paused',
        'blocked',
        'budget_limited',
        'complete'
    )),
    token_budget INTEGER,
    tokens_used INTEGER NOT NULL DEFAULT 0,
    time_used_seconds INTEGER NOT NULL DEFAULT 0,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
)
        "#,
    )
    .execute(&mut *transaction)
    .await
    .expect("legacy goals table should be created");
    sqlx::query(
        r#"
INSERT INTO thread_goals_new (
    thread_id,
    goal_id,
    objective,
    status,
    token_budget,
    tokens_used,
    time_used_seconds,
    created_at_ms,
    updated_at_ms
)
SELECT
    thread_id,
    goal_id,
    objective,
    status,
    token_budget,
    tokens_used,
    time_used_seconds,
    created_at_ms,
    updated_at_ms
FROM thread_goals
        "#,
    )
    .execute(&mut *transaction)
    .await
    .expect("legacy goals should be copied");
    sqlx::query("DROP TABLE thread_goals")
        .execute(&mut *transaction)
        .await
        .expect("current goals table should be dropped");
    let old_goals_table = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'thread_goals'",
    )
    .fetch_optional(&mut *transaction)
    .await
    .expect("fixture schema should load");
    assert_eq!(old_goals_table, None);
    sqlx::query("ALTER TABLE thread_goals_new RENAME TO thread_goals")
        .execute(&mut *transaction)
        .await
        .expect("legacy goals table should be renamed");
    transaction
        .commit()
        .await
        .expect("fixture transaction should commit");
}

async fn create_deferrals_table(pool: &SqlitePool) {
    sqlx::query(
        r#"
CREATE TABLE thread_goal_continuation_deferrals (
    thread_id TEXT PRIMARY KEY NOT NULL REFERENCES thread_goals(thread_id) ON DELETE CASCADE
)
        "#,
    )
    .execute(pool)
    .await
    .expect("deferrals table should be created");
}

fn preserved_goal() -> GoalRow {
    (
        "thread-1".to_string(),
        "goal-1".to_string(),
        "objective".to_string(),
        "active".to_string(),
        Some(100),
        10,
        20,
        1000,
        2000,
    )
}

async fn load_goals(pool: &SqlitePool) -> Vec<GoalRow> {
    sqlx::query_as(
        r#"
SELECT
    thread_id,
    goal_id,
    objective,
    status,
    token_budget,
    tokens_used,
    time_used_seconds,
    created_at_ms,
    updated_at_ms
FROM thread_goals
ORDER BY thread_id
        "#,
    )
    .fetch_all(pool)
    .await
    .expect("goals should load")
}

async fn load_deferrals(pool: &SqlitePool) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT thread_id FROM thread_goal_continuation_deferrals ORDER BY thread_id",
    )
    .fetch_all(pool)
    .await
    .expect("deferrals should load")
}

#[tokio::test]
async fn upgrades_upstream_goals_database_without_losing_deferrals() {
    let fixture = GoalsDbFixture::new().await;
    let pool = fixture.open().await;
    goals_migrator_through(/*version*/ 2)
        .run(&pool)
        .await
        .expect("upstream goals migrations should apply");
    sqlx::query(
        r#"
INSERT INTO thread_goals (
    thread_id,
    goal_id,
    objective,
    status,
    token_budget,
    tokens_used,
    time_used_seconds,
    created_at_ms,
    updated_at_ms
) VALUES
    ('thread-active', 'goal-active', 'active objective', 'active', 100, 10, 20, 1000, 2000),
    ('thread-limited', 'goal-limited', 'limited objective', 'usage_limited', NULL, 30, 40, 3000, 4000)
        "#,
    )
    .execute(&pool)
    .await
    .expect("upstream goals should insert");
    sqlx::query(
        r#"
INSERT INTO thread_goal_continuation_deferrals (thread_id)
VALUES ('thread-active'), ('thread-limited')
        "#,
    )
    .execute(&pool)
    .await
    .expect("upstream deferrals should insert");
    pool.close().await;

    let pool = fixture.open_current().await;
    assert_eq!(
        load_goals(&pool).await,
        vec![
            (
                "thread-active".to_string(),
                "goal-active".to_string(),
                "active objective".to_string(),
                "active".to_string(),
                Some(100),
                10,
                20,
                1000,
                2000,
            ),
            (
                "thread-limited".to_string(),
                "goal-limited".to_string(),
                "limited objective".to_string(),
                "active".to_string(),
                None,
                30,
                40,
                3000,
                4000,
            ),
        ]
    );
    assert_eq!(
        load_deferrals(&pool).await,
        vec!["thread-active".to_string(), "thread-limited".to_string()]
    );
    let foreign_key_violations =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM pragma_foreign_key_check")
            .fetch_one(&pool)
            .await
            .expect("foreign key violations should be counted");
    assert_eq!(foreign_key_violations, 0);
    assert_current_goals_migration_history(&pool).await;

    pool.close().await;
}

#[tokio::test]
async fn repairs_goals_migrations_from_kodex_v0_133_layout() {
    let fixture = GoalsDbFixture::new().await;
    let pool = fixture.open().await;
    goals_migrator_through(/*version*/ 1)
        .run(&pool)
        .await
        .expect("legacy goals migration 1 should apply");
    insert_preserved_goal(&pool).await;
    apply_legacy_status_schema(&pool).await;
    record_migration(
        &pool,
        /*version*/ 2,
        "drop usage limited thread goal status",
        LEGACY_KODEX_GOALS_MIGRATION_2_CHECKSUM,
    )
    .await;
    pool.close().await;

    let pool = fixture.open().await;
    repair_legacy_goals_migration_versions(&pool, &GOALS_MIGRATOR)
        .await
        .expect("legacy goals migration history should be repaired atomically");
    assert_eq!(load_goals(&pool).await, vec![preserved_goal()]);
    assert_eq!(load_deferrals(&pool).await, Vec::<String>::new());
    assert_current_goals_migration_history(&pool).await;

    pool.close().await;
}

#[tokio::test]
async fn repairs_goals_migrations_from_kodex_v0_145_layout() {
    let fixture = GoalsDbFixture::new().await;
    let pool = fixture.open().await;
    goals_migrator_through(/*version*/ 1)
        .run(&pool)
        .await
        .expect("legacy goals migration 1 should apply");
    insert_preserved_goal(&pool).await;
    apply_legacy_status_schema(&pool).await;
    create_deferrals_table(&pool).await;
    sqlx::query("INSERT INTO thread_goal_continuation_deferrals (thread_id) VALUES ('thread-1')")
        .execute(&pool)
        .await
        .expect("legacy deferral should insert");
    record_migration(
        &pool,
        /*version*/ 2,
        "drop usage limited thread goal status",
        LEGACY_KODEX_GOALS_MIGRATION_2_CHECKSUM,
    )
    .await;
    let deferrals_migration = GOALS_MIGRATOR
        .migrations
        .iter()
        .find(|migration| migration.version == 2)
        .expect("deferrals migration should exist");
    record_migration(
        &pool,
        /*version*/ 3,
        deferrals_migration.description.as_ref(),
        deferrals_migration.checksum.as_ref(),
    )
    .await;
    pool.close().await;

    let pool = fixture.open_current().await;
    assert_eq!(load_goals(&pool).await, vec![preserved_goal()]);
    assert_eq!(load_deferrals(&pool).await, vec!["thread-1".to_string()]);
    assert_current_goals_migration_history(&pool).await;

    pool.close().await;
}

#[tokio::test]
async fn refuses_legacy_repair_for_malformed_deferrals_schema() {
    let fixture = GoalsDbFixture::new().await;
    let pool = fixture.open().await;
    goals_migrator_through(/*version*/ 1)
        .run(&pool)
        .await
        .expect("legacy goals migration 1 should apply");
    apply_legacy_status_schema(&pool).await;
    create_deferrals_table(&pool).await;
    record_migration(
        &pool,
        /*version*/ 2,
        "drop usage limited thread goal status",
        LEGACY_KODEX_GOALS_MIGRATION_2_CHECKSUM,
    )
    .await;
    let deferrals_migration = GOALS_MIGRATOR
        .migrations
        .iter()
        .find(|migration| migration.version == 2)
        .expect("deferrals migration should exist");
    record_migration(
        &pool,
        /*version*/ 3,
        deferrals_migration.description.as_ref(),
        deferrals_migration.checksum.as_ref(),
    )
    .await;
    sqlx::query("DROP TABLE thread_goal_continuation_deferrals")
        .execute(&pool)
        .await
        .expect("legacy deferrals table should be dropped");
    sqlx::query(
        "CREATE TABLE thread_goal_continuation_deferrals (thread_id TEXT PRIMARY KEY NOT NULL)",
    )
    .execute(&pool)
    .await
    .expect("malformed deferrals table should be created");
    pool.close().await;

    let runtime_migrator = runtime_goals_migrator();
    let err = fixture
        .sqlite
        .open_goals_db(&runtime_migrator, /*telemetry_override*/ None)
        .await
        .expect_err("malformed legacy schema should not be relabeled");
    assert!(
        err.chain().any(|source| matches!(
            source.downcast_ref::<MigrateError>(),
            Some(MigrateError::VersionMismatch(2))
        )),
        "unexpected migration error: {err:?}"
    );
}

#[tokio::test]
async fn rejects_unknown_goals_migration_checksum() {
    let fixture = GoalsDbFixture::new().await;
    let pool = fixture.open().await;
    GOALS_MIGRATOR
        .run(&pool)
        .await
        .expect("current goals migrations should apply");
    sqlx::query("UPDATE _sqlx_migrations SET checksum = ? WHERE version = 2")
        .bind([0_u8; 48].as_slice())
        .execute(&pool)
        .await
        .expect("unknown migration checksum should be recorded");
    pool.close().await;

    let err = fixture
        .sqlite
        .open_goals_db(&GOALS_MIGRATOR, /*telemetry_override*/ None)
        .await
        .expect_err("unknown migration checksum should be rejected");
    assert!(err.chain().any(|source| matches!(
        source.downcast_ref::<MigrateError>(),
        Some(MigrateError::VersionMismatch(2))
    )));
}
