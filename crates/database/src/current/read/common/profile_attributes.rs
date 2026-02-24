use diesel::prelude::*;
use error_stack::Result;
use model::profile::{Attribute, AttributeOrderMode};
use simple_backend_database::diesel_db::DieselDatabaseError;

use crate::{IntoDatabaseError, define_current_read_commands};

define_current_read_commands!(CurrentReadCommonProfileAttributes);

impl CurrentReadCommonProfileAttributes<'_> {
    pub fn all_profile_attributes(&mut self) -> Result<Vec<Attribute>, DieselDatabaseError> {
        use model::schema::profile_attributes_schema_attribute::dsl::*;

        let rows: Vec<(i16, String)> = profile_attributes_schema_attribute
            .select((attribute_id, attribute_json))
            .order_by(attribute_id.asc())
            .load(self.conn())
            .into_db_error(())?;

        let mut attributes = Vec::with_capacity(rows.len());
        for (attr_id, json) in rows {
            let attr: Attribute = serde_json::from_str(&json).map_err(|e| {
                error_stack::report!(DieselDatabaseError::SerdeDeserialize)
                    .attach_printable(format!("Failed to deserialize attribute {attr_id}: {e}"))
            })?;
            attributes.push(attr);
        }

        Ok(attributes)
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
