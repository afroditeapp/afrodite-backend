use diesel::{
    prelude::*,
    sql_types::{BigInt, Binary},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper, diesel_uuid_wrapper};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{schema_sqlite_types::Integer, AccountId, AccountIdDb};

/// Y coordinate of slippy map tile.
///
/// This might include also .png file extension.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct MapTileY {
    pub y: String,
}

/// X coordinate of slippy map tile.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct MapTileX {
    pub x: u32,
}

/// Z coordinate (or zoom number) of slippy map tile.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct MapTileZ {
    pub z: u32,
}

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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub enum ContentProcessingStateType {
    /// This content slot is empty.
    Empty = 0,
    /// Content is waiting in processing queue.
    InQueue = 1,
    /// Content processing is ongoing.
    Processing = 2,
    /// Content is processed and content ID is now available.
    Completed = 3,
    /// Content processing failed.
    Failed = 4,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ContentProcessingState {
    pub state: ContentProcessingStateType,
    /// Current position in processing queue.
    ///
    /// If ProcessingContentId is added to empty queue, then
    /// this will be 1.
    pub wait_queue_position: Option<u64>,
    /// Content ID of the processed content.
    pub content_id: Option<ContentId>,
}

impl ContentProcessingState {
    pub fn empty() -> Self {
        Self {
            state: ContentProcessingStateType::Empty,
            wait_queue_position: None,
            content_id: None,
        }
    }

    pub fn in_queue_state(wait_queue_position: u64) -> Self {
        Self {
            state: ContentProcessingStateType::InQueue,
            wait_queue_position: Some(wait_queue_position),
            content_id: None,
        }
    }

    pub fn change_to_processing(&mut self) {
        self.state = ContentProcessingStateType::Processing;
        self.wait_queue_position = None;
        self.content_id = None;
    }

    pub fn change_to_completed(&mut self, content_id: ContentId) {
        self.state = ContentProcessingStateType::Completed;
        self.wait_queue_position = None;
        self.content_id = Some(content_id);
    }

    pub fn change_to_failed(&mut self) {
        self.state = ContentProcessingStateType::Failed;
        self.wait_queue_position = None;
        self.content_id = None;
    }
}

/// Content ID which is queued to be processed
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct ContentProcessingId {
    id: uuid::Uuid,
}

impl ContentProcessingId {
    pub fn new_random_id() -> Self {
        Self { id: Uuid::new_v4() }
    }

    pub fn to_content_id(&self) -> ContentId {
        ContentId::new(self.id)
    }
}

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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ModerationRequestContent {
    pub content0: ContentId,
    pub content1: Option<ContentId>,
    pub content2: Option<ContentId>,
    pub content3: Option<ContentId>,
    pub content4: Option<ContentId>,
    pub content5: Option<ContentId>,
    pub content6: Option<ContentId>,
}

impl ModerationRequestContent {
    pub fn content(&self) -> impl Iterator<Item = ContentId> {
        [
            Some(self.content0),
            self.content1,
            self.content2,
            self.content3,
            self.content4,
            self.content5,
            self.content6,
        ]
        .into_iter()
        .flatten()
    }
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::media_moderation_request)]
#[diesel(check_for_backend(crate::Db))]
pub struct MediaModerationRequestRaw {
    pub id: ModerationRequestIdDb,
    pub account_id: AccountIdDb,
    pub queue_number: i64,
    pub content_id_0: ContentId,
    pub content_id_1: Option<ContentId>,
    pub content_id_2: Option<ContentId>,
    pub content_id_3: Option<ContentId>,
    pub content_id_4: Option<ContentId>,
    pub content_id_5: Option<ContentId>,
    pub content_id_6: Option<ContentId>,
}

impl MediaModerationRequestRaw {
    pub fn to_moderation_request_content(&self) -> ModerationRequestContent {
        ModerationRequestContent {
            content0: self.content_id_0,
            content1: self.content_id_1,
            content2: self.content_id_2,
            content3: self.content_id_3,
            content4: self.content_id_4,
            content5: self.content_id_5,
            content6: self.content_id_6,
        }
    }
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

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Clone,
    Copy,
    ToSchema,
    PartialEq,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Integer)]
#[repr(i64)]
pub enum ModerationRequestState {
    /// Admin has not started progress on moderating.
    Waiting = 0,
    InProgress = 1,
    Accepted = 2,
    Denied = 3,
}

diesel_i64_try_from!(ModerationRequestState);

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

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    ToSchema,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Integer)]
#[repr(i64)]
pub enum ContentState {
    /// If user uploads new content to slot the current will be removed.
    InSlot = 0,
    /// Content is in moderation. User can not remove the content.
    InModeration = 1,
    /// Content is moderated as accepted. User can not remove the content until
    /// specific time elapses.
    ModeratedAsAccepted = 2,
    /// Content is moderated as denied.
    ModeratedAsDenied = 3,
}

