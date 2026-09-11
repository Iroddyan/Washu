use anyhow::{Context, Result, bail};

use crate::domain::{
    kanji::{Kanji, KanjiInput},
    vocabulary::Vocabulary,
};

use super::Database;

impl Database {
    /// Searches kanji by character, reading, or meaning, optionally limited to
    /// an exact JLPT level.
    pub async fn search_kanji(
        &self,
        search_term: &str,
        jlpt_level: Option<&str>,
    ) -> Result<Vec<Kanji>> {
        let pattern = format!("%{}%", search_term.trim());
        sqlx::query_as::<_, Kanji>(
            "SELECT id, character, meaning, on_readings, kun_readings, stroke_count, \
             jlpt_level, grade, notes FROM kanji \
             WHERE (character LIKE ?1 OR on_readings LIKE ?1 OR kun_readings LIKE ?1 \
                    OR meaning LIKE ?1) \
             AND (?2 IS NULL OR jlpt_level = ?2) \
             ORDER BY character COLLATE NOCASE, id",
        )
        .bind(pattern)
        .bind(jlpt_level)
        .fetch_all(self.pool())
        .await
        .context("searching kanji")
    }

    pub async fn create_kanji(&self, input: KanjiInput) -> Result<Kanji> {
        input.validate()?;
        let mut transaction = self
            .pool()
            .begin()
            .await
            .context("starting kanji transaction")?;
        let result = sqlx::query(
            "INSERT INTO kanji \
             (character, meaning, on_readings, kun_readings, stroke_count, jlpt_level, grade, notes) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .bind(input.character.trim())
        .bind(input.meaning.trim())
        .bind(input.on_readings)
        .bind(input.kun_readings)
        .bind(input.stroke_count)
        .bind(input.jlpt_level)
        .bind(input.grade)
        .bind(input.notes)
        .execute(&mut *transaction)
        .await
        .context("creating kanji")?;

        sqlx::query(
            "INSERT INTO user_kanji (kanji_id, status, due_at) \
             VALUES (?1, 'new', CURRENT_TIMESTAMP)",
        )
        .bind(result.last_insert_rowid())
        .execute(&mut *transaction)
        .await
        .context("initializing kanji learning state")?;

        transaction
            .commit()
            .await
            .context("committing kanji creation")?;
        self.kanji_by_id(result.last_insert_rowid()).await
    }

    pub async fn update_kanji(&self, id: i64, input: KanjiInput) -> Result<Kanji> {
        input.validate()?;
        let result = sqlx::query(
            "UPDATE kanji SET character = ?1, meaning = ?2, on_readings = ?3, \
             kun_readings = ?4, stroke_count = ?5, jlpt_level = ?6, grade = ?7, notes = ?8 \
             WHERE id = ?9",
        )
        .bind(input.character.trim())
        .bind(input.meaning.trim())
        .bind(input.on_readings)
        .bind(input.kun_readings)
        .bind(input.stroke_count)
        .bind(input.jlpt_level)
        .bind(input.grade)
        .bind(input.notes)
        .bind(id)
        .execute(self.pool())
        .await
        .context("updating kanji")?;
        if result.rows_affected() == 0 {
            bail!("Kanji entry no longer exists.");
        }
        self.kanji_by_id(id).await
    }

    pub async fn delete_kanji(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM kanji WHERE id = ?1")
            .bind(id)
            .execute(self.pool())
            .await
            .context("deleting kanji")?;
        Ok(result.rows_affected() == 1)
    }

    /// Idempotently associates a vocabulary entry with a kanji entry.
    pub async fn attach_kanji_to_vocabulary(
        &self,
        kanji_id: i64,
        vocabulary_id: i64,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO kanji_vocabulary (kanji_id, vocabulary_id) VALUES (?1, ?2)",
        )
        .bind(kanji_id)
        .bind(vocabulary_id)
        .execute(self.pool())
        .await
        .context("attaching vocabulary to kanji")?;
        Ok(())
    }

