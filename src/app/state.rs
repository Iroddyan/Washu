use anyhow::Result;
use tokio::runtime::Runtime;

use crate::{database::Database, utils::AppPaths};

/// Services shared by application actions and UI presenters.
///
/// GTK widgets receive this state, but database access remains encapsulated by
/// the application and database layers.
#[derive(Clone)]
pub struct AppState {
    pub paths: AppPaths,
    pub database: Database,
}

impl AppState {
    pub fn initialize() -> Result<Self> {
        let paths = AppPaths::discover()?;
        paths.ensure_directories()?;

        let runtime = Runtime::new()?;
        let database = runtime.block_on(Database::open(paths.database_file()))?;

        Ok(Self { paths, database })
    }
}
