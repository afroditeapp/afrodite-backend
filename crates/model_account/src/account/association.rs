use diesel::{Insertable, Queryable, Selectable};
use model::{AccountIdDb, StringResource};
use serde::{Deserialize, Serialize};
use simple_backend_model::{NonEmptyString, UnixTime};
use utoipa::ToSchema;

use crate::AccountId;

/// API response type for an association membership entry.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct AssociationMembership {
    pub creation_unix_time: UnixTime,
    pub edit_unix_time: UnixTime,
    pub full_name: Option<NonEmptyString>,
    pub domicile: Option<NonEmptyString>,
    pub membership_type: i16,
}

/// Admin API response for an entry with email included.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct AssociationMember {
    pub aid_member: AccountId,
    pub aid_creator: Option<AccountId>,
    pub aid_editor: Option<AccountId>,
    pub creation_unix_time: UnixTime,
    pub edit_unix_time: UnixTime,
    pub full_name: Option<NonEmptyString>,
    pub domicile: Option<NonEmptyString>,
    pub email: Option<NonEmptyString>,
    pub membership_type: i16,
}

/// Admin paged list query.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetAssociationMembersPage {
    pub page: i64,
}

/// Admin paged list response.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct AssociationMembersPage {
    pub entries: Vec<AssociationMember>,
}

/// User-facing input for creating/editing own membership entry.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct UpdateAssociationMembership {
    pub full_name: Option<NonEmptyString>,
    pub domicile: Option<NonEmptyString>,
    pub membership_type: i16,
}

/// Internal DB model for association_membership table.
#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::association_membership)]
#[diesel(check_for_backend(crate::Db))]
pub struct AssociationMembershipEntryInternal {
    pub account_id_member: AccountIdDb,
    pub account_id_creator: Option<AccountIdDb>,
    pub account_id_editor: Option<AccountIdDb>,
    pub creation_unix_time: UnixTime,
    pub edit_unix_time: UnixTime,
    pub full_name: Option<NonEmptyString>,
    pub domicile: Option<NonEmptyString>,
    pub membership_type: i16,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetAssociationMembership {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub membership: Option<AssociationMembership>,
}

impl From<Option<AssociationMembership>> for GetAssociationMembership {
    fn from(membership: Option<AssociationMembership>) -> Self {
        Self { membership }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetAssociationMember {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub member: Option<AssociationMember>,
}

impl From<Option<AssociationMember>> for GetAssociationMember {
    fn from(member: Option<AssociationMember>) -> Self {
        Self { member }
    }
}

/// Data export entry for association membership (no account IDs).
#[derive(Serialize)]
pub struct AssociationMembershipDataExportEntry {
    pub creation_unix_time: UnixTime,
    pub edit_unix_time: UnixTime,
    pub full_name: Option<NonEmptyString>,
    pub domicile: Option<NonEmptyString>,
    pub membership_type: i16,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetAssociationMembersOnlyInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub info_markdown: Option<StringResource>,
}
