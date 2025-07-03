use model::schema_sqlite_types::Integer;
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_i64_try_from;
use utoipa::{IntoParams, ToSchema};

mod profile_content_edited_time;
pub use profile_content_edited_time::*;

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
            _ => return Err(format!("Unknown media content type {value}")),
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
