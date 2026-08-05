use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use model::AccountIdInternal;
use model_account::{AssociationMembership, AssociationMembershipEntryInternal};
use simple_backend_utils::Result;

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountAssociation);

impl CurrentReadAccountAssociation<'_> {
    /// Get own association membership entry for a user.
    pub fn get_own_entry(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<AssociationMembership>, DieselDatabaseError> {
        use crate::schema::association_membership::dsl::*;

        let result = association_membership
            .filter(account_id_member.eq(id.as_db_id()))
            .select(AssociationMembershipEntryInternal::as_select())
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(result.map(|internal| AssociationMembership {
            creation_unix_time: internal.creation_unix_time,
            edit_unix_time: internal.edit_unix_time,
            full_name: internal.full_name,
            domicile: internal.domicile,
            membership_type: internal.membership_type,
        }))
    }
}
