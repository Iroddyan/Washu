use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};

use crate::{database::Database, utils::AppPaths};

/// Creates portable SQLite snapshots without exposing persistence mechanics to
/// the user interface.
#[derive(Clone)]
pub struct BackupService {
    database: Database,
    backups_dir: PathBuf,
}

impl BackupService {
    pub fn new(database: Database, paths: &AppPaths) -> Self {
        Self {
            database,
            backups_dir: paths.backups_dir(),
        }
    }

    pub async fn create_backup(&self) -> Result<PathBuf> {
        fs::create_dir_all(&self.backups_dir)
            .with_context(|| format!("creating {}", self.backups_dir.display()))?;
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("reading system clock")?
            .as_millis();
        let destination = self.backups_dir.join(format!("washu-{created_at}.sqlite3"));
        self.database.backup_to(&destination).await?;
        Ok(destination)
    }

    pub fn backups_dir(&self) -> &std::path::Path {
        &self.backups_dir
    }
}

#[cfg(test)]
mod tests {
    use sqlx::Row;

    use crate::{
        database::Database, domain::vocabulary::VocabularyInput, srs::grading::ReviewGrade,
        utils::AppPaths,
    };

    use super::BackupService;

    #[tokio::test]
    async fn backup_is_a_standalone_sqlite_database() {
        let directory = tempfile::tempdir().expect("create temporary directory");
        let paths = AppPaths::for_testing(directory.path());
        paths.ensure_directories().expect("create app directories");
        let database = Database::open(&paths.database_file())
            .await
            .expect("open database");
        database
            .create_vocabulary(VocabularyInput {
                expression: "猫".into(),
                meaning: "cat".into(),
                ..Default::default()
            })
            .await
            .expect("create vocabulary");
        let card = database
            .next_due_vocabulary()
            .await
            .expect("load due card")
            .expect("card should be due");
        database
            .grade_vocabulary(&card, ReviewGrade::Good, Some(451))
            .await
            .expect("grade card");

        let backup = BackupService::new(database, &paths)
            .create_backup()
            .await
            .expect("create backup");
        let snapshot = Database::open(&backup).await.expect("open backup");

        assert_eq!(snapshot.search_vocabulary("猫").await.unwrap().len(), 1);
        let summary = snapshot
            .review_summary()
            .await
            .expect("load backup summary");
        assert_eq!(summary.learned_count, 1);
        assert_eq!(summary.reviewed_today, 1);

        let review = sqlx::query("SELECT item_type, item_id, rating, response_ms FROM reviews")
            .fetch_one(snapshot.pool())
            .await
            .expect("load review from backup");
        assert_eq!(review.get::<String, _>("item_type"), "vocabulary");
        assert_eq!(review.get::<i64, _>("item_id"), card.vocabulary_id);
        assert_eq!(review.get::<i64, _>("rating"), 3);
        assert_eq!(review.get::<i64, _>("response_ms"), 451);
    }
}
