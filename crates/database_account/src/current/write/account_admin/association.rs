use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::AccountIdInternal;
use simple_backend_utils::db::MyRunQueryDsl;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountAssociationAdmin);

impl CurrentWriteAccountAssociationAdmin<'_> {
    pub fn upsert_manual_registry(&mut self, value: &str) -> Result<(), DieselDatabaseError> {
        use crate::schema::manual_association_membership_registry::dsl::*;

        insert_into(manual_association_membership_registry)
            .values((row_type.eq(0), registry.eq(value)))
            .on_conflict(row_type)
            .do_update()
            .set(registry.eq(value))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn delete_entry(
        &mut self,
        member_id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::association_membership::dsl::*;

        diesel::delete(association_membership.filter(account_id_member.eq(member_id.as_db_id())))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
