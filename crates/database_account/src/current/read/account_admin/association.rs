use database::{DieselDatabaseError, define_current_read_commands};
use diesel::{alias, prelude::*};
use error_stack::Result;
use model_account::{AssociationMemberManual, AssociationMemberManualEntryInternal};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountAssociationAdmin);

impl CurrentReadAccountAssociationAdmin<'_> {
    /// Get all association membership manual entries.
    pub fn get_all_manual(&mut self) -> Result<Vec<AssociationMemberManual>, DieselDatabaseError> {
        use crate::schema::association_membership_manual;

        let (creator_aid, editor_aid) = alias!(
            crate::schema::account_id as creator_aid,
            crate::schema::account_id as editor_aid
        );

        let entries = association_membership_manual::table
            .left_outer_join(
                creator_aid.on(association_membership_manual::account_id_creator
                    .eq(creator_aid.field(crate::schema::account_id::id).nullable())),
            )
            .left_outer_join(
                editor_aid.on(association_membership_manual::account_id_editor
                    .eq(editor_aid.field(crate::schema::account_id::id).nullable())),
            )
            .order(association_membership_manual::id.asc())
            .select((
                AssociationMemberManualEntryInternal::as_select(),
                creator_aid
                    .field(crate::schema::account_id::uuid)
                    .nullable(),
                editor_aid.field(crate::schema::account_id::uuid).nullable(),
            ))
            .load(self.conn())
            .into_db_error(())?;

        let result = entries
            .into_iter()
            .map(
                |(internal, creator_uuid, editor_uuid)| AssociationMemberManual {
                    id: internal.id,
                    aid_creator: creator_uuid,
                    aid_editor: editor_uuid,
                    creation_unix_time: internal.creation_unix_time,
                    edit_unix_time: internal.edit_unix_time,
                    full_name: internal.full_name,
                    domicile: internal.domicile,
                    email: internal.email,
                    membership_type: internal.membership_type,
                },
            )
            .collect();

        Ok(result)
    }
}
