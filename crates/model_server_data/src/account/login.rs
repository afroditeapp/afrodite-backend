use diesel::{prelude::Insertable, sql_types::Text};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_string_wrapper;

#[derive(Debug, Clone, PartialEq, Default, Insertable)]
#[diesel(table_name = crate::schema::sign_in_with_info)]
#[diesel(check_for_backend(crate::Db))]
#[diesel(treat_none_as_null = true)]
pub struct SignInWithInfo {
    pub apple_account_id: Option<AppleAccountId>,
    pub google_account_id: Option<GoogleAccountId>,
}

impl SignInWithInfo {
    pub fn some_sign_in_with_method_is_set(&self) -> bool {
        self.google_account_id.is_some() || self.apple_account_id.is_some()
    }
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
