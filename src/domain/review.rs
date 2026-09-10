use serde::{Deserialize, Serialize};

use crate::domain::mastery::CardState;

/// A vocabulary card that is ready to present in the review session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct DueVocabularyCard {
    pub vocabulary_id: i64,
    pub expression: String,
    pub reading: Option<String>,
    pub meaning: String,
    pub status: String,
    pub ease: f64,
    pub interval_days: i64,
    pub repetitions: i64,
    pub lapses: i64,
}

impl DueVocabularyCard {
    pub fn state(&self) -> CardState {
        CardState {
            status: self.status.clone(),
            ease: self.ease,
            interval_days: self.interval_days,
            repetitions: self.repetitions,
            lapses: self.lapses,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReviewSummary {
    pub due_count: i64,
    pub new_count: i64,
    pub learned_count: i64,
    pub reviewed_today: i64,
}
