use diesel::{
    AsChangeset, AsExpression, FromSqlRow, Insertable, Queryable, Selectable, sql_types::BigInt,
};
use model::AccountIdDb;
use serde::{Deserialize, Serialize};
use simple_backend_model::{NonEmptyString, UnixTime, diesel_i64_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::AccountId;

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    IntoParams,
    PartialEq,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct AssociationMemberIdManual {
    pub id: i64,
}

impl AssociationMemberIdManual {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

impl TryFrom<i64> for AssociationMemberIdManual {
    type Error = String;

    fn try_from(id: i64) -> Result<Self, Self::Error> {
        Ok(Self { id })
    }
}

impl AsRef<i64> for AssociationMemberIdManual {
    fn as_ref(&self) -> &i64 {
        &self.id
    }
}

diesel_i64_wrapper!(AssociationMemberIdManual);

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct AssociationMemberManual {
    pub id: AssociationMemberIdManual,
    pub aid_creator: Option<AccountId>,
    pub aid_editor: Option<AccountId>,
    pub creation_unix_time: UnixTime,
    pub edit_unix_time: UnixTime,
    pub full_name: Option<NonEmptyString>,
    pub domicile: Option<NonEmptyString>,
    pub email: Option<NonEmptyString>,
    pub membership_type: i16,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct NewAssociationMemberManual {
    pub full_name: Option<NonEmptyString>,
    pub domicile: Option<NonEmptyString>,
    pub email: Option<NonEmptyString>,
    pub membership_type: i16,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct EditAssociationMemberManual {
    pub id: AssociationMemberIdManual,
    pub full_name: Option<NonEmptyString>,
    pub domicile: Option<NonEmptyString>,
    pub email: Option<NonEmptyString>,
    pub membership_type: i16,
}

/// Internal DB model for association_membership_manual table.
#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::association_membership_manual)]
#[diesel(check_for_backend(crate::Db))]
pub struct AssociationMemberManualEntryInternal {
    pub id: AssociationMemberIdManual,
    pub account_id_creator: Option<AccountIdDb>,
    pub account_id_editor: Option<AccountIdDb>,
    pub creation_unix_time: UnixTime,
    pub edit_unix_time: UnixTime,
    pub full_name: Option<NonEmptyString>,
    pub domicile: Option<NonEmptyString>,
    pub email: Option<NonEmptyString>,
    pub membership_type: i16,
}

/// New entry for DB insertion.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::association_membership_manual)]
pub struct NewAssociationMemberManualEntry {
    pub account_id_creator: Option<AccountIdDb>,
    pub account_id_editor: Option<AccountIdDb>,
    pub creation_unix_time: UnixTime,
    pub edit_unix_time: UnixTime,
    pub full_name: Option<NonEmptyString>,
    pub domicile: Option<NonEmptyString>,
    pub email: Option<NonEmptyString>,
    pub membership_type: i16,
}

/// Update fields for DB.
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::association_membership_manual)]
#[diesel(treat_none_as_null = true)]
pub struct UpdateAssociationMemberManualEntry {
    pub account_id_editor: AccountIdDb,
    pub edit_unix_time: UnixTime,
    pub full_name: Option<NonEmptyString>,
    pub domicile: Option<NonEmptyString>,
    pub email: Option<NonEmptyString>,
    pub membership_type: i16,
}
