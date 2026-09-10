use adw::prelude::*;

use crate::{
    app::state::AppState,
    ui::navigation::{self, DESTINATIONS},
};

/// Constructs the Phase 0 GNOME application shell.
pub fn build(application: &adw::Application, state: AppState) {
    let window = adw::ApplicationWindow::builder()
        .application(application)
        .title("Washū")
        .default_width(1100)
        .default_height(720)
        .build();

    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&gtk::Label::builder().label("和習").build()));

    let sidebar = navigation::build_sidebar();
    let stack = gtk::Stack::builder()
        .hexpand(true)
        .vexpand(true)
        .transition_type(gtk::StackTransitionType::Crossfade)
        .transition_duration(150)
        .build();

    for (name, title) in DESTINATIONS {
        if name == "vocabulary" {
            stack.add_titled(
                &crate::ui::vocabulary::build(state.clone()),
                Some(name),
                title,
            );
        } else {
            stack.add_titled(&placeholder_page(title, &state), Some(name), title);
        }
    }

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .vexpand(true)
        .build();
    let sidebar_scroll = gtk::ScrolledWindow::builder()
        .child(&sidebar)
        .width_request(190)
        .vexpand(true)
        .build();
    content.append(&sidebar_scroll);
    content.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    content.append(&stack);

    sidebar.connect_row_selected(move |_, row| {
        if let Some(row) = row {
            stack.set_visible_child_name(row.widget_name().as_str());
        }
    });
    sidebar.select_row(sidebar.row_at_index(0).as_ref());

    let root = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    root.append(&header);
    root.append(&content);
    window.set_content(Some(&root));
    window.present();
}

pub fn build_error(application: &adw::Application, detail: &str) {
    let window = adw::ApplicationWindow::builder()
        .application(application)
        .title("Washū")
        .default_width(520)
        .default_height(240)
        .build();
    let page = adw::StatusPage::builder()
        .title("Washū could not start")
        .description(detail)
        .icon_name("dialog-error-symbolic")
        .build();
    window.set_content(Some(&page));
    window.present();
}

fn placeholder_page(title: &str, state: &AppState) -> adw::StatusPage {
    let description = if title == "Home" {
        format!(
            "Your learning workspace is ready.\n\nData: {}\nConfiguration: {}\nCache: {}",
            state.paths.data_dir().display(),
            state.paths.config_dir().display(),
            state.paths.cache_dir().display(),
        )
    } else {
        format!("{title} will be available in a later milestone.")
    };

    adw::StatusPage::builder()
        .title(title)
        .description(description.as_str())
        .icon_name("applications-education-symbolic")
        .build()
}
