use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use model::UnixTime;
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::{IntoParams, ToSchema};

use crate::ProfileContentEditedTime;

#[derive(
    Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct ProfileEditedTime(UnixTime);

impl ProfileEditedTime {
    pub fn current_time() -> Self {
        Self(UnixTime::current_time())
    }
}

impl TryFrom<i64> for ProfileEditedTime {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self(UnixTime::new(value)))
    }
}

impl AsRef<i64> for ProfileEditedTime {
    fn as_ref(&self) -> &i64 {
        &self.0.ut
    }
}

diesel_i64_wrapper!(ProfileEditedTime);

/// Filter value for profile edited time. The value is max seconds since
/// profile edited time.
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
pub struct ProfileEditedTimeFilter {
    pub value: i64,
}

impl ProfileEditedTimeFilter {
    pub fn is_match(
        &self,
        profile_edited_time: ProfileEditedTime,
        content_edited_time: ProfileContentEditedTime,
        current_time: &UnixTime,
    ) -> bool {
        let latest_edit_time = *profile_edited_time
            .as_ref()
            .max(content_edited_time.as_ref());
        let seconds_since_latest_edit_time = *current_time.as_ref() - latest_edit_time;
        seconds_since_latest_edit_time <= self.value
    }
}

impl TryFrom<i64> for ProfileEditedTimeFilter {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self { value })
    }
}

impl AsRef<i64> for ProfileEditedTimeFilter {
    fn as_ref(&self) -> &i64 {
        &self.value
    }
}

diesel_i64_wrapper!(ProfileEditedTimeFilter);
