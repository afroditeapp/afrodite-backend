
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_try_from;
use utoipa::{IntoParams, ToSchema};

use crate::schema_sqlite_types::Integer;

use super::AccountId;

#[derive(Debug, Clone, Deserialize, Serialize, IntoParams, PartialEq)]
pub struct ReportQueryParams {
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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateReportResult {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_outdated_report_content: bool,
}

impl UpdateReportResult {
    pub fn success() -> Self {
        Self {
            error_outdated_report_content: false,
        }
    }

    pub fn outdated_report_content() -> Self {
        Self {
            error_outdated_report_content: true
        }
    }
}
