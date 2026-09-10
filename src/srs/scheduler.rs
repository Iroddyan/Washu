use crate::{domain::mastery::CardState, srs::grading::ReviewGrade};

/// Result of one deterministic scheduling decision.
#[derive(Clone, Debug, PartialEq)]
pub struct ScheduledCard {
    pub state: CardState,
    pub due_in_days: i64,
}

/// The intentionally small MVP scheduler.
///
/// It only depends on existing card state and the learner's grade. The database
/// turns the returned day offset into a persisted due timestamp.
pub fn schedule(current: &CardState, grade: ReviewGrade) -> ScheduledCard {
    let mut state = current.clone();
    let due_in_days = match grade {
        ReviewGrade::Again => {
            state.status = "learning".into();
            state.ease = (state.ease - 0.20).max(1.3);
            state.interval_days = 0;
            state.repetitions = 0;
            state.lapses += 1;
            0
        }
        ReviewGrade::Hard => {
            state.status = "review".into();
            state.ease = (state.ease - 0.15).max(1.3);
            state.interval_days = state.interval_days.max(1);
            state.repetitions += 1;
            state.interval_days
        }
        ReviewGrade::Good => {
            state.status = "review".into();
            state.interval_days = if state.repetitions == 0 {
                1
            } else {
                ((state.interval_days as f64 * state.ease).ceil() as i64).max(1)
            };
            state.repetitions += 1;
            state.interval_days
        }
        ReviewGrade::Easy => {
            state.status = "review".into();
            state.ease += 0.15;
            state.interval_days = if state.repetitions == 0 {
                4
            } else {
                ((state.interval_days as f64 * state.ease).ceil() as i64).max(1)
            };
            state.repetitions += 1;
            state.interval_days
        }
    };

    ScheduledCard { state, due_in_days }
}

#[cfg(test)]
mod tests {
    use crate::{
        domain::mastery::CardState,
        srs::{grading::ReviewGrade, scheduler::schedule},
    };

    fn new_card() -> CardState {
        CardState {
            status: "new".into(),
            ease: 2.5,
            interval_days: 0,
            repetitions: 0,
            lapses: 0,
        }
    }

    #[test]
    fn good_starts_a_new_card_on_a_one_day_interval() {
        let scheduled = schedule(&new_card(), ReviewGrade::Good);
        assert_eq!(scheduled.due_in_days, 1);
        assert_eq!(scheduled.state.status, "review");
        assert_eq!(scheduled.state.repetitions, 1);
    }

    #[test]
    fn easy_starts_a_new_card_on_a_four_day_interval() {
        let scheduled = schedule(&new_card(), ReviewGrade::Easy);
        assert_eq!(scheduled.due_in_days, 4);
        assert_eq!(scheduled.state.ease, 2.65);
    }

    #[test]
    fn again_resets_the_interval_and_records_a_lapse() {
        let scheduled = schedule(
            &CardState {
                interval_days: 6,
                repetitions: 3,
                ..new_card()
            },
            ReviewGrade::Again,
        );
        assert_eq!(scheduled.due_in_days, 0);
        assert_eq!(scheduled.state.lapses, 1);
        assert_eq!(scheduled.state.repetitions, 0);
    }
}
