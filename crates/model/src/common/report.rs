
use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::schema_sqlite_types::Integer;

use super::AccountId;

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, FromSqlRow, AsExpression, ToSchema
)]
#[diesel(sql_type = BigInt)]
#[serde(transparent)]
pub struct ReportIdDb(pub i64);

impl ReportIdDb {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    pub fn as_i64(&self) -> &i64 {
        &self.0
    }
}

diesel_i64_wrapper!(ReportIdDb);

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
pub enum ReportTypeNumber {
    ProfileName = 0,
    ProfileText = 1,
    ProfileContent = 2,
}

impl ReportTypeNumber {
    pub const MAX_COUNT: usize = 100;
}

diesel_i64_try_from!(ReportTypeNumber);

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

#[derive(Debug, Clone, Deserialize, Serialize, IntoParams, PartialEq)]
pub struct ReportQueryParams {
    /// Report target
    pub target: AccountId,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateReportResult {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_outdated_report_content: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_too_many_reports: bool,
}

impl UpdateReportResult {
    pub fn success() -> Self {
        Self {
            error_outdated_report_content: false,
            error_too_many_reports: false,
        }
    }

    pub fn outdated_report_content() -> Self {
        Self {
            error_outdated_report_content: true,
            error_too_many_reports: false,
        }
    }

    pub fn too_many_reports() -> Self {
        Self {
            error_outdated_report_content: false,
            error_too_many_reports: true,
        }
    }
}
