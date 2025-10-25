use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::{UnixTime, diesel_i64_wrapper};
use utoipa::ToSchema;

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Serialize,
    Deserialize,
    ToSchema,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct AccountCreatedTime(UnixTime);

impl AccountCreatedTime {
    pub fn current_time() -> Self {
        Self(UnixTime::current_time())
    }
}

impl TryFrom<i64> for AccountCreatedTime {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self(UnixTime::new(value)))
    }
}

impl AsRef<i64> for AccountCreatedTime {
    fn as_ref(&self) -> &i64 {
        &self.0.ut
    }
}

diesel_i64_wrapper!(AccountCreatedTime);

#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Serialize,
    Deserialize,
    ToSchema,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct InitialSetupCompletedTime(UnixTime);

impl InitialSetupCompletedTime {
    pub fn current_time() -> Self {
        Self(UnixTime::current_time())
    }
}

impl TryFrom<i64> for InitialSetupCompletedTime {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self(UnixTime::new(value)))
    }
}

impl AsRef<i64> for InitialSetupCompletedTime {
    fn as_ref(&self) -> &i64 {
        &self.0.ut
    }
}

diesel_i64_wrapper!(InitialSetupCompletedTime);
