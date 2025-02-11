
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_try_from;
use utoipa::{IntoParams, ToSchema};

use crate::schema_sqlite_types::Integer;

use super::AccountId;

#[derive(Debug, Clone, Deserialize, Serialize, IntoParams, PartialEq)]
pub struct ReportQueryParams {
    /// Report creator
    pub creator: AccountId,
    /// Report target
    pub target: AccountId,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Eq,
    TryFromPrimitive,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Integer)]
#[repr(i64)]
pub enum ReportProcessingState {
    Empty = 0,
    Waiting = 1,
    Done = 2,
}

impl Default for ReportProcessingState {
    fn default() -> Self {
        Self::Empty
    }
}

diesel_i64_try_from!(ReportProcessingState);
