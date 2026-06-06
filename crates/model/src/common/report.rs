use diesel::{
    deserialize::FromSqlRow,
    expression::AsExpression,
    sql_types::{BigInt, SmallInt},
};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::{SimpleDieselEnum, diesel_i64_wrapper};
use utoipa::{IntoParams, ToSchema};

use super::AccountId;
use crate::CustomReportId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromSqlRow, AsExpression)]
#[diesel(sql_type = BigInt)]
pub struct ReportIdDb(pub i64);

impl TryFrom<i64> for ReportIdDb {
    type Error = String;

    fn try_from(id: i64) -> Result<Self, Self::Error> {
        Ok(Self(id))
    }
}

impl AsRef<i64> for ReportIdDb {
    fn as_ref(&self) -> &i64 {
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
pub struct CustomReportTypeValue(i8);

impl CustomReportTypeValue {
    pub fn new(value: u8) -> Result<Self, String> {
        let min = ReportType::FIRST_CUSTOM_REPORT_TYPE_NUMBER as u8;
        let max = ReportType::LAST_CUSTOM_REPORT_TYPE_NUMBER as u8;
        if value < min || value > max {
            Err(format!(
                "Invalid custom report type number value {value}, min: {min}, max: {max}"
            ))
        } else {
            Ok(Self(value as i8))
        }
    }

    pub fn to_report_type(&self) -> ReportType {
        ReportType { n: self.0 }
    }

    pub fn to_report_type_internal(&self) -> ReportTypeInternal {
        ReportTypeInternal::CustomReport(*self)
    }

    pub fn to_custom_report_id(&self) -> Result<CustomReportId, String> {
        CustomReportId::new(self.0 as u8)
    }
}

#[derive(Debug, Clone, Copy, diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = SmallInt)]
pub enum ReportTypeInternal {
    ProfileName,
    ProfileText,
    ProfileContent,
    ChatMessage,
    /// Values from 64 to 127
    CustomReport(CustomReportTypeValue),
}

impl ReportTypeInternal {
    pub fn db_value(&self) -> i16 {
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

impl TryFrom<i16> for ReportTypeInternal {
    type Error = String;
    fn try_from(value: i16) -> Result<Self, Self::Error> {
        let value = TryInto::<i8>::try_into(value).map_err(|e| e.to_string())?;
        let v = match value {
            0 => Self::ProfileName,
            1 => Self::ProfileText,
            2 => Self::ProfileContent,
            3 => Self::ChatMessage,
            64..=127 => Self::CustomReport(CustomReportTypeValue(value)),
            v => return Err(format!("Unknown report type number value {v}")),
        };
        Ok(v)
    }
}

impl<DB: diesel::backend::Backend> diesel::deserialize::FromSql<diesel::sql_types::SmallInt, DB>
    for ReportTypeInternal
where
    i16: diesel::deserialize::FromSql<diesel::sql_types::SmallInt, DB>,
{
    fn from_sql(
        value: <DB as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let value = i16::from_sql(value)?;
        TryInto::<Self>::try_into(value).map_err(|e| e.into())
    }
}

impl diesel::serialize::ToSql<diesel::sql_types::SmallInt, diesel::pg::Pg> for ReportTypeInternal
where
    i16: diesel::serialize::ToSql<diesel::sql_types::SmallInt, diesel::pg::Pg>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>,
    ) -> diesel::serialize::Result {
        let value = self.db_value();
        <i16 as diesel::serialize::ToSql<diesel::sql_types::SmallInt, diesel::pg::Pg>>::to_sql(
            &value,
            &mut out.reborrow(),
        )
    }
}

impl diesel::serialize::ToSql<diesel::sql_types::SmallInt, diesel::sqlite::Sqlite>
    for ReportTypeInternal
where
    i16: diesel::serialize::ToSql<diesel::sql_types::SmallInt, diesel::sqlite::Sqlite>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, diesel::sqlite::Sqlite>,
    ) -> diesel::serialize::Result {
        let value: i32 = self.db_value().into();
        out.set_value(value);
        Ok(diesel::serialize::IsNull::No)
    }
}

impl From<ReportTypeInternal> for ReportType {
    fn from(value: ReportTypeInternal) -> Self {
        Self { n: value.to_i8() }
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
pub struct ReportType {
    /// This is i8 so that max value is 127. That makes SQLite to
    /// store the value using single byte.
    pub n: i8,
}

impl ReportType {
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
    SimpleDieselEnum,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = SmallInt)]
#[repr(i16)]
#[derive(Default)]
pub enum ReportProcessingState {
    #[default]
    Waiting = 0,
    AcceptedByAdminBot = 1,
    AcceptedByAdmin = 2,
    RejectedByAdminBot = 3,
    RejectedByAdmin = 4,
}

impl ReportProcessingState {
    pub const fn processed_states() -> [Self; 4] {
        [
            Self::AcceptedByAdminBot,
            Self::AcceptedByAdmin,
            Self::RejectedByAdminBot,
            Self::RejectedByAdmin,
        ]
    }

    pub const fn rejected_states() -> [Self; 2] {
        [Self::RejectedByAdminBot, Self::RejectedByAdmin]
    }

    pub const fn accepted_states() -> [Self; 2] {
        [Self::AcceptedByAdminBot, Self::AcceptedByAdmin]
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, IntoParams, PartialEq)]
pub struct ReportQueryParams {
    /// Report target
    pub target: AccountId,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, ToSchema)]
pub struct UpdateReportResult {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_outdated_report_content: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_too_many_reports: bool,
}

impl UpdateReportResult {
    pub fn success() -> Self {
        Default::default()
    }

    pub fn outdated_report_content() -> Self {
        Self {
            error: true,
            error_outdated_report_content: true,
            ..Default::default()
        }
    }

    pub fn too_many_reports() -> Self {
        Self {
            error: true,
            error_too_many_reports: true,
            ..Default::default()
        }
    }

    pub fn is_success(&self) -> bool {
        *self == Self::success()
    }
}
