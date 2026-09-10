use serde::{Deserialize, Serialize};

#[cfg(test)]
use anyhow::{Result, bail};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewGrade {
    Again,
    Hard,
    Good,
    Easy,
}

impl ReviewGrade {
    pub fn rating(self) -> i64 {
        match self {
            Self::Again => 1,
            Self::Hard => 2,
            Self::Good => 3,
            Self::Easy => 4,
        }
    }

    #[cfg(test)]
    pub fn from_rating(rating: i64) -> Result<Self> {
        match rating {
            1 => Ok(Self::Again),
            2 => Ok(Self::Hard),
            3 => Ok(Self::Good),
            4 => Ok(Self::Easy),
            _ => bail!("Review rating must be between 1 and 4."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ReviewGrade;

    #[test]
    fn ratings_round_trip_to_grades() {
        for rating in 1..=4 {
            assert_eq!(ReviewGrade::from_rating(rating).unwrap().rating(), rating);
        }
        assert!(ReviewGrade::from_rating(0).is_err());
        assert!(ReviewGrade::from_rating(5).is_err());
    }
}
