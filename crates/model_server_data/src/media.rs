use model::schema_sqlite_types::Integer;
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_try_from;
use utoipa::{IntoParams, ToSchema};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    ToSchema,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Integer)]
#[repr(i64)]
pub enum ContentSlot {
    Content0 = 0,
    Content1 = 1,
    Content2 = 2,
    Content3 = 3,
    Content4 = 4,
    Content5 = 5,
    Content6 = 6,
}

impl TryFrom<i64> for ContentSlot {
    type Error = String;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let slot = match value {
            0 => Self::Content0,
            1 => Self::Content1,
            2 => Self::Content2,
            3 => Self::Content3,
            4 => Self::Content4,
            5 => Self::Content5,
            6 => Self::Content6,
            value => return Err(format!("Unknown content slot value {}", value)),
        };

        Ok(slot)
    }
}

diesel_i64_try_from!(ContentSlot);

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    Hash,
    diesel::FromSqlRow,
    diesel::AsExpression,
    ToSchema,
)]
#[diesel(sql_type = Integer)]
#[repr(i64)]
pub enum MediaContentType {
    JpegImage = 0,
}

impl MediaContentType {
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::JpegImage => "jpg",
        }
    }
}

diesel_i64_try_from!(MediaContentType);

impl TryFrom<i64> for MediaContentType {
    type Error = String;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = match value {
            0 => Self::JpegImage,
            _ => return Err(format!("Unknown media content type {}", value)),
        };

        Ok(value)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct NewContentParams {
    /// Client captured this content.
    pub secure_capture: bool,
    pub content_type: MediaContentType,
}
