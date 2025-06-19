use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::{Result, ResultExt};

define_current_read_commands!(CurrentReadAccountReport);

impl CurrentReadAccountReport<'_> {
    pub fn custom_reports_hash(&mut self) -> Result<Option<String>, DieselDatabaseError> {
        use crate::schema::custom_reports_file_hash::dsl::*;

        custom_reports_file_hash
            .filter(row_type.eq(0))
            .select(sha256_hash)
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)
    }
}
