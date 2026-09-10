use gtk::prelude::*;

use crate::app::state::AppState;

/// Minimal settings page for the v0.1 data-management promise.
pub fn build(state: AppState) -> gtk::Box {
    let status = gtk::Label::builder().xalign(0.0).wrap(true).build();
    let export = gtk::Button::with_label("Create Backup");
    export.add_css_class("suggested-action");

    let page = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(14)
        .margin_top(36)
        .margin_bottom(36)
        .margin_start(42)
        .margin_end(42)
        .build();
    page.append(
        &gtk::Label::builder()
            .label("Settings")
            .xalign(0.0)
            .css_classes(["title-1"])
            .build(),
    );
    page.append(
        &gtk::Label::builder()
            .label("Backup & Export")
            .xalign(0.0)
            .css_classes(["title-3"])
            .build(),
    );
    page.append(
        &gtk::Label::builder()
            .label(format!(
                "Create a standalone SQLite backup of your vocabulary and review history. Backups are saved to {}.",
                state.actions.backups_dir().display()
            ))
            .xalign(0.0)
            .wrap(true)
            .build(),
    );
    page.append(&export);
    page.append(&status);

    export.connect_clicked(move |_| match state.actions.create_backup() {
        Ok(path) => status.set_text(&format!("Backup created: {}", path.display())),
        Err(error) => status.set_text(&format!("Backup could not be created: {error}")),
    });

    page
}
