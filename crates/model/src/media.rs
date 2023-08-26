use diesel::{
    prelude::*,
    sql_types::{BigInt, Binary},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    macros::{diesel_i64_wrapper, diesel_uuid_wrapper},
    AccountId, AccountIdDb,
};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct SlotNumber {
    pub slot_number: u8,
}

#[derive(Debug, Clone, Copy)]
#[repr(i64)]
pub enum ImageSlot {
    Image1 = 0,
    Image2 = 1,
    Image3 = 2,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ModerationRequestContent {
    /// Use slot 1 image as camera image.
    camera_image: bool,
    /// Include slot 1 image in moderation request.
    image1: ContentId,
    /// Include slot 2 image in moderation request.
    image2: Option<ContentId>,
    /// Include slot 3 image in moderation request.
    image3: Option<ContentId>,
}

impl ModerationRequestContent {
    pub fn content(&self) -> impl Iterator<Item = ContentId> {
        [Some(self.image1), self.image2, self.image3]
            .into_iter()
            .flatten()
    }

    pub fn slot_1_is_security_image(&self) -> bool {
        self.camera_image
    }

    pub fn slot_1(&self) -> ContentId {
        self.image1
    }

    pub fn slot_2(&self) -> Option<ContentId> {
        self.image2
    }
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::media_moderation_queue_number)]
#[diesel(check_for_backend(crate::Db))]
pub struct QueueNumberRaw {
    pub queue_number: ModerationQueueNumber,
    pub account_id: AccountIdDb,
    pub sub_queue: i64,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::media_moderation_request)]
#[diesel(check_for_backend(crate::Db))]
pub struct ModerationRequestRaw {
    pub id: ModerationRequestIdDb,
    pub account_id: AccountIdDb,
    pub queue_number: i64,
    pub json_text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ModerationRequestInternal {
    pub moderation_request_id: ModerationRequestIdDb,
    pub account_id: AccountId,
    pub state: ModerationRequestState,
    pub content: ModerationRequestContent,
}

impl ModerationRequestInternal {
    pub fn new(
        moderation_request_id: ModerationRequestIdDb,
        account_id: AccountId,
        state: ModerationRequestState,
        content: ModerationRequestContent,
    ) -> Self {
        Self {
            moderation_request_id,
            account_id,
            state,
            content,
        }
    }

    pub fn into_request(self) -> ModerationRequest {
        ModerationRequest {
            content: self.content,
            state: self.state,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ModerationRequest {
    pub state: ModerationRequestState,
    pub content: ModerationRequestContent,
}

#[derive(thiserror::Error, Debug)]
pub enum EnumParsingError {
    #[error("ParsingFailed, value: {0}")]
    ParsingError(i64),
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, ToSchema, PartialEq)]
#[repr(i64)]
pub enum ModerationRequestState {
    Waiting = 0,
    InProgress = 1,
    Accepted = 2,
    Denied = 3,
}

impl ModerationRequestState {
    pub fn completed(&self) -> bool {
        match self {
            Self::Accepted | Self::Denied => true,
            _ => false,
        }
    }
}

impl TryFrom<i64> for ModerationRequestState {
    type Error = EnumParsingError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = match value {
            0 => Self::Waiting,
            1 => Self::InProgress,
            2 => Self::Accepted,
            3 => Self::Denied,
            _ => return Err(EnumParsingError::ParsingError(value)),
        };

        Ok(value)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[repr(i64)]
pub enum ContentState {
    /// If user uploads new content to slot the current will be removed.
    InSlot = 0,
    /// Content is in moderation. User can not remove the content.
    InModeration = 1,
    /// Content is moderated as accepted. User can not remove the content.
    ModeratedAsAccepted = 2,
    /// Content is moderated as denied. Making new moderation request removes
    /// the content.
    ModeratedAsDenied = 3,
}

/// Admin sets this when moderating the image.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[repr(i64)]
pub enum MediaContentType {
    NotSet = 0,
    /// Normal image.
    Normal = 1,
    /// Security image.
    Security = 2,
}

// TODO: Remove content with state ModeratedAsDenied when new moderation request
// is created. Get content id from Moderation table.

impl TryFrom<i64> for ContentState {
    type Error = EnumParsingError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = match value {
            0 => Self::InSlot,
            1 => Self::InModeration,
            2 => Self::ModeratedAsAccepted,
            3 => Self::ModeratedAsDenied,
            _ => return Err(EnumParsingError::ParsingError(value)),
        };

        Ok(value)
    }
}

impl TryFrom<i64> for MediaContentType {
    type Error = EnumParsingError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = match value {
            0 => Self::NotSet,
            1 => Self::Normal,
            2 => Self::Security,
            _ => return Err(EnumParsingError::ParsingError(value)),
        };

        Ok(value)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct SlotId {
    pub slot_id: u8,
}

/// Content ID for media content for example images
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    IntoParams,
    PartialEq,
    Eq,
    Hash,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = Binary)]
pub struct ContentId {
    pub content_id: uuid::Uuid,
}

diesel_uuid_wrapper!(ContentId);

impl ContentId {
    pub fn new_random_id() -> Self {
        Self {
            content_id: Uuid::new_v4(),
        }
    }

    pub fn new(content_id: uuid::Uuid) -> Self {
        Self { content_id }
    }

    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.content_id
    }

