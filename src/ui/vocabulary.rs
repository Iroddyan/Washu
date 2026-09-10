use std::{cell::RefCell, rc::Rc};

use gtk::prelude::*;

use crate::{
    app::state::AppState,
    domain::vocabulary::{Vocabulary, VocabularyInput},
};

/// A compact vocabulary workspace: search on the left, selected entry editor
/// on the right. All persistence goes through `AppActions`.
pub fn build(state: AppState) -> gtk::Box {
    let selected_id = Rc::new(RefCell::new(None));
    let entries = Rc::new(RefCell::new(Vec::<Vocabulary>::new()));
    let search = gtk::SearchEntry::builder()
        .placeholder_text("Search Japanese, reading, or meaning")
        .build();
    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Browse)
        .css_classes(["boxed-list"])
        .build();
    let status = gtk::Label::builder().xalign(0.0).wrap(true).build();
    let (form, fields) = editor_fields();

    let new_button = gtk::Button::with_label("New");
    let save_button = gtk::Button::with_label("Save");
    save_button.add_css_class("suggested-action");
    let delete_button = gtk::Button::with_label("Delete");
    delete_button.add_css_class("destructive-action");

    let sidebar = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(18)
        .margin_bottom(18)
        .margin_start(18)
        .margin_end(12)
        .width_request(340)
        .build();
    sidebar.append(&search);
    let list_scroll = gtk::ScrolledWindow::builder()
        .child(&list)
        .vexpand(true)
        .build();
    sidebar.append(&list_scroll);
    sidebar.append(&new_button);

    let editor = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .hexpand(true)
        .build();
    editor.append(
        &gtk::Label::builder()
            .label("Vocabulary entry")
            .xalign(0.0)
            .css_classes(["title-2"])
            .build(),
    );
    editor.append(&form);
    let buttons = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();
    buttons.append(&save_button);
    buttons.append(&delete_button);
    editor.append(&buttons);
    editor.append(&status);

    let root = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .vexpand(true)
        .build();
    root.append(&sidebar);
    root.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    root.append(&editor);

    refresh_list(&state, &list, &search, &entries, &status);

    {
        let state = state.clone();
        let list = list.clone();
        let search = search.clone();
        let entries = entries.clone();
        let status = status.clone();
        let selected_id = selected_id.clone();
        let fields = fields.clone();
        search.clone().connect_search_changed(move |_| {
            *selected_id.borrow_mut() = None;
            clear_fields(&fields);
            refresh_list(&state, &list, &search, &entries, &status);
        });
    }
    {
        let entries = entries.clone();
        let selected_id = selected_id.clone();
        let fields = fields.clone();
        list.connect_row_selected(move |_, row| {
            let Some(row) = row else { return };
            let vocabulary = entries.borrow();
            let Some(entry) = vocabulary.get(row.index() as usize) else {
                return;
            };
            *selected_id.borrow_mut() = Some(entry.id);
            set_fields(&fields, entry);
        });
    }
    {
        let fields = fields.clone();
        let selected_id = selected_id.clone();
        let status = status.clone();
        new_button.connect_clicked(move |_| {
            *selected_id.borrow_mut() = None;
            clear_fields(&fields);
            status.set_text("Creating a new vocabulary entry.");
            fields.expression.grab_focus();
        });
    }
    {
        let state = state.clone();
        let list = list.clone();
        let search = search.clone();
        let entries = entries.clone();
        let selected_id = selected_id.clone();
        let fields = fields.clone();
        let status = status.clone();
        save_button.connect_clicked(move |_| {
            let input = fields.input();
            let result = match *selected_id.borrow() {
                Some(id) => state
                    .actions
                    .update_vocabulary(id, input)
                    .map(|_| "Entry updated."),
                None => state
                    .actions
                    .create_vocabulary(input)
                    .map(|_| "Entry created."),
            };
            match result {
                Ok(message) => {
                    status.set_text(message);
                    *selected_id.borrow_mut() = None;
                    refresh_list(&state, &list, &search, &entries, &status);
                }
                Err(error) => status.set_text(&error.to_string()),
            }
        });
    }
    {
        let state = state.clone();
        let list = list.clone();
        let search = search.clone();
        let entries = entries.clone();
        let selected_id = selected_id.clone();
        let fields = fields.clone();
        let status = status.clone();
        delete_button.connect_clicked(move |_| {
            let Some(id) = *selected_id.borrow() else {
                status.set_text("Select an entry to delete.");
                return;
            };
            match state.actions.delete_vocabulary(id) {
                Ok(true) => {
                    *selected_id.borrow_mut() = None;
                    clear_fields(&fields);
                    status.set_text("Entry deleted.");
                    refresh_list(&state, &list, &search, &entries, &status);
                }
                Ok(false) => status.set_text("The selected entry no longer exists."),
                Err(error) => status.set_text(&error.to_string()),
            }
        });
    }

    root
}

