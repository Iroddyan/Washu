use anyhow::Result;
use std::sync::Arc;

use tokio::runtime::Runtime;

use crate::{app::actions::AppActions, database::Database, utils::AppPaths};

/// Services shared by application actions and UI presenters.
///
/// GTK widgets receive this state, but database access remains encapsulated by
/// the application and database layers.
#[derive(Clone)]
pub struct AppState {
    pub paths: AppPaths,
    pub actions: AppActions,
}

impl AppState {
    pub fn initialize() -> Result<Self> {
        let paths = AppPaths::discover()?;
        paths.ensure_directories()?;

        let runtime = Arc::new(Runtime::new()?);
        let database = runtime.block_on(Database::open(&paths.database_file()))?;

        Ok(Self {
            paths,
            actions: AppActions::new(database, runtime),
        })
    }
}