    pub fn raw_jpg_image(&self) -> String {
        format!("{}.raw.jpg", self.content_id.as_hyphenated())
    }

    /// Image file name with extension.
    pub fn jpg_image(&self) -> String {
        format!("{}.jpg", self.content_id.as_hyphenated())
    }
}

impl sqlx::Type<sqlx::Sqlite> for ContentId {
    fn type_info() -> <sqlx::Sqlite as sqlx::Database>::TypeInfo {
        <Uuid as sqlx::Type<sqlx::Sqlite>>::type_info()
    }

    fn compatible(ty: &<sqlx::Sqlite as sqlx::Database>::TypeInfo) -> bool {
        <Uuid as sqlx::Type<sqlx::Sqlite>>::compatible(ty)
    }
}

impl<'a> sqlx::Encode<'a, sqlx::Sqlite> for ContentId {
    fn encode_by_ref<'q>(
        &self,
        buf: &mut <sqlx::Sqlite as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        self.content_id.encode_by_ref(buf)
    }

    fn encode<'q>(
        self,
        buf: &mut <sqlx::Sqlite as sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull
    where
        Self: Sized,
    {
        self.content_id.encode_by_ref(buf)
    }

    fn produces(&self) -> Option<<sqlx::Sqlite as sqlx::Database>::TypeInfo> {
        <Uuid as sqlx::Encode<'a, sqlx::Sqlite>>::produces(&self.content_id)
    }

    fn size_hint(&self) -> usize {
        self.content_id.size_hint()
    }
}

impl sqlx::Decode<'_, sqlx::Sqlite> for ContentId {
    fn decode(
        value: <sqlx::Sqlite as sqlx::database::HasValueRef<'_>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        <Uuid as sqlx::Decode<'_, sqlx::Sqlite>>::decode(value).map(|id| ContentId::new(id))
    }
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::media_content)]
#[diesel(check_for_backend(crate::Db))]
pub struct MediaContentRaw {
    pub id: ContentIdDb,
    pub uuid: ContentId,
    pub account_id: AccountIdDb,
    pub moderation_state: i64,
    pub content_type: i64,
    pub slot_number: i64,
}

impl MediaContentRaw {
    pub fn to_content_id_internal(&self) -> ContentIdInternal {
        ContentIdInternal {
            content_id: self.uuid,
            content_row_id: self.id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MediaContentInternal {
    pub content_id: ContentIdInternal,
    pub state: ContentState,
    pub content_type: MediaContentType,
    pub slot_number: i64,
}

#[derive(Debug, Clone)]
pub struct ContentIdInternal {
    pub content_id: ContentId,
    pub content_row_id: ContentIdDb,
}

impl ContentIdInternal {
    pub fn as_content_id(&self) -> ContentId {
        self.content_id
    }
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::current_account_media)]
#[diesel(check_for_backend(crate::Db))]
pub struct CurrentAccountMediaRaw {
    pub account_id: AccountIdDb,
    pub security_content_id: Option<ContentIdDb>,
    pub profile_content_id: Option<ContentIdDb>,
    pub grid_crop_size: f64,
    pub grid_crop_x: f64,
    pub grid_crop_y: f64,
}

#[derive(Debug, Clone)]
pub struct CurrentAccountMediaInternal {
    pub security_content_id: Option<ContentIdInternal>,
    pub profile_content_id: Option<ContentIdInternal>,
    pub grid_crop_size: f64,
    pub grid_crop_x: f64,
    pub grid_crop_y: f64,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct PrimaryImage {
    pub content_id: Option<ContentId>,
    pub grid_crop_size: f64,
    pub grid_crop_x: f64,
    pub grid_crop_y: f64,
}

impl From<CurrentAccountMediaInternal> for PrimaryImage {
    fn from(value: CurrentAccountMediaInternal) -> Self {
        Self {
            content_id: value.profile_content_id.map(|c| c.as_content_id()),
            grid_crop_size: value.grid_crop_size,
            grid_crop_x: value.grid_crop_x,
            grid_crop_y: value.grid_crop_y,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SecurityImage {
    pub content_id: Option<ContentId>,
}

impl From<CurrentAccountMediaInternal> for SecurityImage {
    fn from(value: CurrentAccountMediaInternal) -> Self {
        Self {
            content_id: value.security_content_id.map(|c| c.as_content_id()),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ImageAccessCheck {
    /// If false image access is allowed when profile is set as public.
    /// If true image access is allowed when users are a match.
    pub is_match: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct NormalImages {
    pub data: Vec<ContentId>,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    sqlx::Type,
    PartialEq,
    Eq,
    Hash,
    ToSchema,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct ModerationRequestIdDb(pub i64);

impl ModerationRequestIdDb {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    pub fn as_i64(&self) -> &i64 {
        &self.0
    }
}

diesel_i64_wrapper!(ModerationRequestIdDb);

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    sqlx::Type,
    PartialEq,
    Eq,
    Hash,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct ModerationQueueNumber(pub i64);

impl ModerationQueueNumber {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    pub fn as_i64(&self) -> &i64 {
        &self.0
    }
}

diesel_i64_wrapper!(ModerationQueueNumber);

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    sqlx::Type,
    PartialEq,
    Eq,
    Hash,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct ContentIdDb(pub i64);

impl ContentIdDb {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    pub fn as_i64(&self) -> &i64 {
        &self.0
    }
}

diesel_i64_wrapper!(ContentIdDb);
