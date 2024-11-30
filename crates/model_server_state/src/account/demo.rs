use diesel::{deserialize::FromSqlRow, expression::AsExpression};
use model::{AccountId, RefreshToken};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::{IntoParams, ToSchema};

use crate::schema_sqlite_types::BigInt;

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct DemoModePassword {
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Default)]
pub struct DemoModeLoginResult {
    /// This password is locked.
    pub locked: bool,
    pub token: Option<DemoModeLoginToken>,
}

impl DemoModeLoginResult {
    pub fn locked() -> Self {
        Self {
            locked: true,
            token: None,
        }
    }

    pub fn token(token: DemoModeLoginToken) -> Self {
        Self {
            locked: false,
            token: Some(token),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct DemoModeConfirmLogin {
    pub password: DemoModePassword,
    pub token: DemoModeLoginToken,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Default)]
pub struct DemoModeConfirmLoginResult {
    /// This password is locked.
    pub locked: bool,
    pub token: Option<DemoModeToken>,
}

impl DemoModeConfirmLoginResult {
    pub fn locked() -> Self {
        Self {
            locked: true,
            token: None,
        }
    }

    pub fn token(token: DemoModeToken) -> Self {
        Self {
            locked: false,
            token: Some(token),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, PartialEq)]
pub struct DemoModeLoginToken {
    pub token: String,
}

impl DemoModeLoginToken {
    pub fn generate_new() -> Self {
        Self {
            token: RefreshToken::generate_new().into_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, PartialEq)]
pub struct DemoModeToken {
    pub token: String,
}

impl DemoModeToken {
    pub fn generate_new() -> Self {
        Self {
            token: RefreshToken::generate_new().into_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct DemoModeLoginToAccount {
    pub token: DemoModeToken,
    pub aid: AccountId,
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
pub struct DemoModeId {
    pub id: i64,
}

impl DemoModeId {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.id
    }
}

diesel_i64_wrapper!(DemoModeId);

pub enum AccessibleAccountsInfo {
    All,
    Specific {
        config_file_accounts: Vec<AccountId>,
        demo_mode_id: DemoModeId,
    },
}
