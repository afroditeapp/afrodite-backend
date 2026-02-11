use diesel::prelude::*;
use error_stack::Result;
use model::profile::AttributeOrderMode;
use simple_backend_database::diesel_db::DieselDatabaseError;

use crate::{IntoDatabaseError, define_current_read_commands};

define_current_read_commands!(CurrentReadCommonProfileAttributes);

impl CurrentReadCommonProfileAttributes<'_> {
    pub fn all_profile_attributes(
        &mut self,
    ) -> Result<Vec<(i16, String, String)>, DieselDatabaseError> {
        use model::schema::profile_attributes_schema_attribute::dsl::*;

        profile_attributes_schema_attribute
            .select((attribute_id, attribute_json, sha256_hash))
            .order_by(attribute_id.asc())
            .load(self.conn())
            .into_db_error(())
    }

    pub fn attribute_order_mode(
        &mut self,
    ) -> Result<Option<AttributeOrderMode>, DieselDatabaseError> {
        use model::schema::profile_attributes_schema::dsl::*;

        profile_attributes_schema
            .filter(row_type.eq(0))
            .select(attribute_order_mode)
            .first(self.conn())
            .optional()
            .into_db_error(())
    }

    pub fn profile_attributes_hash(&mut self) -> Result<Option<String>, DieselDatabaseError> {
        use model::schema::profile_attributes_schema_hash::dsl::*;

        profile_attributes_schema_hash
            .filter(row_type.eq(0))
            .select(sha256_hash)
            .first(self.conn())
            .optional()
            .into_db_error(())
    }
}
