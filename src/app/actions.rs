use std::sync::Arc;

use anyhow::Result;
use tokio::runtime::Runtime;

use crate::{
    database::Database,
    domain::vocabulary::{Vocabulary, VocabularyInput},
};

/// Synchronous application commands used by the current GTK presentation.
///
/// The commands form the UI's only route to persistence. A later milestone can
/// make these operations background tasks without changing GTK's database
/// boundary or the vocabulary repository.
#[derive(Clone)]
pub struct AppActions {
    database: Database,
    runtime: Arc<Runtime>,
}

impl AppActions {
    pub fn new(database: Database, runtime: Arc<Runtime>) -> Self {
        Self { database, runtime }
    }

    pub fn search_vocabulary(&self, search_term: &str) -> Result<Vec<Vocabulary>> {
        self.runtime
            .block_on(self.database.search_vocabulary(search_term))
    }

    pub fn create_vocabulary(&self, input: VocabularyInput) -> Result<Vocabulary> {
        self.runtime
            .block_on(self.database.create_vocabulary(input))
    }

    pub fn update_vocabulary(&self, id: i64, input: VocabularyInput) -> Result<Vocabulary> {
        self.runtime
            .block_on(self.database.update_vocabulary(id, input))
    }

    pub fn delete_vocabulary(&self, id: i64) -> Result<bool> {
        self.runtime.block_on(self.database.delete_vocabulary(id))
    }
}
