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
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.value
    }

    pub fn is_match(
        &self,
        profile_created_time: InitialSetupCompletedTime,
        current_time: &UnixTime,
    ) -> bool {
        let seconds_since_account_creation =
            *current_time.as_i64() - *profile_created_time.as_i64();
        seconds_since_account_creation <= self.value
    }
}

diesel_i64_wrapper!(ProfileCreatedTimeFilter);
