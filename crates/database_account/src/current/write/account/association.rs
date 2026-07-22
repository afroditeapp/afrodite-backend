use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::AccountIdInternal;
use model_account::AssociationMembershipEntryInternal;
use simple_backend_utils::db::MyRunQueryDsl;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountAssociation);

impl CurrentWriteAccountAssociation<'_> {
    /// Upsert (insert or update) own association membership entry.
    pub fn upsert_own_entry(
        &mut self,
        data: AssociationMembershipEntryInternal,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::association_membership::dsl::*;

        insert_into(association_membership)
            .values(&data)
            .on_conflict(account_id_member)
            .do_update()
            .set((
                account_id_editor.eq(&data.account_id_editor),
                edit_unix_time.eq(&data.edit_unix_time),
                full_name.eq(&data.full_name),
                domicile.eq(&data.domicile),
                membership_type.eq(&data.membership_type),
            ))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    /// Remove own association membership entry.
    pub fn remove_own_entry(&mut self, id: AccountIdInternal) -> Result<(), DieselDatabaseError> {
        use crate::schema::association_membership::dsl::*;

        diesel::delete(association_membership.filter(account_id_member.eq(id.as_db_id())))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
