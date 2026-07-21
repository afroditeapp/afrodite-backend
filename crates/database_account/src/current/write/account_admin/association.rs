use database::{DieselDatabaseError, define_current_write_commands};
use diesel::{delete, insert_into, prelude::*, update};
use error_stack::Result;
use model_account::{
    AssociationMemberIdManual, NewAssociationMemberManualEntry, UpdateAssociationMemberManualEntry,
};

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountAssociationAdmin);

impl CurrentWriteAccountAssociationAdmin<'_> {
    /// Insert a new entry.
    pub fn insert_entry_manual(
        &mut self,
        data: NewAssociationMemberManualEntry,
    ) -> Result<AssociationMemberIdManual, DieselDatabaseError> {
        use crate::schema::association_membership_manual::dsl::*;

        let entry_id: AssociationMemberIdManual = insert_into(association_membership_manual)
            .values(data)
            .returning(id)
            .get_result(self.conn())
            .into_db_error(())?;

        Ok(entry_id)
    }

    /// Update an existing entry.
    pub fn update_entry_manual(
        &mut self,
        entry_id: AssociationMemberIdManual,
        data: UpdateAssociationMemberManualEntry,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::association_membership_manual::dsl::*;

        update(association_membership_manual)
            .filter(id.eq(entry_id))
            .set(data)
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    /// Delete an existing entry.
    pub fn delete_entry_manual(
        &mut self,
        entry_id: AssociationMemberIdManual,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::association_membership_manual::dsl::*;

        delete(association_membership_manual.filter(id.eq(entry_id)))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
