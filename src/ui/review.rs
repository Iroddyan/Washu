use std::{cell::RefCell, rc::Rc, time::Instant};

use gtk::prelude::*;

use crate::{app::state::AppState, domain::review::DueVocabularyCard, srs::grading::ReviewGrade};

pub fn build(state: AppState) -> gtk::Box {
    let card = Rc::new(RefCell::new(None::<DueVocabularyCard>));
    let shown_at = Rc::new(RefCell::new(None::<Instant>));
    let expression = gtk::Label::builder()
        .xalign(0.5)
        .wrap(true)
        .css_classes(["title-1"])
        .build();
    let answer = gtk::Label::builder().xalign(0.5).wrap(true).build();
    let reveal = gtk::Button::with_label("Reveal Answer");
    reveal.add_css_class("suggested-action");
    let status = gtk::Label::builder().xalign(0.5).wrap(true).build();
    let grades = grade_buttons();
    grades.set_sensitive(false);

    let page = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(20)
        .valign(gtk::Align::Center)
        .halign(gtk::Align::Center)
        .width_request(580)
        .margin_start(24)
        .margin_end(24)
        .build();
    page.append(
        &gtk::Label::builder()
            .label("Review")
            .xalign(0.5)
            .css_classes(["title-2"])
            .build(),
    );
    page.append(&expression);
    page.append(&reveal);
    page.append(&answer);
    page.append(&grades);
    page.append(&status);

    load_next(
        &state,
        &card,
        &shown_at,
        &expression,
        &answer,
        &reveal,
        &grades,
        &status,
    );

    {
        let card = card.clone();
        let answer = answer.clone();
        let reveal = reveal.clone();
        let grades = grades.clone();
        reveal
            .clone()
            .connect_clicked(move |_| reveal_answer(&card, &answer, &reveal, &grades));
    }
    for (button, grade) in grade_button_pairs(&grades) {
        let state = state.clone();
        let card = card.clone();
        let shown_at = shown_at.clone();
        let expression = expression.clone();
        let answer = answer.clone();
        let reveal = reveal.clone();
        let grades = grades.clone();
        let status = status.clone();
        button.connect_clicked(move |_| {
            apply_grade(
                &state,
                &card,
                &shown_at,
                &expression,
                &answer,
                &reveal,
                &grades,
                &status,
                grade,
            );
        });
    }

    let controller = gtk::EventControllerKey::new();
    {
        let state = state.clone();
        let card = card.clone();
        let shown_at = shown_at.clone();
        let expression = expression.clone();
        let answer = answer.clone();
        let reveal = reveal.clone();
        let grades = grades.clone();
        let status = status.clone();
        controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk::gdk::Key::space {
                reveal_answer(&card, &answer, &reveal, &grades);
                return gtk::glib::Propagation::Stop;
            }
            let grade = match key.to_unicode() {
                Some('1') => Some(ReviewGrade::Again),
                Some('2') => Some(ReviewGrade::Hard),
                Some('3') => Some(ReviewGrade::Good),
                Some('4') => Some(ReviewGrade::Easy),
                _ => None,
            };
            if let Some(grade) = grade {
                apply_grade(
                    &state,
                    &card,
                    &shown_at,
                    &expression,
                    &answer,
                    &reveal,
                    &grades,
                    &status,
                    grade,
                );
                gtk::glib::Propagation::Stop
            } else {
                gtk::glib::Propagation::Proceed
            }
        });
    }
    page.add_controller(controller);
    {
        let state = state.clone();
        let card = card.clone();
        let shown_at = shown_at.clone();
        let expression = expression.clone();
        let answer = answer.clone();
        let reveal = reveal.clone();
        let grades = grades.clone();
        let status = status.clone();
        page.connect_map(move |_| {
            load_next(
                &state,
                &card,
                &shown_at,
                &expression,
                &answer,
                &reveal,
                &grades,
                &status,
            );
        });
    }
    page
}

fn grade_buttons() -> gtk::Box {
    let box_ = gtk::Box::builder()
        .spacing(8)
        .halign(gtk::Align::Center)
        .build();
    for label in ["1 Again", "2 Hard", "3 Good", "4 Easy"] {
        box_.append(&gtk::Button::with_label(label));
    }
    box_
}

fn grade_button_pairs(grades: &gtk::Box) -> Vec<(gtk::Button, ReviewGrade)> {
    let mut buttons = Vec::new();
    let mut child = grades.first_child();
    for grade in [
        ReviewGrade::Again,
        ReviewGrade::Hard,
        ReviewGrade::Good,
        ReviewGrade::Easy,
    ] {
        let Some(widget) = child else { break };
        let button = widget
            .clone()
            .downcast::<gtk::Button>()
            .expect("grade button");
        child = widget.next_sibling();
        buttons.push((button, grade));
    }
    buttons
}

fn reveal_answer(
    card: &Rc<RefCell<Option<DueVocabularyCard>>>,
    answer: &gtk::Label,
    reveal: &gtk::Button,
    grades: &gtk::Box,
) {
    if let Some(card) = card.borrow().as_ref() {
        let reading = card.reading.as_deref().unwrap_or("—");
        answer.set_text(&format!("{reading}\n{}", card.meaning));
        answer.set_visible(true);
        reveal.set_visible(false);
        grades.set_sensitive(true);
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_grade(
    state: &AppState,
    card: &Rc<RefCell<Option<DueVocabularyCard>>>,
    shown_at: &Rc<RefCell<Option<Instant>>>,
    expression: &gtk::Label,
    answer: &gtk::Label,
    reveal: &gtk::Button,
    grades: &gtk::Box,
    status: &gtk::Label,
    grade: ReviewGrade,
) {
    if !grades.is_sensitive() {
        return;
    }
    let Some(assessed_card) = card.borrow().clone() else {
        return;
    };
    let response_ms = shown_at
        .borrow()
        .as_ref()
        .map(|instant| instant.elapsed().as_millis().min(i64::MAX as u128) as i64);
    match state
        .actions
        .grade_vocabulary(&assessed_card, grade, response_ms)
    {
        Ok(()) => load_next(
            state, card, shown_at, expression, answer, reveal, grades, status,
        ),
        Err(error) => status.set_text(&error.to_string()),
    }
}

#[allow(clippy::too_many_arguments)]
fn load_next(
    state: &AppState,
    card: &Rc<RefCell<Option<DueVocabularyCard>>>,
    shown_at: &Rc<RefCell<Option<Instant>>>,
    expression: &gtk::Label,
    answer: &gtk::Label,
    reveal: &gtk::Button,
    grades: &gtk::Box,
    status: &gtk::Label,
) {
    match state.actions.next_due_vocabulary() {
        Ok(Some(next)) => {
            expression.set_text(&next.expression);
            answer.set_text("");
            answer.set_visible(false);
            reveal.set_visible(true);
            grades.set_sensitive(false);
            status.set_text("Space reveals the answer. Use 1–4 to grade.");
            *shown_at.borrow_mut() = Some(Instant::now());
            *card.borrow_mut() = Some(next);
        }
        Ok(None) => {
            expression.set_text("You’re all caught up.");
            answer.set_text("");
            answer.set_visible(false);
            reveal.set_visible(false);
            grades.set_sensitive(false);
            status.set_text("Add vocabulary to create new review cards.");
            *shown_at.borrow_mut() = None;
            *card.borrow_mut() = None;
        }
        Err(error) => status.set_text(&error.to_string()),
    }
}
