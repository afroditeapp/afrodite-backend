use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use model::{InitialSetupCompletedTime, UnixTime};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::{IntoParams, ToSchema};

/// Filter value for profile created time. The value is max seconds since
/// profile creation time (initial setup completed).
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
pub struct ProfileCreatedTimeFilter {
    pub value: i64,
}

impl ProfileCreatedTimeFilter {
    pub fn is_match(
        &self,
        profile_created_time: InitialSetupCompletedTime,
        current_time: &UnixTime,
    ) -> bool {
        let seconds_since_account_creation =
            *current_time.as_ref() - *profile_created_time.as_ref();
        seconds_since_account_creation <= self.value
    }
}

impl TryFrom<i64> for ProfileCreatedTimeFilter {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self { value })
    }
}

impl AsRef<i64> for ProfileCreatedTimeFilter {
    fn as_ref(&self) -> &i64 {
        &self.value
    }
}

diesel_i64_wrapper!(ProfileCreatedTimeFilter);
