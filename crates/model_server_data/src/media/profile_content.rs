use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use model::ProfileContentVersion;
use serde::{Deserialize, Serialize};
use simple_backend_model::{UnixTime, diesel_i64_wrapper};
use utoipa::ToSchema;

#[derive(Clone, Copy)]
pub struct ProfileContentModificationMetadata {
    pub version: ProfileContentVersion,
    pub time: ProfileContentEditedTime,
}

impl ProfileContentModificationMetadata {
    pub fn generate() -> Self {
        Self {
            version: ProfileContentVersion::new_random(),
            time: ProfileContentEditedTime::current_time(),
        }
    }
}

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
pub struct ProfileContentEditedTime(UnixTime);

impl ProfileContentEditedTime {
    pub fn current_time() -> Self {
        Self(UnixTime::current_time())
    }
}

impl TryFrom<i64> for ProfileContentEditedTime {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self(UnixTime::new(value)))
    }
}

impl AsRef<i64> for ProfileContentEditedTime {
    fn as_ref(&self) -> &i64 {
        &self.0.ut
    }
}

diesel_i64_wrapper!(ProfileContentEditedTime);
