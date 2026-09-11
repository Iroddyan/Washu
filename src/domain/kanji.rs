use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// A character entry owned by the learner's local content library.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Kanji {
    pub id: i64,
    pub character: String,
    pub meaning: String,
    pub on_readings: Option<String>,
    pub kun_readings: Option<String>,
    pub stroke_count: Option<i64>,
    pub jlpt_level: Option<String>,
    pub grade: Option<i64>,
    pub notes: Option<String>,
}

/// Editable kanji fields. Learning state remains in `user_kanji`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KanjiInput {
    pub character: String,
    pub meaning: String,
    pub on_readings: Option<String>,
    pub kun_readings: Option<String>,
    pub stroke_count: Option<i64>,
    pub jlpt_level: Option<String>,
    pub grade: Option<i64>,
    pub notes: Option<String>,
}

impl KanjiInput {
    pub fn validate(&self) -> Result<()> {
        let character = self.character.trim();
        if character.is_empty() {
            bail!("Character is required.");
        }
        if character.chars().count() != 1 {
            bail!("A kanji entry must contain exactly one character.");
        }
        if self.meaning.trim().is_empty() {
            bail!("Meaning is required.");
        }
        if self.stroke_count.is_some_and(|count| count < 1) {
            bail!("Stroke count must be a positive number.");
        }
        if self.grade.is_some_and(|grade| grade < 1) {
            bail!("Grade must be a positive number.");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::KanjiInput;

    #[test]
    fn a_single_character_and_meaning_are_required() {
        let input = KanjiInput {
            character: "日本".into(),
            meaning: "Japan".into(),
            ..Default::default()
        };

        assert!(input.validate().is_err());
    }

    #[test]
    fn stroke_count_and_grade_must_be_positive() {
        let input = KanjiInput {
            character: "日".into(),
            meaning: "sun".into(),
            stroke_count: Some(0),
            grade: Some(0),
            ..Default::default()
        };

        assert!(input.validate().is_err());
    }
}
