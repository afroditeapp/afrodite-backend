use diesel::{ExpressionMethods, RunQueryDsl, insert_into};
use error_stack::Result;
use model::profile::AttributeOrderMode;
use simple_backend_database::diesel_db::DieselDatabaseError;
use simple_backend_utils::db::MyRunQueryDsl;

use crate::{IntoDatabaseError, define_current_write_commands};

define_current_write_commands!(CurrentWriteCommonProfileAttributes);

impl CurrentWriteCommonProfileAttributes<'_> {
    pub fn delete_all_profile_attributes(&mut self) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_attributes_schema_attribute::dsl::*;

        diesel::delete(profile_attributes_schema_attribute)
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn insert_profile_attribute(
        &mut self,
        attr_id: i16,
        json: &str,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_attributes_schema_attribute::dsl::*;

        insert_into(profile_attributes_schema_attribute)
            .values((attribute_id.eq(attr_id), attribute_json.eq(json)))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn upsert_profile_attribute(
        &mut self,
        attr_id: i16,
        json: &str,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_attributes_schema_attribute::dsl::*;

        insert_into(profile_attributes_schema_attribute)
            .values((attribute_id.eq(attr_id), attribute_json.eq(json)))
            .on_conflict(attribute_id)
            .do_update()
            .set(attribute_json.eq(json))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn upsert_profile_attributes_order_mode(
        &mut self,
        mode: AttributeOrderMode,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_attributes_schema::dsl::*;

        insert_into(profile_attributes_schema)
            .values((row_type.eq(0), attribute_order_mode.eq(mode)))
            .on_conflict(row_type)
            .do_update()
            .set(attribute_order_mode.eq(mode))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn upsert_profile_attributes_hash(
        &mut self,
        hash: &str,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::profile_attributes_schema_hash::dsl::*;

        insert_into(profile_attributes_schema_hash)
            .values((row_type.eq(0), sha256_hash.eq(hash)))
            .on_conflict(row_type)
            .do_update()
            .set(sha256_hash.eq(hash))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
