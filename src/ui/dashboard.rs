use gtk::prelude::*;

use crate::app::state::AppState;

pub fn build(state: AppState, start_session: impl Fn() + 'static) -> gtk::Box {
    let page = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(18)
        .margin_top(36)
        .margin_bottom(36)
        .margin_start(42)
        .margin_end(42)
        .build();
    page.append(
        &gtk::Label::builder()
            .label("Today")
            .xalign(0.0)
            .css_classes(["title-1"])
            .build(),
    );
    let summary = gtk::Label::builder()
        .xalign(0.0)
        .css_classes(["title-3"])
        .build();
    page.append(&summary);
    let start = gtk::Button::with_label("Start Today’s Session");
    start.add_css_class("suggested-action");
    start.connect_clicked(move |_| start_session());
    page.append(&start);
    page.append(
        &gtk::Label::builder()
            .label(format!(
                "Data: {}\nConfiguration: {}\nCache: {}",
                state.paths.data_dir().display(),
                state.paths.config_dir().display(),
                state.paths.cache_dir().display(),
            ))
            .xalign(0.0)
            .wrap(true)
            .build(),
    );
    refresh_summary(&state, &summary);
    {
        let state = state.clone();
        let summary = summary.clone();
        page.connect_map(move |_| refresh_summary(&state, &summary));
    }
    page
}

fn refresh_summary(state: &AppState, label: &gtk::Label) {
    match state.actions.review_summary() {
        Ok(summary) => label.set_text(&format!(
            "{} reviews due · {} new cards · {} cards learned · {} reviewed today",
            summary.due_count, summary.new_count, summary.learned_count, summary.reviewed_today
        )),
        Err(error) => label.set_text(&error.to_string()),
    }
}
