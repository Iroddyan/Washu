use crate::domain::vocabulary::{Vocabulary, VocabularyInput};
use anyhow::{Context, Result, bail};

use super::Database;

impl Database {
    pub async fn search_vocabulary(&self, search_term: &str) -> Result<Vec<Vocabulary>> {
        let pattern = format!("%{}%", search_term.trim());
        sqlx::query_as::<_, Vocabulary>(
            "SELECT id, expression, reading, meaning, part_of_speech, jlpt_level, \
             frequency_rank, notes, created_at, updated_at \
             FROM vocabulary \
             WHERE expression LIKE ?1 OR reading LIKE ?1 OR meaning LIKE ?1 \
             ORDER BY expression COLLATE NOCASE, id",
        )
        .bind(pattern)
        .fetch_all(self.pool())
        .await
        .context("searching vocabulary")
    }

    pub async fn create_vocabulary(&self, input: VocabularyInput) -> Result<Vocabulary> {
        input.validate()?;
        let result = sqlx::query(
            "INSERT INTO vocabulary \
             (expression, reading, meaning, part_of_speech, jlpt_level, frequency_rank, notes) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .bind(input.expression.trim())
        .bind(input.reading)
        .bind(input.meaning.trim())
        .bind(input.part_of_speech)
        .bind(input.jlpt_level)
        .bind(input.frequency_rank)
        .bind(input.notes)
        .execute(self.pool())
        .await
        .context("creating vocabulary")?;

        self.vocabulary_by_id(result.last_insert_rowid()).await
    }

    pub async fn update_vocabulary(&self, id: i64, input: VocabularyInput) -> Result<Vocabulary> {
        input.validate()?;
        let result = sqlx::query(
            "UPDATE vocabulary SET expression = ?1, reading = ?2, meaning = ?3, \
             part_of_speech = ?4, jlpt_level = ?5, frequency_rank = ?6, notes = ?7, \
             updated_at = CURRENT_TIMESTAMP WHERE id = ?8",
        )
        .bind(input.expression.trim())
        .bind(input.reading)
        .bind(input.meaning.trim())
        .bind(input.part_of_speech)
        .bind(input.jlpt_level)
        .bind(input.frequency_rank)
        .bind(input.notes)
        .bind(id)
        .execute(self.pool())
        .await
        .context("updating vocabulary")?;

        if result.rows_affected() == 0 {
            bail!("Vocabulary entry no longer exists.");
        }
        self.vocabulary_by_id(id).await
    }

    pub async fn delete_vocabulary(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM vocabulary WHERE id = ?1")
            .bind(id)
            .execute(self.pool())
            .await
            .context("deleting vocabulary")?;
        Ok(result.rows_affected() == 1)
    }

    async fn vocabulary_by_id(&self, id: i64) -> Result<Vocabulary> {
        sqlx::query_as::<_, Vocabulary>(
            "SELECT id, expression, reading, meaning, part_of_speech, jlpt_level, \
             frequency_rank, notes, created_at, updated_at FROM vocabulary WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .context("loading vocabulary")?
        .ok_or_else(|| anyhow::anyhow!("Vocabulary entry no longer exists."))
    }
}

#[cfg(test)]
mod tests {
    use crate::{database::Database, domain::vocabulary::VocabularyInput};

    #[tokio::test]
    async fn vocabulary_entries_can_be_created_searched_updated_and_deleted() {
        let directory = tempfile::tempdir().expect("create temporary directory");
        let database = Database::open(&directory.path().join("washu.db"))
            .await
            .expect("open database");
        let input = VocabularyInput {
            expression: "食べる".into(),
            reading: Some("たべる".into()),
            meaning: "to eat".into(),
            ..Default::default()
        };

        let created = database
            .create_vocabulary(input)
            .await
            .expect("create entry");
        assert_eq!(database.search_vocabulary("たべ").await.unwrap().len(), 1);

        let updated = database
            .update_vocabulary(
                created.id,
                VocabularyInput {
                    expression: "食べる".into(),
                    meaning: "to eat (food)".into(),
                    ..Default::default()
                },
            )
            .await
            .expect("update entry");
        assert_eq!(updated.meaning, "to eat (food)");
        assert!(database.delete_vocabulary(created.id).await.unwrap());
        assert!(
            database
                .search_vocabulary("食べる")
                .await
                .unwrap()
                .is_empty()
        );
    }
}
