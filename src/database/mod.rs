pub mod kanji;
pub mod reviews;
pub mod vocabulary;

use std::path::Path;

use anyhow::{Context, Result};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};

/// Database boundary for application services. UI code must use application
/// actions and repositories rather than accessing this pool directly.
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn open(path: &Path) -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePool::connect_with(options)
            .await
            .context("opening SQLite database")?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("running database migrations")?;

        Ok(Self { pool })
    }

    pub(crate) fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Creates a transactionally consistent standalone SQLite snapshot.
    pub async fn backup_to(&self, destination: &Path) -> Result<()> {
        let destination = destination
            .to_str()
            .context("converting backup path to text")?;
        let literal = destination.replace('\'', "''");
        sqlx::query(&format!("VACUUM INTO '{literal}'"))
            .execute(self.pool())
            .await
            .context("creating SQLite backup")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::Row;

    use super::Database;

    #[tokio::test]
    async fn reopening_a_database_keeps_the_initial_schema_applied_once() {
        let directory = tempfile::tempdir().expect("create temporary directory");
        let database_path = directory.path().join("washu.db");
        let database = Database::open(&database_path).await.expect("open database");
        database.pool().close().await;

        let database = Database::open(&database_path)
            .await
            .expect("reopen database");

        let row = sqlx::query(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'vocabulary'",
        )
        .fetch_one(database.pool())
        .await
        .expect("query schema");

        assert_eq!(row.get::<String, _>("name"), "vocabulary");

        let migrations = sqlx::query("SELECT COUNT(*) AS count FROM _sqlx_migrations")
            .fetch_one(database.pool())
            .await
            .expect("query migration history");
        assert_eq!(migrations.get::<i64, _>("count"), 1);
    }
}
