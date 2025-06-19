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
    pub fn new(value: i64) -> Self {
        Self(UnixTime::new(value))
    }

    pub fn as_i64(&self) -> &i64 {
        &self.0.ut
    }

    pub fn current_time() -> Self {
        Self(UnixTime::current_time())
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
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.value
    }

    pub fn is_match(
        &self,
        profile_edited_time: ProfileEditedTime,
        content_edited_time: Option<ProfileContentEditedTime>,
        current_time: &UnixTime,
    ) -> bool {
        let latest_edit_time = if let Some(content_edited_time) = content_edited_time {
            *profile_edited_time
                .as_i64()
                .max(content_edited_time.as_i64())
        } else {
            *profile_edited_time.as_i64()
        };
        let seconds_since_latest_edit_time = *current_time.as_i64() - latest_edit_time;
        seconds_since_latest_edit_time <= self.value
    }
}

diesel_i64_wrapper!(ProfileEditedTimeFilter);
