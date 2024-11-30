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

impl UnreadNewsCount {
    pub fn new(count: i64) -> Self {
        Self { c: count }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.c
    }
}

diesel_i64_wrapper!(UnreadNewsCount);

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UnreadNewsCountResult {
    pub v: NewsSyncVersion,
    pub c: UnreadNewsCount,
}
