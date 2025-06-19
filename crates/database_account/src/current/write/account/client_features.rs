use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{ExpressionMethods, insert_into, prelude::*};
use error_stack::Result;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountClientFeatures);

impl CurrentWriteAccountClientFeatures<'_> {
    pub fn upsert_client_features_file_hash(
        &mut self,
        sha256_client_features_file_hash: &str,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::client_features_file_hash::dsl::*;

        insert_into(client_features_file_hash)
            .values((
                row_type.eq(0),
                sha256_hash.eq(sha256_client_features_file_hash),
            ))
            .on_conflict(row_type)
            .do_update()
            .set(sha256_hash.eq(sha256_client_features_file_hash))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
