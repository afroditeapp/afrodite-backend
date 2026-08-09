use diesel::{deserialize::FromSqlRow, expression::AsExpression, prelude::*, sql_types::BigInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::{UnixTime, diesel_i64_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::AccountIdDb;

/// Custom email message ID
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
pub struct CustomEmailId {
    pub eid: i64,
}

impl CustomEmailId {
    pub fn new(id: i64) -> Self {
        Self { eid: id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.eid
    }
}

impl TryFrom<i64> for CustomEmailId {
    type Error = String;

    fn try_from(id: i64) -> Result<Self, Self::Error> {
        Ok(Self { eid: id })
    }
}

impl AsRef<i64> for CustomEmailId {
    fn as_ref(&self) -> &i64 {
        &self.eid
    }
}

diesel_i64_wrapper!(CustomEmailId);

impl From<CustomEmailId> for i64 {
    fn from(value: CustomEmailId) -> Self {
        value.eid
    }
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::custom_email)]
#[diesel(check_for_backend(crate::Db))]
pub struct CustomEmailInternal {
    pub id: CustomEmailId,
    pub account_id_creator: Option<AccountIdDb>,
    pub sending_initiated_unix_time: Option<simple_backend_model::UnixTime>,
    pub sending_completed_unix_time: Option<simple_backend_model::UnixTime>,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::custom_email_translations)]
#[diesel(check_for_backend(crate::Db))]
pub struct CustomEmailTranslationInternal {
    pub locale: String,
    pub email_id: CustomEmailId,
    pub message_subject: String,
    pub message_body: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CustomEmailTranslation {
    pub subject: String,
    pub body: String,
    /// "default" or 2 letter country code.
    pub locale: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CustomEmail {
    pub id: CustomEmailId,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub sending_initiated_unix_time: Option<UnixTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub sending_completed_unix_time: Option<UnixTime>,
    pub translations: Vec<CustomEmailTranslation>,
}

#[derive(Debug, Clone, Deserialize, IntoParams)]
pub struct GetCustomEmailListParams {
    pub page: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateCustomEmail {
    pub id: CustomEmailId,
    /// Translation with "default" locale must exist.
    pub translations: Vec<CustomEmailTranslation>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GetCustomEmailConfig {
    #[serde(default)]
    pub email_body_is_html: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct SendCustomEmail {
    pub email_id: CustomEmailId,
    pub target_group: CustomEmailTargetGroup,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, ToSchema)]
pub enum CustomEmailTargetGroup {
    AllAccounts,
    AssociationMembers,
}
