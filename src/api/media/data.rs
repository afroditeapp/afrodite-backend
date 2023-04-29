use serde::{Deserialize, Serialize};

use sqlx::Encode;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::api::model::{AccountIdInternal, AccountIdLight};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ImageFileName {
    image_file: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct SlotNumber {
    pub slot_number: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ImageFile {
    #[schema(value_type = String, format = Binary)]
    data: Vec<u8>,
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

    pub fn camera(&self) -> bool {
        self.camera_image
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ModerationRequestInternal {
    moderation_request_id: i64,
    account_id: AccountIdLight,
    state: ModerationRequestState,
    content: ModerationRequestContent,
}

impl ModerationRequestInternal {
    pub fn new(
        moderation_request_id: i64,
        account_id: AccountIdLight,
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
pub enum StateParsingError {
    #[error("ParsingFailed, value: {0}")]
    ParsingError(i64),
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, ToSchema)]
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
    type Error = StateParsingError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = match value {
            0 => Self::Waiting,
            1 => Self::InProgress,
            2 => Self::Accepted,
            3 => Self::Denied,
            _ => return Err(StateParsingError::ParsingError(value)),
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

// TODO: Remove content with state ModeratedAsDenied when new moderation request
// is created. Get content id from Moderation table.

impl TryFrom<i64> for ContentState {
    type Error = StateParsingError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = match value {
            0 => Self::InSlot,
            1 => Self::InModeration,
            2 => Self::ModeratedAsAccepted,
            3 => Self::ModeratedAsDenied,
            _ => return Err(StateParsingError::ParsingError(value)),
        };

        Ok(value)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ModerationList {
    pub list: Vec<Moderation>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct HandleModerationRequest {
    pub accept: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct SlotId {
    pub slot_id: u8,
}

/// Content ID for media content for example images
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, IntoParams, PartialEq, Eq, Hash)]
pub struct ContentId {
    pub content_id: uuid::Uuid,
}

impl ContentId {
    pub fn new_random_id() -> Self {
        Self {
            content_id: Uuid::new_v4(),
        }
    }

    pub fn new(content_id: Uuid) -> Self {
        Self { content_id }
    }

    pub fn as_uuid(&self) -> Uuid {
        self.content_id
    }

    pub fn raw_jpg_image(&self) -> String {
        format!("{}.raw.jpg", self.content_id.as_hyphenated())
    }

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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct Content {
    pub content_id: ContentId,
    pub state: ContentState,
    pub slot_number: i64,
}

#[derive(Debug, Clone)]
pub struct ContentIdInternal {
    pub content_id: uuid::Uuid,
    pub content_row_id: i64,
}

impl ContentIdInternal {
    pub fn as_content_id(&self) -> ContentId {
        ContentId::new(self.content_id)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ModerationId {
    pub request_id: ModerationRequestId,
    pub account_id: AccountIdInternal,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Moderation {
    pub request_creator_id: AccountIdLight,
    pub request_id: ModerationRequestId,
    pub moderator_id: AccountIdLight,
    pub content: ModerationRequestContent,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ModerationRequestId {
    pub request_row_id: i64,
}

#[derive(Debug, Copy, Clone)]
pub struct ModerationRequestQueueNumber {
    pub number: i64,
}
