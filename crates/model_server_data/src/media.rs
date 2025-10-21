use diesel::sql_types::SmallInt;
use serde::{Deserialize, Serialize};
use simple_backend_model::SimpleDieselEnum;
use utoipa::{IntoParams, ToSchema};

mod profile_content;
pub use profile_content::*;

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    Hash,
    SimpleDieselEnum,
    diesel::FromSqlRow,
    diesel::AsExpression,
    num_enum::TryFromPrimitive,
    ToSchema,
)]
#[diesel(sql_type = SmallInt)]
#[repr(i16)]
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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct NewContentParams {
    /// Client captured this content.
    pub secure_capture: bool,
    pub content_type: MediaContentType,
}
