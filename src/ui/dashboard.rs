use gtk::prelude::*;

use crate::app::state::AppState;

pub fn build(state: AppState, start_session: impl Fn() + 'static) -> gtk::Box {
    let summary = state.actions.review_summary().unwrap_or_default();
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
    page.append(
        &gtk::Label::builder()
            .label(format!(
                "{} reviews due · {} new cards · {} cards learned · {} reviewed today",
                summary.due_count, summary.new_count, summary.learned_count, summary.reviewed_today
            ))
            .xalign(0.0)
            .css_classes(["title-3"])
            .build(),
    );
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
    page
}