    pub async fn detach_kanji_from_vocabulary(
        &self,
        kanji_id: i64,
        vocabulary_id: i64,
    ) -> Result<bool> {
        let result =
            sqlx::query("DELETE FROM kanji_vocabulary WHERE kanji_id = ?1 AND vocabulary_id = ?2")
                .bind(kanji_id)
                .bind(vocabulary_id)
                .execute(self.pool())
                .await
                .context("detaching vocabulary from kanji")?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn vocabulary_for_kanji(&self, kanji_id: i64) -> Result<Vec<Vocabulary>> {
        sqlx::query_as::<_, Vocabulary>(
            "SELECT v.id, v.expression, v.reading, v.meaning, v.part_of_speech, v.jlpt_level, \
             v.frequency_rank, v.notes, v.created_at, v.updated_at \
             FROM vocabulary v JOIN kanji_vocabulary kv ON kv.vocabulary_id = v.id \
             WHERE kv.kanji_id = ?1 ORDER BY v.expression COLLATE NOCASE, v.id",
        )
        .bind(kanji_id)
        .fetch_all(self.pool())
        .await
        .context("loading vocabulary linked to kanji")
    }

    pub async fn kanji_for_vocabulary(&self, vocabulary_id: i64) -> Result<Vec<Kanji>> {
        sqlx::query_as::<_, Kanji>(
            "SELECT k.id, k.character, k.meaning, k.on_readings, k.kun_readings, k.stroke_count, \
             k.jlpt_level, k.grade, k.notes \
             FROM kanji k JOIN kanji_vocabulary kv ON kv.kanji_id = k.id \
             WHERE kv.vocabulary_id = ?1 ORDER BY k.character COLLATE NOCASE, k.id",
        )
        .bind(vocabulary_id)
        .fetch_all(self.pool())
        .await
        .context("loading kanji linked to vocabulary")
    }

    async fn kanji_by_id(&self, id: i64) -> Result<Kanji> {
        sqlx::query_as::<_, Kanji>(
            "SELECT id, character, meaning, on_readings, kun_readings, stroke_count, \
             jlpt_level, grade, notes FROM kanji WHERE id = ?1",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .context("loading kanji")?
        .ok_or_else(|| anyhow::anyhow!("Kanji entry no longer exists."))
    }
}

#[cfg(test)]
mod tests {
    use sqlx::Row;

    use crate::{
        database::Database,
        domain::{kanji::KanjiInput, vocabulary::VocabularyInput},
    };

    #[tokio::test]
    async fn kanji_entries_can_be_created_searched_filtered_updated_and_deleted() {
        let directory = tempfile::tempdir().expect("create temporary directory");
        let database = Database::open(&directory.path().join("washu.db"))
            .await
            .expect("open database");
        let created = database
            .create_kanji(KanjiInput {
                character: "食".into(),
                meaning: "eat; food".into(),
                on_readings: Some("ショク".into()),
                kun_readings: Some("た.べる".into()),
                stroke_count: Some(9),
                jlpt_level: Some("N5".into()),
                grade: Some(2),
                ..Default::default()
            })
            .await
            .expect("create kanji");
        database
            .create_kanji(KanjiInput {
                character: "語".into(),
                meaning: "language".into(),
                jlpt_level: Some("N4".into()),
                ..Default::default()
            })
            .await
            .expect("create second kanji");

        assert_eq!(database.search_kanji("べる", None).await.unwrap().len(), 1);
        assert_eq!(
            database.search_kanji("", Some("N5")).await.unwrap().len(),
            1
        );
        assert!(
            database
                .search_kanji("", Some("N3"))
                .await
                .unwrap()
                .is_empty()
        );

        let updated = database
            .update_kanji(
                created.id,
                KanjiInput {
                    character: "食".into(),
                    meaning: "to eat; food".into(),
                    jlpt_level: Some("N5".into()),
                    ..Default::default()
                },
            )
            .await
            .expect("update kanji");
        assert_eq!(updated.meaning, "to eat; food");

        let state = sqlx::query("SELECT status FROM user_kanji WHERE kanji_id = ?1")
            .bind(created.id)
            .fetch_one(database.pool())
            .await
            .expect("load initialized state");
        assert_eq!(state.get::<String, _>("status"), "new");

        assert!(database.delete_kanji(created.id).await.unwrap());
        assert!(database.search_kanji("食", None).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn kanji_vocabulary_links_can_be_attached_detached_and_cascade_deleted() {
        let directory = tempfile::tempdir().expect("create temporary directory");
        let database = Database::open(&directory.path().join("washu.db"))
            .await
            .expect("open database");
        let kanji = database
            .create_kanji(KanjiInput {
                character: "猫".into(),
                meaning: "cat".into(),
                ..Default::default()
            })
            .await
            .expect("create kanji");
        let vocabulary = database
            .create_vocabulary(VocabularyInput {
                expression: "猫".into(),
                meaning: "cat".into(),
                ..Default::default()
            })
            .await
            .expect("create vocabulary");

        database
            .attach_kanji_to_vocabulary(kanji.id, vocabulary.id)
            .await
            .expect("attach vocabulary");
        database
            .attach_kanji_to_vocabulary(kanji.id, vocabulary.id)
            .await
            .expect("reattach vocabulary idempotently");
        assert_eq!(
            database.vocabulary_for_kanji(kanji.id).await.unwrap().len(),
            1
        );
        assert_eq!(
            database
                .kanji_for_vocabulary(vocabulary.id)
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(
            database
                .detach_kanji_from_vocabulary(kanji.id, vocabulary.id)
                .await
                .unwrap()
        );
        assert!(
            !database
                .detach_kanji_from_vocabulary(kanji.id, vocabulary.id)
                .await
                .unwrap()
        );

        database
            .attach_kanji_to_vocabulary(kanji.id, vocabulary.id)
            .await
            .expect("reattach vocabulary");
        assert!(database.delete_kanji(kanji.id).await.unwrap());
        let links = sqlx::query("SELECT COUNT(*) AS count FROM kanji_vocabulary")
            .fetch_one(database.pool())
            .await
            .expect("count links");
        assert_eq!(links.get::<i64, _>("count"), 0);
        let states = sqlx::query("SELECT COUNT(*) AS count FROM user_kanji")
            .fetch_one(database.pool())
            .await
            .expect("count states");
        assert_eq!(states.get::<i64, _>("count"), 0);
    }
}
