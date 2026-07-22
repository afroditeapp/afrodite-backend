use database::{DieselDatabaseError, define_current_read_commands};
use diesel::{alias, prelude::*};
use error_stack::Result;
use model::{AccountId, UnixTime};
use model_account::{
    AssociationMember, AssociationMemberManual, AssociationMemberManualEntryInternal,
    AssociationMembersPage, GetAssociationMembersPage,
};
use simple_backend_utils::string::NonEmptyString;

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

    /// Get a paged list of association membership entries with email.
    pub fn get_page(
        &mut self,
        query: &GetAssociationMembersPage,
    ) -> Result<AssociationMembersPage, DieselDatabaseError> {
        use crate::schema::{account_email_address_state, association_membership::dsl::*};

        let (member_aid, creator_aid, editor_aid) = alias!(
            crate::schema::account_id as member_aid,
            crate::schema::account_id as creator_aid,
            crate::schema::account_id as editor_aid
        );

        let entries = association_membership
            .inner_join(
                member_aid
                    .on(account_id_member.eq(member_aid.field(crate::schema::account_id::id))),
            )
            .left_outer_join(creator_aid.on(
                account_id_creator.eq(creator_aid.field(crate::schema::account_id::id).nullable()),
            ))
            .left_outer_join(editor_aid.on(
                account_id_editor.eq(editor_aid.field(crate::schema::account_id::id).nullable()),
            ))
            .left_outer_join(
                account_email_address_state::table
                    .on(account_id_member.eq(account_email_address_state::account_id)),
            )
            .order(account_id_member.asc())
            .offset(query.page * 25)
            .limit(25)
            .select((
                creation_unix_time,
                edit_unix_time,
                full_name,
                domicile,
                membership_type,
                member_aid.field(crate::schema::account_id::uuid),
                creator_aid
                    .field(crate::schema::account_id::uuid)
                    .nullable(),
                editor_aid.field(crate::schema::account_id::uuid).nullable(),
                account_email_address_state::email.nullable(),
            ))
            .load::<(
                UnixTime,
                UnixTime,
                Option<String>,
                Option<String>,
                i16,
                AccountId,
                Option<AccountId>,
                Option<AccountId>,
                Option<String>,
            )>(self.conn())
            .into_db_error(())?;

        let entries = entries
            .into_iter()
            .map(
                |(
                    created,
                    edited,
                    name,
                    dom,
                    mtype,
                    member_uuid,
                    creator_uuid,
                    editor_uuid,
                    email,
                )| {
                    AssociationMember {
                        aid_member: member_uuid,
                        aid_creator: creator_uuid,
                        aid_editor: editor_uuid,
                        creation_unix_time: created,
                        edit_unix_time: edited,
                        full_name: name.and_then(NonEmptyString::from_string),
                        domicile: dom.and_then(NonEmptyString::from_string),
                        email: email.and_then(NonEmptyString::from_string),
                        membership_type: mtype,
                    }
                },
            )
            .collect();

        Ok(AssociationMembersPage { entries })
    }
}
