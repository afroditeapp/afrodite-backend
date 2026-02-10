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

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema)]
pub enum MediaContentUploadType {
    /// JPEG and PNG images are supported
    Image,
}

impl MediaContentUploadType {
    /// Convert upload type to the stored content type.
    /// Images are always processed and stored as JPEG.
    pub fn to_stored_type(&self) -> MediaContentType {
        match self {
            Self::Image => MediaContentType::JpegImage,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct NewContentParams {
    /// Client captured this content.
    pub secure_capture: bool,
    pub content_type: MediaContentUploadType,
}
