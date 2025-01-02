use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use model::{AccountCreatedTime, UnixTime};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::{IntoParams, ToSchema};

/// Filter value for account created time. The value is max seconds since
/// account creation time.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    IntoParams,
    PartialEq,
    Eq,
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct AccountCreatedTimeFilter {
    pub value: i64,
}

impl AccountCreatedTimeFilter {
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.value
    }

    pub fn is_match(&self, account_created_time: AccountCreatedTime, current_time: &UnixTime) -> bool {
        let seconds_since_accont_creation = *current_time.as_i64() - *account_created_time.as_i64();
        seconds_since_accont_creation <= self.value
    }
}

diesel_i64_wrapper!(AccountCreatedTimeFilter);
