use diesel::sql_types::Text;
use model::{AccessToken, RefreshToken};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_string_wrapper;
use utoipa::ToSchema;

mod news;
pub use news::*;

mod login;
pub use login::*;

/// AccessToken and RefreshToken
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct AuthPair {
    pub refresh: RefreshToken,
    pub access: AccessToken,
}

impl AuthPair {
    pub fn new(refresh: RefreshToken, access: AccessToken) -> Self {
        Self { refresh, access }
    }
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    diesel::FromSqlRow,
    diesel::AsExpression,
    ToSchema,
)]
#[diesel(sql_type = Text)]
#[serde(try_from = "String")]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

diesel_string_wrapper!(EmailAddress);

impl TryFrom<String> for EmailAddress {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.trim() != value {
            return Err("Email address contains leading or trailing whitespace".to_string());
        }

        if !value.contains('@') {
            return Err("Email address does not have '@' character".to_string());
        }

        // Client uses 254 character limit. Let's assume that one character
        // is max 4 bytes.
        if value.len() > 254 * 4 {
            return Err("Email address is too long".to_string());
        }

        Ok(Self(value.to_lowercase()))
    }
}

impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
