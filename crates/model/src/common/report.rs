use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::BigInt};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_struct_try_from, diesel_i64_try_from, diesel_i64_wrapper};
use utoipa::{IntoParams, ToSchema};

use super::AccountId;
use crate::{CustomReportId, schema_sqlite_types::Integer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromSqlRow, AsExpression)]
#[diesel(sql_type = BigInt)]
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

#[derive(Debug, Serialize, ToSchema)]
pub struct ReportId {
    id: i64,
}

impl ReportId {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}

impl From<ReportIdDb> for ReportId {
    fn from(value: ReportIdDb) -> Self {
        Self { id: value.0 }
    }
}

/// Values from 64 to 127
#[derive(Debug, Clone, Copy)]
pub struct CustomReportTypeNumberValue(i8);

impl CustomReportTypeNumberValue {
    pub fn new(value: u8) -> Result<Self, String> {
        let min = ReportTypeNumber::FIRST_CUSTOM_REPORT_TYPE_NUMBER as u8;
        let max = ReportTypeNumber::LAST_CUSTOM_REPORT_TYPE_NUMBER as u8;
        if value < min || value > max {
            Err(format!(
                "Invalid custom report type number value {}, min: {}, max: {}",
                value, min, max
            ))
        } else {
            Ok(Self(value as i8))
        }
    }

    pub fn to_report_type_number(&self) -> ReportTypeNumber {
        ReportTypeNumber { n: self.0 }
    }

    pub fn to_report_type_number_internal(&self) -> ReportTypeNumberInternal {
        ReportTypeNumberInternal::CustomReport(*self)
    }

    pub fn to_custom_report_id(&self) -> Result<CustomReportId, String> {
        CustomReportId::new(self.0 as u8)
    }
}

#[derive(Debug, Clone, Copy, diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = Integer)]
pub enum ReportTypeNumberInternal {
    ProfileName,
    ProfileText,
    ProfileContent,
    ChatMessage,
    /// Values from 64 to 127
    CustomReport(CustomReportTypeNumberValue),
}

impl ReportTypeNumberInternal {
    pub fn db_value(&self) -> i64 {
        self.to_i8().into()
    }

    fn to_i8(self) -> i8 {
        match self {
            Self::ProfileName => 0,
            Self::ProfileText => 1,
            Self::ProfileContent => 2,
            Self::ChatMessage => 3,
            Self::CustomReport(value) => value.0,
        }
    }
}

diesel_i64_struct_try_from!(ReportTypeNumberInternal);

impl TryFrom<i64> for ReportTypeNumberInternal {
    type Error = String;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = TryInto::<i8>::try_into(value).map_err(|e| e.to_string())?;
        let v = match value {
            0 => Self::ProfileName,
            1 => Self::ProfileText,
            2 => Self::ProfileContent,
            3 => Self::ChatMessage,
            64..=127 => Self::CustomReport(CustomReportTypeNumberValue(value)),
            v => return Err(format!("Unknown report type number value {}", v)),
        };
        Ok(v)
    }
}

impl From<ReportTypeNumberInternal> for i64 {
    fn from(value: ReportTypeNumberInternal) -> Self {
        value.db_value()
    }
}

impl From<ReportTypeNumberInternal> for ReportTypeNumber {
    fn from(value: ReportTypeNumberInternal) -> Self {
        Self { n: value.to_i8() }
    }
}

impl TryFrom<ReportTypeNumber> for ReportTypeNumberInternal {
    type Error = String;
    fn try_from(value: ReportTypeNumber) -> Result<Self, Self::Error> {
        Into::<i64>::into(value.n).try_into()
    }
}

/// Values:
///
/// * ProfileName = 0
/// * ProfileText = 1
/// * ProfileContent = 2
/// * ChatMessage = 3
/// * CustomReport = values from 64 to 127
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema)]
pub struct ReportTypeNumber {
    /// This is i8 so that max value is 127. That makes SQLite to
    /// store the value using single byte.
    pub n: i8,
}

impl ReportTypeNumber {
    pub const FIRST_CUSTOM_REPORT_TYPE_NUMBER: i8 = 64;
    pub const LAST_CUSTOM_REPORT_TYPE_NUMBER: i8 = 127;
    /// Max count for reports related to some account with specific type.
    pub const MAX_COUNT: usize = 100;
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

#[derive(Debug, Clone, Deserialize, Serialize, IntoParams, PartialEq)]
pub struct ReportQueryParams {
    /// Report target
    pub target: AccountId,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, ToSchema)]
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

    pub fn is_success(&self) -> bool {
        *self == Self::success()
    }
}
