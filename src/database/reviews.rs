use anyhow::{Context, Result};
use sqlx::Row;

use crate::{
    domain::review::{DueVocabularyCard, ReviewSummary},
    srs::{grading::ReviewGrade, scheduler::schedule},
};

use super::Database;

impl Database {
    pub async fn next_due_vocabulary(&self) -> Result<Option<DueVocabularyCard>> {
        sqlx::query_as::<_, DueVocabularyCard>(
            "SELECT v.id AS vocabulary_id, v.expression, v.reading, v.meaning, \
             u.status, u.ease, u.interval_days, u.repetitions, u.lapses \
             FROM user_vocabulary u JOIN vocabulary v ON v.id = u.vocabulary_id \
             WHERE u.due_at IS NOT NULL AND u.due_at <= CURRENT_TIMESTAMP \
             ORDER BY CASE u.status WHEN 'learning' THEN 0 WHEN 'new' THEN 1 ELSE 2 END, \
             u.due_at, v.id LIMIT 1",
        )
        .fetch_optional(self.pool())
        .await
        .context("loading next due vocabulary card")
    }

    pub async fn grade_vocabulary(
        &self,
        card: &DueVocabularyCard,
        grade: ReviewGrade,
        response_ms: Option<i64>,
    ) -> Result<()> {
        let scheduled = schedule(&card.state(), grade);
        let due_modifier = format!("+{} days", scheduled.due_in_days);
        let mut transaction = self
            .pool()
            .begin()
            .await
            .context("starting review transaction")?;

        sqlx::query(
            "UPDATE user_vocabulary SET status = ?1, ease = ?2, interval_days = ?3, \
             repetitions = ?4, lapses = ?5, due_at = datetime('now', ?6), \
             last_reviewed_at = CURRENT_TIMESTAMP WHERE vocabulary_id = ?7",
        )
        .bind(&scheduled.state.status)
        .bind(scheduled.state.ease)
        .bind(scheduled.state.interval_days)
        .bind(scheduled.state.repetitions)
        .bind(scheduled.state.lapses)
        .bind(due_modifier)
        .bind(card.vocabulary_id)
        .execute(&mut *transaction)
        .await
        .context("updating vocabulary learning state")?;

        sqlx::query(
            "INSERT INTO reviews (item_type, item_id, direction, rating, response_ms) \
             VALUES ('vocabulary', ?1, 'recognition', ?2, ?3)",
        )
        .bind(card.vocabulary_id)
        .bind(grade.rating())
        .bind(response_ms)
        .execute(&mut *transaction)
        .await
        .context("recording review history")?;

        transaction.commit().await.context("committing review")
    }

    pub async fn review_summary(&self) -> Result<ReviewSummary> {
        let vocabulary = sqlx::query(
            "SELECT \
             COUNT(*) FILTER (WHERE due_at IS NOT NULL AND due_at <= CURRENT_TIMESTAMP) AS due_count, \
             COUNT(*) FILTER (WHERE status = 'new') AS new_count, \
             COUNT(*) FILTER (WHERE status = 'review') AS learned_count \
             FROM user_vocabulary",
        )
        .fetch_one(self.pool())
        .await
        .context("loading vocabulary progress")?;
        let reviews = sqlx::query(
            "SELECT COUNT(*) AS reviewed_today FROM reviews WHERE date(reviewed_at) = date('now')",
        )
        .fetch_one(self.pool())
        .await
        .context("loading today's review count")?;

        Ok(ReviewSummary {
            due_count: vocabulary.get("due_count"),
            new_count: vocabulary.get("new_count"),
            learned_count: vocabulary.get("learned_count"),
            reviewed_today: reviews.get("reviewed_today"),
        })
    }
}

#[cfg(test)]
mod tests {
    use sqlx::Row;

    use crate::{
        database::Database, domain::vocabulary::VocabularyInput, srs::grading::ReviewGrade,
    };

    #[tokio::test]
    async fn grading_a_due_card_updates_learning_state_and_history() {
        let directory = tempfile::tempdir().expect("create temporary directory");
        let database = Database::open(&directory.path().join("washu.db"))
            .await
            .expect("open database");
        database
            .create_vocabulary(VocabularyInput {
                expression: "猫".into(),
                meaning: "cat".into(),
                ..Default::default()
            })
            .await
            .expect("create entry");

        let card = database
            .next_due_vocabulary()
            .await
            .expect("load due card")
            .expect("card should be due");
        database
            .grade_vocabulary(&card, ReviewGrade::Good, Some(825))
            .await
            .expect("grade card");

        let state = sqlx::query("SELECT status, interval_days FROM user_vocabulary")
            .fetch_one(database.pool())
            .await
            .expect("load state");
        assert_eq!(state.get::<String, _>("status"), "review");
        assert_eq!(state.get::<i64, _>("interval_days"), 1);
        let reviews = sqlx::query("SELECT rating, response_ms FROM reviews")
            .fetch_one(database.pool())
            .await
            .expect("load review");
        assert_eq!(reviews.get::<i64, _>("rating"), 3);
        assert_eq!(reviews.get::<i64, _>("response_ms"), 825);
    }

    #[tokio::test]
    async fn graded_review_and_home_summary_survive_a_database_restart() {
        let directory = tempfile::tempdir().expect("create temporary directory");
        let database_path = directory.path().join("washu.db");
        let database = Database::open(&database_path).await.expect("open database");
        database
            .create_vocabulary(VocabularyInput {
                expression: "犬".into(),
                meaning: "dog".into(),
                ..Default::default()
            })
            .await
            .expect("create entry");
        let card = database
            .next_due_vocabulary()
            .await
            .expect("load due card")
            .expect("card should be due");
        database
            .grade_vocabulary(&card, ReviewGrade::Good, Some(612))
            .await
            .expect("grade card");
        database.pool().close().await;

        let restarted = Database::open(&database_path)
            .await
            .expect("restart database");
        assert!(
            restarted
                .next_due_vocabulary()
                .await
                .expect("load next card after restart")
                .is_none()
        );

        let summary = restarted.review_summary().await.expect("load home summary");
        assert_eq!(summary.due_count, 0);
        assert_eq!(summary.new_count, 0);
        assert_eq!(summary.learned_count, 1);
        assert_eq!(summary.reviewed_today, 1);

        let review =
            sqlx::query("SELECT item_type, item_id, direction, rating, response_ms FROM reviews")
                .fetch_one(restarted.pool())
                .await
                .expect("load persisted review");
        assert_eq!(review.get::<String, _>("item_type"), "vocabulary");
        assert_eq!(review.get::<i64, _>("item_id"), card.vocabulary_id);
        assert_eq!(review.get::<String, _>("direction"), "recognition");
        assert_eq!(review.get::<i64, _>("rating"), 3);
        assert_eq!(review.get::<i64, _>("response_ms"), 612);
    }
}
