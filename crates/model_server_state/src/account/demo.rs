use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use model::{AccessToken, AccountId};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct DemoAccountLoginCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Default)]
pub struct DemoAccountLoginResult {
    /// This password is locked.
    pub locked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<DemoAccountToken>,
}

impl DemoAccountLoginResult {
    pub fn locked() -> Self {
        Self {
            locked: true,
            token: None,
        }
    }

    pub fn token(token: DemoAccountToken) -> Self {
        Self {
            locked: false,
            token: Some(token),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, PartialEq)]
pub struct DemoAccountToken {
    pub token: String,
}

impl DemoAccountToken {
    pub fn generate_new() -> Self {
        Self {
            token: AccessToken::generate_new().into_string(),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    IntoParams,
    PartialEq,
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
#[serde(transparent)]
pub struct DemoAccountId {
    pub id: i64,
}

impl TryFrom<i64> for DemoAccountId {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self { id: value })
    }
}

impl AsRef<i64> for DemoAccountId {
    fn as_ref(&self) -> &i64 {
        &self.id
    }
}

diesel_i64_wrapper!(DemoAccountId);

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Default)]
pub struct DemoAccountRegisterAccountResult {
    /// Account ID if registration was successful
    #[serde(skip_serializing_if = "Option::is_none")]
    aid: Option<AccountId>,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error: bool,

    /// True when the demo account has reached its maximum account limit
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_max_account_count: bool,
}

impl DemoAccountRegisterAccountResult {
    pub fn success(aid: AccountId) -> Self {
        Self {
            aid: Some(aid),
            ..Default::default()
        }
    }

    pub fn error_max_account_count() -> Self {
        Self {
            error: true,
            error_max_account_count: true,
            ..Default::default()
        }
    }
}

pub enum AccessibleAccountsInfo {
    All,
    Specific {
        config_file_accounts: Vec<AccountId>,
        demo_account_id: DemoAccountId,
    },
}
