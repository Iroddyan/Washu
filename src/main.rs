mod app;
mod database;
mod domain;
mod srs;
mod ui;
mod utils;

use gtk::prelude::*;

fn main() -> gtk::glib::ExitCode {
    let application = adw::Application::builder()
        .application_id("io.github.washu.Washu")
        .build();

    application.connect_activate(app::activate);
    application.run()
}
