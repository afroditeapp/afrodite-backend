use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};

define_current_read_commands!(CurrentReadAccountClientFeatures);

impl CurrentReadAccountClientFeatures<'_> {
    pub fn client_features_hash(&mut self) -> Result<Option<String>, DieselDatabaseError> {
        use crate::schema::client_features_file_hash::dsl::*;

        client_features_file_hash
            .filter(row_type.eq(0))
            .select(sha256_hash)
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)
    }
}
