use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_wrapper;
use utoipa::ToSchema;

use crate::sync_version_wrappers;

sync_version_wrappers!(NewsSyncVersion,);

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct UnreadNewsCount {
    pub c: i64,
}

impl TryFrom<i64> for UnreadNewsCount {
    type Error = String;

    fn try_from(count: i64) -> Result<Self, Self::Error> {
        Ok(Self { c: count })
    }
}

impl AsRef<i64> for UnreadNewsCount {
    fn as_ref(&self) -> &i64 {
        &self.c
    }
}

diesel_i64_wrapper!(UnreadNewsCount);

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UnreadNewsCountResult {
    pub v: NewsSyncVersion,
    pub c: UnreadNewsCount,
    /// If true, client should not show the notification
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub h: bool,
}
