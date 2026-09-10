//! Vocabulary repository operations will be added in the vocabulary milestone.

use super::Database;

impl Database {
    /// Verifies that the vocabulary repository boundary can obtain its pool
    /// without exposing SQLite to the UI layer.
    pub(crate) fn vocabulary_pool(&self) -> &sqlx::SqlitePool {
        self.pool()
    }
}
