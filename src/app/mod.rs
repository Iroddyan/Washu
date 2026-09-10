pub mod actions;
pub mod state;

use self::state::AppState;

/// Initializes application services before constructing the user interface.
pub fn activate(application: &adw::Application) {
    match AppState::initialize() {
        Ok(state) => crate::ui::window::build(application, state),
        Err(error) => {
            tracing_fallback(&format!("Washū could not start: {error:#}"));
            crate::ui::window::build_error(application, &error.to_string());
        }
    }
}

fn tracing_fallback(message: &str) {
    eprintln!("{message}");
}
