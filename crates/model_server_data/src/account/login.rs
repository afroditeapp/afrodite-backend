use diesel::{
    prelude::*,
    sql_types::{SmallInt, Text},
};
use model::UnixTime;
use serde::{Deserialize, Serialize};
use simple_backend_model::{SimpleDieselEnum, diesel_string_wrapper};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = crate::schema::sign_in_with_info)]
#[diesel(check_for_backend(crate::Db))]
pub struct SignInWithInfo {
    pub apple_account_id: Option<AppleAccountId>,
    pub google_account_id: Option<GoogleAccountId>,
}

impl SignInWithInfo {
    pub fn some_sign_in_with_method_is_set(&self) -> bool {
        self.google_account_id.is_some() || self.apple_account_id.is_some()
    }
}

/// Sign in with provider type number.
///
/// Known values:
/// - `0` = Apple
/// - `1` = Google
#[derive(
    Debug,
    Default,
    Deserialize,
    Serialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    ToSchema,
    SimpleDieselEnum,
    diesel::FromSqlRow,
    diesel::AsExpression,
    num_enum::TryFromPrimitive,
)]
#[diesel(sql_type = SmallInt)]
#[repr(i16)]
pub enum SignInWithProviderTypeNumber {
    #[default]
    Apple = 0,
    Google = 1,
}

/// A single sign in with provider ID change history entry.
#[derive(Debug, Clone, Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::account_sign_in_with_history)]
#[diesel(check_for_backend(crate::Db))]
pub struct SignInWithHistoryEntry {
    pub provider_type_number: SignInWithProviderTypeNumber,
    pub old_id: Option<String>,
    pub new_id: Option<String>,
    pub change_unix_time: UnixTime,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, PartialEq, diesel::FromSqlRow, diesel::AsExpression,
)]
#[diesel(sql_type = Text)]
#[serde(transparent)]
pub struct GoogleAccountId(pub String);

impl GoogleAccountId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for GoogleAccountId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

impl AsRef<str> for GoogleAccountId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

diesel_string_wrapper!(GoogleAccountId);

#[derive(
    Debug, Serialize, Deserialize, Clone, PartialEq, diesel::FromSqlRow, diesel::AsExpression,
)]
#[diesel(sql_type = Text)]
#[serde(transparent)]
pub struct AppleAccountId(pub String);

impl AppleAccountId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for AppleAccountId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

impl AsRef<str> for AppleAccountId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

diesel_string_wrapper!(AppleAccountId);
