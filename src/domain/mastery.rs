use serde::{Deserialize, Serialize};

/// Learning state shared by schedulable content. Content fields are not part of
/// this type so the scheduling engine remains reusable for future languages.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct CardState {
    pub status: String,
    pub ease: f64,
    pub interval_days: i64,
    pub repetitions: i64,
    pub lapses: i64,
}