#[derive(Clone)]
struct Fields {
    expression: gtk::Entry,
    reading: gtk::Entry,
    meaning: gtk::Entry,
    part_of_speech: gtk::Entry,
    jlpt_level: gtk::Entry,
    frequency_rank: gtk::Entry,
    notes: gtk::Entry,
}

impl Fields {
    fn input(&self) -> VocabularyInput {
        VocabularyInput {
            expression: self.expression.text().to_string(),
            reading: optional_text(&self.reading),
            meaning: self.meaning.text().to_string(),
            part_of_speech: optional_text(&self.part_of_speech),
            jlpt_level: optional_text(&self.jlpt_level),
            frequency_rank: self.frequency_rank.text().trim().parse().ok(),
            notes: optional_text(&self.notes),
        }
    }
}

fn editor_fields() -> (gtk::Grid, Fields) {
    let fields = Fields {
        expression: gtk::Entry::new(),
        reading: gtk::Entry::new(),
        meaning: gtk::Entry::new(),
        part_of_speech: gtk::Entry::new(),
        jlpt_level: gtk::Entry::new(),
        frequency_rank: gtk::Entry::new(),
        notes: gtk::Entry::new(),
    };
    fields
        .frequency_rank
        .set_input_purpose(gtk::InputPurpose::Digits);

    let form = gtk::Grid::builder()
        .row_spacing(10)
        .column_spacing(12)
        .build();
    for (row, (label, entry)) in [
        ("Expression", &fields.expression),
        ("Reading", &fields.reading),
        ("Meaning", &fields.meaning),
        ("Part of speech", &fields.part_of_speech),
        ("JLPT level", &fields.jlpt_level),
        ("Frequency rank", &fields.frequency_rank),
        ("Notes", &fields.notes),
    ]
    .into_iter()
    .enumerate()
    {
        let caption = gtk::Label::builder().label(label).xalign(1.0).build();
        entry.set_hexpand(true);
        form.attach(&caption, 0, row as i32, 1, 1);
        form.attach(entry, 1, row as i32, 1, 1);
    }
    (form, fields)
}

fn refresh_list(
    state: &AppState,
    list: &gtk::ListBox,
    search: &gtk::SearchEntry,
    entries: &Rc<RefCell<Vec<Vocabulary>>>,
    status: &gtk::Label,
) {
    match state.actions.search_vocabulary(search.text().as_str()) {
        Ok(results) => {
            while let Some(child) = list.first_child() {
                list.remove(&child);
            }
            for entry in &results {
                let subtitle = entry.reading.as_deref().unwrap_or("");
                let label = gtk::Label::builder()
                    .label(format!(
                        "{}  {subtitle}\n{}",
                        entry.expression, entry.meaning
                    ))
                    .xalign(0.0)
                    .wrap(true)
                    .margin_top(8)
                    .margin_bottom(8)
                    .margin_start(10)
                    .margin_end(10)
                    .build();
                list.append(&gtk::ListBoxRow::builder().child(&label).build());
            }
            *entries.borrow_mut() = results;
        }
        Err(error) => status.set_text(&error.to_string()),
    }
}

fn optional_text(entry: &gtk::Entry) -> Option<String> {
    let value = entry.text().trim().to_owned();
    (!value.is_empty()).then_some(value)
}

fn set_fields(fields: &Fields, entry: &Vocabulary) {
    fields.expression.set_text(&entry.expression);
    fields
        .reading
        .set_text(entry.reading.as_deref().unwrap_or(""));
    fields.meaning.set_text(&entry.meaning);
    fields
        .part_of_speech
        .set_text(entry.part_of_speech.as_deref().unwrap_or(""));
    fields
        .jlpt_level
        .set_text(entry.jlpt_level.as_deref().unwrap_or(""));
    fields.frequency_rank.set_text(
        &entry
            .frequency_rank
            .map(|rank| rank.to_string())
            .unwrap_or_default(),
    );
    fields.notes.set_text(entry.notes.as_deref().unwrap_or(""));
}

fn clear_fields(fields: &Fields) {
    fields.expression.set_text("");
    fields.reading.set_text("");
    fields.meaning.set_text("");
    fields.part_of_speech.set_text("");
    fields.jlpt_level.set_text("");
    fields.frequency_rank.set_text("");
    fields.notes.set_text("");
}
