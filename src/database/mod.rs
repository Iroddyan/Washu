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
}

#[cfg(test)]
mod tests {
    use sqlx::Row;

    use super::Database;

    #[tokio::test]
    async fn opening_a_database_applies_the_initial_schema() {
        let directory = tempfile::tempdir().expect("create temporary directory");
        let database = Database::open(&directory.path().join("washu.db"))
            .await
            .expect("open database");

        let row = sqlx::query(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'vocabulary'",
        )
        .fetch_one(database.pool())
        .await
        .expect("query schema");

        assert_eq!(row.get::<String, _>("name"), "vocabulary");
    }
}