diesel_i64_try_from!(ContentState);

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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct SlotId {
    pub slot_id: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct NewContentParams {
    /// Client captured this content.
    pub secure_capture: bool,
    pub content_type: MediaContentType,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ContentInfo {
    pub id: ContentId,
    pub content_type: MediaContentType,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ContentInfoDetailed {
    pub id: ContentId,
    pub content_type: MediaContentType,
    pub state: ContentState,
    pub slot: Option<ContentSlot>,
    pub secure_capture: bool,
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

    /// File name for unprocessed user uploaded content.
    pub fn raw_content_file_name(&self) -> String {
        format!("{}.raw", self.content_id.as_hyphenated())
    }

    pub fn content_file_name(&self) -> String {
        format!("{}", self.content_id.as_hyphenated())
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
    pub content_state: ContentState,
    pub secure_capture: bool,
    pub content_type_number: MediaContentType,
    pub slot_number: ContentSlot,
}

impl From<MediaContentRaw> for MediaContentInternal {
    fn from(value: MediaContentRaw) -> MediaContentInternal {
        MediaContentInternal {
            content_id: value.uuid,
            content_row_id: value.id,
            content_type: value.content_type_number,
            state: value.content_state,
            secure_capture: value.secure_capture,
            slot_number: if value.content_state == ContentState::InSlot {
                Some(value.slot_number)
            } else {
                None
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct MediaContentInternal {
    pub content_id: ContentId,
    pub content_row_id: ContentIdDb,
    pub content_type: MediaContentType,
    pub state: ContentState,
    pub secure_capture: bool,
    pub slot_number: Option<ContentSlot>,
}

impl From<MediaContentInternal> for ContentId {
    fn from(value: MediaContentInternal) -> Self {
        value.content_id
    }
}

impl From<MediaContentInternal> for ContentInfo {
    fn from(value: MediaContentInternal) -> Self {
        ContentInfo {
            id: value.content_id,
            content_type: value.content_type,
        }
    }
}

impl From<MediaContentInternal> for ContentInfoDetailed {
    fn from(value: MediaContentInternal) -> Self {
        ContentInfoDetailed {
            id: value.content_id,
            content_type: value.content_type,
            state: value.state,
            slot: value.slot_number,
            secure_capture: value.secure_capture,
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::current_account_media)]
#[diesel(check_for_backend(crate::Db))]
pub struct CurrentAccountMediaRaw {
    pub account_id: AccountIdDb,
    pub security_content_id: Option<ContentIdDb>,
    pub profile_content_id_0: Option<ContentIdDb>,
    pub profile_content_id_1: Option<ContentIdDb>,
    pub profile_content_id_2: Option<ContentIdDb>,
    pub profile_content_id_3: Option<ContentIdDb>,
    pub profile_content_id_4: Option<ContentIdDb>,
    pub profile_content_id_5: Option<ContentIdDb>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
    pub pending_security_content_id: Option<ContentIdDb>,
    pub pending_profile_content_id_0: Option<ContentIdDb>,
    pub pending_profile_content_id_1: Option<ContentIdDb>,
    pub pending_profile_content_id_2: Option<ContentIdDb>,
    pub pending_profile_content_id_3: Option<ContentIdDb>,
    pub pending_profile_content_id_4: Option<ContentIdDb>,
    pub pending_profile_content_id_5: Option<ContentIdDb>,
    pub pending_grid_crop_size: Option<f64>,
    pub pending_grid_crop_x: Option<f64>,
    pub pending_grid_crop_y: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct CurrentAccountMediaInternal {
    pub security_content_id: Option<MediaContentInternal>,
    pub profile_content_id_0: Option<MediaContentInternal>,
    pub profile_content_id_1: Option<MediaContentInternal>,
    pub profile_content_id_2: Option<MediaContentInternal>,
    pub profile_content_id_3: Option<MediaContentInternal>,
    pub profile_content_id_4: Option<MediaContentInternal>,
    pub profile_content_id_5: Option<MediaContentInternal>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
    pub pending_security_content_id: Option<MediaContentInternal>,
    pub pending_profile_content_id_0: Option<MediaContentInternal>,
    pub pending_profile_content_id_1: Option<MediaContentInternal>,
    pub pending_profile_content_id_2: Option<MediaContentInternal>,
    pub pending_profile_content_id_3: Option<MediaContentInternal>,
    pub pending_profile_content_id_4: Option<MediaContentInternal>,
    pub pending_profile_content_id_5: Option<MediaContentInternal>,
    pub pending_grid_crop_size: Option<f64>,
    pub pending_grid_crop_x: Option<f64>,
    pub pending_grid_crop_y: Option<f64>,
}

impl CurrentAccountMediaInternal {
    pub const GRID_CROP_SIZE_DEFAULT: f64 = 1.0;
    pub const GRID_CROP_X_DEFAULT: f64 = 0.0;
    pub const GRID_CROP_Y_DEFAULT: f64 = 0.0;
}

/// Update normal or pending profile content
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SetProfileContent {
    /// Primary profile image which is shown in grid view.
    pub content_id_0: ContentId,
    pub content_id_1: Option<ContentId>,
    pub content_id_2: Option<ContentId>,
    pub content_id_3: Option<ContentId>,
    pub content_id_4: Option<ContentId>,
    pub content_id_5: Option<ContentId>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
}

impl SetProfileContent {
    pub fn iter(&self) -> impl Iterator<Item = ContentId> {
        [
            Some(self.content_id_0),
            self.content_id_1,
            self.content_id_2,
            self.content_id_3,
            self.content_id_4,
            self.content_id_5,
        ]
        .into_iter()
        .filter_map(|c| c.as_ref().cloned())
    }
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SetProfileContentInternal {
    /// Primary profile image which is shown in grid view.
    pub content_id_0: Option<ContentId>,
    pub content_id_1: Option<ContentId>,
    pub content_id_2: Option<ContentId>,
    pub content_id_3: Option<ContentId>,
    pub content_id_4: Option<ContentId>,
    pub content_id_5: Option<ContentId>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
}

impl From<SetProfileContent> for SetProfileContentInternal {
    fn from(value: SetProfileContent) -> Self {
        Self {
            content_id_0: Some(value.content_id_0),
            content_id_1: value.content_id_1,
            content_id_2: value.content_id_2,
            content_id_3: value.content_id_3,
            content_id_4: value.content_id_4,
            content_id_5: value.content_id_5,
            grid_crop_size: value.grid_crop_size,
            grid_crop_x: value.grid_crop_x,
            grid_crop_y: value.grid_crop_y,
        }
    }
}

/// Current content in public profile.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ProfileContent {
    /// Primary profile image which is shown in grid view.
    pub content_id_0: Option<ContentInfo>,
    pub content_id_1: Option<ContentInfo>,
    pub content_id_2: Option<ContentInfo>,
    pub content_id_3: Option<ContentInfo>,
    pub content_id_4: Option<ContentInfo>,
    pub content_id_5: Option<ContentInfo>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
}

impl From<CurrentAccountMediaInternal> for ProfileContent {
    fn from(value: CurrentAccountMediaInternal) -> Self {
        Self {
            content_id_0: value.profile_content_id_0.map(|c| c.into()),
            content_id_1: value.profile_content_id_1.map(|c| c.into()),
            content_id_2: value.profile_content_id_2.map(|c| c.into()),
            content_id_3: value.profile_content_id_3.map(|c| c.into()),
            content_id_4: value.profile_content_id_4.map(|c| c.into()),
            content_id_5: value.profile_content_id_5.map(|c| c.into()),
            grid_crop_size: value.grid_crop_size,
            grid_crop_x: value.grid_crop_x,
            grid_crop_y: value.grid_crop_y,
        }
    }
}

/// Profile image settings which will be applied when moderation request is
/// accepted.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct PendingProfileContent {
    /// Primary profile image which is shown in grid view.
    ///
    /// If this is None, then server will not change the current profile content
    /// when moderation is accepted.
    pub content_id_0: Option<ContentInfo>,
    pub content_id_1: Option<ContentInfo>,
    pub content_id_2: Option<ContentInfo>,
    pub content_id_3: Option<ContentInfo>,
    pub content_id_4: Option<ContentInfo>,
    pub content_id_5: Option<ContentInfo>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
}

impl From<CurrentAccountMediaInternal> for PendingProfileContent {
    fn from(value: CurrentAccountMediaInternal) -> Self {
        Self {
            content_id_0: value.pending_profile_content_id_0.map(|c| c.into()),
            content_id_1: value.pending_profile_content_id_1.map(|c| c.into()),
            content_id_2: value.pending_profile_content_id_2.map(|c| c.into()),
            content_id_3: value.pending_profile_content_id_3.map(|c| c.into()),
            content_id_4: value.pending_profile_content_id_4.map(|c| c.into()),
            content_id_5: value.pending_profile_content_id_5.map(|c| c.into()),
            grid_crop_size: value.pending_grid_crop_size,
            grid_crop_x: value.pending_grid_crop_x,
            grid_crop_y: value.pending_grid_crop_y,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SecurityImage {
    pub content_id: Option<ContentInfo>,
}

impl From<CurrentAccountMediaInternal> for SecurityImage {
    fn from(value: CurrentAccountMediaInternal) -> Self {
        Self {
            content_id: value.security_content_id.map(|c| c.into()),
        }
    }
}

/// Security image settings which will be applied when moderation request is
/// accepted.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct PendingSecurityImage {
    pub content_id: Option<ContentInfo>,
}

impl From<CurrentAccountMediaInternal> for PendingSecurityImage {
    fn from(value: CurrentAccountMediaInternal) -> Self {
        Self {
            content_id: value.pending_security_content_id.map(|c| c.into()),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ContentAccessCheck {
    /// If false media content access is allowed when profile is set as public.
    /// If true media content access is allowed when users are a match.
    pub is_match: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct AccountContent {
    pub data: Vec<ContentInfoDetailed>,
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
