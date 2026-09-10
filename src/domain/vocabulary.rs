use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// A dictionary entry owned by the learner's local content library.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Vocabulary {
    pub id: i64,
    pub expression: String,
    pub reading: Option<String>,
    pub meaning: String,
    pub part_of_speech: Option<String>,
    pub jlpt_level: Option<String>,
    pub frequency_rank: Option<i64>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Editable vocabulary fields. Learning state intentionally is not included:
/// it is owned by `user_vocabulary`, rather than by reference content.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VocabularyInput {
    pub expression: String,
    pub reading: Option<String>,
    pub meaning: String,
    pub part_of_speech: Option<String>,
    pub jlpt_level: Option<String>,
    pub frequency_rank: Option<i64>,
    pub notes: Option<String>,
}

impl VocabularyInput {
    pub fn validate(&self) -> Result<()> {
        if self.expression.trim().is_empty() {
            bail!("Expression is required.");
        }
        if self.meaning.trim().is_empty() {
            bail!("Meaning is required.");
        }
        if self.frequency_rank.is_some_and(|rank| rank < 1) {
            bail!("Frequency rank must be a positive number.");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::VocabularyInput;

    #[test]
    fn an_expression_and_meaning_are_required() {
        let input = VocabularyInput {
            expression: "  ".into(),
            meaning: "to eat".into(),
            ..Default::default()
        };

        assert!(input.validate().is_err());
    }

    #[test]
    fn frequency_rank_must_be_positive() {
        let input = VocabularyInput {
            expression: "食べる".into(),
            meaning: "to eat".into(),
            frequency_rank: Some(0),
            ..Default::default()
        };

        assert!(input.validate().is_err());
    }
}
