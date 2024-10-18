use diesel::{
    prelude::*,
    sql_types::{BigInt, Binary},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper, diesel_uuid_wrapper};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{schema_sqlite_types::Integer, AccountId, AccountIdDb, EnumParsingError, NextQueueNumberType};

/// Y coordinate of slippy map tile.
///
/// This might include also .png file extension.
#[derive(Debug, Clone, Deserialize, Serialize, IntoParams)]
pub struct MapTileY {
    pub y: String,
}

/// X coordinate of slippy map tile.
#[derive(Debug, Clone, Deserialize, Serialize, IntoParams)]
pub struct MapTileX {
    pub x: u32,
}

/// Z coordinate (or zoom number) of slippy map tile.
#[derive(Debug, Clone, Deserialize, Serialize, IntoParams)]
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
    pub cid: Option<ContentId>,
}

impl ContentProcessingState {
    pub fn empty() -> Self {
        Self {
            state: ContentProcessingStateType::Empty,
            wait_queue_position: None,
            cid: None,
        }
    }

    pub fn in_queue_state(wait_queue_position: u64) -> Self {
        Self {
            state: ContentProcessingStateType::InQueue,
            wait_queue_position: Some(wait_queue_position),
            cid: None,
        }
    }

    pub fn change_to_processing(&mut self) {
        self.state = ContentProcessingStateType::Processing;
        self.wait_queue_position = None;
        self.cid = None;
    }

    pub fn change_to_completed(&mut self, content_id: ContentId) {
        self.state = ContentProcessingStateType::Completed;
        self.wait_queue_position = None;
        self.cid = Some(content_id);
    }

    pub fn change_to_failed(&mut self) {
        self.state = ContentProcessingStateType::Failed;
        self.wait_queue_position = None;
        self.cid = None;
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
    pub c0: ContentId,
    pub c1: Option<ContentId>,
    pub c2: Option<ContentId>,
    pub c3: Option<ContentId>,
    pub c4: Option<ContentId>,
    pub c5: Option<ContentId>,
    pub c6: Option<ContentId>,
}

impl ModerationRequestContent {
    pub fn iter(&self) -> impl Iterator<Item = ContentId> {
        [
            Some(self.c0),
            self.c1,
            self.c2,
            self.c3,
            self.c4,
            self.c5,
            self.c6,
        ]
        .into_iter()
        .flatten()
    }

    pub fn exists(&self, id: ContentId) -> bool {
        self.find(id).is_some()
    }

    pub fn not_exists(&self, id: ContentId) -> bool {
        !self.exists(id)
    }

    pub fn find(&self, id: ContentId) -> Option<ContentId> {
        self.iter().find(|c| *c == id)
    }
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::media_moderation_request)]
#[diesel(check_for_backend(crate::Db))]
pub struct MediaModerationRequestRaw {
    pub id: ModerationRequestIdDb,
    pub account_id: AccountIdDb,
    pub queue_number: ModerationQueueNumber,
    pub queue_number_type: NextQueueNumberType,
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
            c0: self.content_id_0,
            c1: self.content_id_1,
            c2: self.content_id_2,
            c3: self.content_id_3,
            c4: self.content_id_4,
            c5: self.content_id_5,
            c6: self.content_id_6,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModerationRequestInternal {
    pub moderation_request_id: ModerationRequestIdDb,
    pub account_id: AccountId,
    pub state: ModerationRequestState,
    pub content: ModerationRequestContent,
    pub queue_number: ModerationQueueNumber,
    pub queue_type: NextQueueNumberType,
}

impl ModerationRequestInternal {
    pub fn new(
        moderation_request_id: ModerationRequestIdDb,
        account_id: AccountId,
        state: ModerationRequestState,
        content: ModerationRequestContent,
        queue_number: ModerationQueueNumber,
        queue_type: NextQueueNumberType,
    ) -> Self {
        Self {
            moderation_request_id,
            account_id,
            state,
            content,
            queue_number,
            queue_type,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct CurrentModerationRequest {
    pub request: Option<ModerationRequest>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ModerationRequest {
    pub state: ModerationRequestState,
    pub content: ModerationRequestContent,
    // Waiting position in moderation queue if request state is Waiting.
    pub waiting_position: Option<i64>,
}

impl ModerationRequest {
    pub fn new(request: ModerationRequestInternal, waiting_position: Option<i64>) -> Self {
        Self {
            content: request.content,
            state: request.state,
            waiting_position,
        }
    }
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
    Rejected = 3,
}

diesel_i64_try_from!(ModerationRequestState);

impl ModerationRequestState {
    pub fn completed(&self) -> bool {
        match self {
            Self::Accepted | Self::Rejected => true,
            Self::InProgress | Self::Waiting => false,
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
            3 => Self::Rejected,
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
    /// Content is moderated as rejected.
    ModeratedAsRejected = 3,
}

diesel_i64_try_from!(ContentState);

// TODO: Remove content with state ModeratedAsRejected when new moderation request
// is created. Get content id from Moderation table.

impl TryFrom<i64> for ContentState {
    type Error = EnumParsingError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let value = match value {
            0 => Self::InSlot,
            1 => Self::InModeration,
            2 => Self::ModeratedAsAccepted,
            3 => Self::ModeratedAsRejected,
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
    pub cid: ContentId,
    pub ctype: MediaContentType,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ContentInfoDetailed {
    pub cid: ContentId,
    pub ctype: MediaContentType,
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
    pub cid: uuid::Uuid,
}

diesel_uuid_wrapper!(ContentId);

impl ContentId {
    pub fn new_random_id() -> Self {
        Self {
            cid: Uuid::new_v4(),
        }
    }

    pub fn new(content_id: uuid::Uuid) -> Self {
        Self { cid: content_id }
    }

    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.cid
    }

    /// File name for unprocessed user uploaded content.
    pub fn raw_content_file_name(&self) -> String {
        format!("{}.raw", self.cid.as_hyphenated())
    }

    pub fn content_file_name(&self) -> String {
        format!("{}", self.cid.as_hyphenated())
    }

    pub fn not_in(&self, mut iter: impl Iterator<Item = ContentId>) -> bool {
        !iter.any(|c| c == *self)
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
    slot_number: ContentSlot,
}

impl MediaContentRaw {
    pub fn slot_number(&self) -> Option<ContentSlot> {
        if self.content_state == ContentState::InSlot {
            Some(self.slot_number)
        } else {
            None
        }
    }

    pub fn content_id(&self) -> ContentId {
        self.uuid
    }

    pub fn state(&self) -> ContentState {
        self.content_state
    }

    pub fn content_type(&self) -> MediaContentType {
        self.content_type_number
    }

    pub fn content_row_id(&self) -> ContentIdDb {
        self.id
    }
}

impl From<MediaContentRaw> for ContentId {
    fn from(value: MediaContentRaw) -> Self {
        value.uuid
    }
}

impl From<MediaContentRaw> for ContentInfo {
    fn from(value: MediaContentRaw) -> Self {
        ContentInfo {
            cid: value.uuid,
            ctype: value.content_type_number,
        }
    }
}

impl From<MediaContentRaw> for ContentInfoDetailed {
    fn from(value: MediaContentRaw) -> Self {
        ContentInfoDetailed {
            cid: value.uuid,
            ctype: value.content_type_number,
            state: value.content_state,
            slot: value.slot_number(),
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
    pub profile_content_version_uuid: ProfileContentVersion,
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
    pub security_content_id: Option<MediaContentRaw>,
    pub profile_content_version_uuid: ProfileContentVersion,
    pub profile_content_id_0: Option<MediaContentRaw>,
    pub profile_content_id_1: Option<MediaContentRaw>,
    pub profile_content_id_2: Option<MediaContentRaw>,
    pub profile_content_id_3: Option<MediaContentRaw>,
    pub profile_content_id_4: Option<MediaContentRaw>,
    pub profile_content_id_5: Option<MediaContentRaw>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
    pub pending_security_content_id: Option<MediaContentRaw>,
    pub pending_profile_content_id_0: Option<MediaContentRaw>,
    pub pending_profile_content_id_1: Option<MediaContentRaw>,
    pub pending_profile_content_id_2: Option<MediaContentRaw>,
    pub pending_profile_content_id_3: Option<MediaContentRaw>,
    pub pending_profile_content_id_4: Option<MediaContentRaw>,
    pub pending_profile_content_id_5: Option<MediaContentRaw>,
    pub pending_grid_crop_size: Option<f64>,
    pub pending_grid_crop_x: Option<f64>,
    pub pending_grid_crop_y: Option<f64>,
}

impl CurrentAccountMediaInternal {
    pub fn iter_all_content(&self) -> impl Iterator<Item = &MediaContentRaw> {
        self.iter_current_profile_content()
            .chain(self.iter_pending_profile_content())
            .chain(self.security_content_id.iter())
            .chain(self.pending_security_content_id.iter())
    }

    pub fn iter_current_profile_content(&self) -> impl Iterator<Item = &MediaContentRaw> {
        [
            &self.profile_content_id_0,
            &self.profile_content_id_1,
            &self.profile_content_id_2,
            &self.profile_content_id_3,
            &self.profile_content_id_4,
            &self.profile_content_id_5,
        ]
        .into_iter()
        .flatten()
    }

    fn iter_pending_profile_content(&self) -> impl Iterator<Item = &MediaContentRaw> {
        [
            &self.pending_profile_content_id_0,
            &self.pending_profile_content_id_1,
            &self.pending_profile_content_id_2,
            &self.pending_profile_content_id_3,
            &self.pending_profile_content_id_4,
            &self.pending_profile_content_id_5,
        ]
        .into_iter()
        .flatten()
    }

    /// Returns true if pending security and profile content is empty.
    pub fn pending_content_is_empty(&self) -> bool {
        self.pending_security_content_id.is_none()
            && self.pending_profile_content_id_0.is_none()
            && self.pending_profile_content_id_1.is_none()
            && self.pending_profile_content_id_2.is_none()
            && self.pending_profile_content_id_3.is_none()
            && self.pending_profile_content_id_4.is_none()
            && self.pending_profile_content_id_5.is_none()
    }
}

/// Update normal or pending profile content
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SetProfileContent {
    /// Primary profile image which is shown in grid view.
    pub c0: ContentId,
    pub c1: Option<ContentId>,
    pub c2: Option<ContentId>,
    pub c3: Option<ContentId>,
    pub c4: Option<ContentId>,
    pub c5: Option<ContentId>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
}

impl SetProfileContent {
    pub fn iter(&self) -> impl Iterator<Item = ContentId> {
        [
            Some(self.c0),
            self.c1,
            self.c2,
            self.c3,
            self.c4,
            self.c5,
        ]
        .into_iter()
        .filter_map(|c| c.as_ref().cloned())
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub struct SetProfileContentInternal {
    /// Primary profile image which is shown in grid view.
    pub c0: Option<ContentId>,
    pub c1: Option<ContentId>,
    pub c2: Option<ContentId>,
    pub c3: Option<ContentId>,
    pub c4: Option<ContentId>,
    pub c5: Option<ContentId>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
}

impl From<SetProfileContent> for SetProfileContentInternal {
    fn from(value: SetProfileContent) -> Self {
        Self {
            c0: Some(value.c0),
            c1: value.c1,
            c2: value.c2,
            c3: value.c3,
            c4: value.c4,
            c5: value.c5,
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
    pub c0: Option<ContentInfo>,
    pub c1: Option<ContentInfo>,
    pub c2: Option<ContentInfo>,
    pub c3: Option<ContentInfo>,
    pub c4: Option<ContentInfo>,
    pub c5: Option<ContentInfo>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
}

impl From<CurrentAccountMediaInternal> for ProfileContent {
    fn from(value: CurrentAccountMediaInternal) -> Self {
        Self {
            c0: value.profile_content_id_0.map(|c| c.into()),
            c1: value.profile_content_id_1.map(|c| c.into()),
            c2: value.profile_content_id_2.map(|c| c.into()),
            c3: value.profile_content_id_3.map(|c| c.into()),
            c4: value.profile_content_id_4.map(|c| c.into()),
            c5: value.profile_content_id_5.map(|c| c.into()),
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
    pub c0: Option<ContentInfo>,
    pub c1: Option<ContentInfo>,
    pub c2: Option<ContentInfo>,
    pub c3: Option<ContentInfo>,
    pub c4: Option<ContentInfo>,
    pub c5: Option<ContentInfo>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
}

impl From<CurrentAccountMediaInternal> for PendingProfileContent {
    fn from(value: CurrentAccountMediaInternal) -> Self {
        Self {
            c0: value.pending_profile_content_id_0.map(|c| c.into()),
            c1: value.pending_profile_content_id_1.map(|c| c.into()),
            c2: value.pending_profile_content_id_2.map(|c| c.into()),
            c3: value.pending_profile_content_id_3.map(|c| c.into()),
            c4: value.pending_profile_content_id_4.map(|c| c.into()),
            c5: value.pending_profile_content_id_5.map(|c| c.into()),
            grid_crop_size: value.pending_grid_crop_size,
            grid_crop_x: value.pending_grid_crop_x,
            grid_crop_y: value.pending_grid_crop_y,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SecurityContent {
    pub c0: Option<ContentInfo>,
}

impl From<CurrentAccountMediaInternal> for SecurityContent {
    fn from(value: CurrentAccountMediaInternal) -> Self {
        Self {
            c0: value.security_content_id.map(|c| c.into()),
        }
    }
}

/// Security content settings which will be applied when moderation request is
/// accepted.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct PendingSecurityContent {
    pub c0: Option<ContentInfo>,
}

impl From<CurrentAccountMediaInternal> for PendingSecurityContent {
    fn from(value: CurrentAccountMediaInternal) -> Self {
        Self {
            c0: value.pending_security_content_id.map(|c| c.into()),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GetContentQueryParams {
    /// If false media content access is allowed when profile is set as public.
    /// If true media content access is allowed when users are a match.
    #[serde(default)]
    pub is_match: bool,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GetProfileContentQueryParams {
    version: Option<uuid::Uuid>,
    /// If false profile content access is allowed when profile is set as public.
    /// If true profile content access is allowed when users are a match.
    #[serde(default)]
    is_match: bool,
}

impl GetProfileContentQueryParams {
    pub fn version(&self) -> Option<ProfileContentVersion> {
        self.version.map(ProfileContentVersion::new)
    }

    pub fn allow_get_content_if_match(&self) -> bool {
        self.is_match
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetProfileContentResult {
    pub c: Option<ProfileContent>,
    pub v: Option<ProfileContentVersion>,
}

impl GetProfileContentResult {
    pub fn current_version_latest_response(version: ProfileContentVersion) -> Self {
        Self {
            c: None,
            v: Some(version),
        }
    }

    pub fn content_with_version(content: ProfileContent, version: ProfileContentVersion) -> Self {
        Self {
            c: Some(content),
            v: Some(version),
        }
    }

    pub fn empty() -> Self {
        Self {
            c: None,
            v: None,
        }
    }
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
    PartialEq,
    Eq,
    Hash,
    ToSchema,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
#[serde(transparent)]
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
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = BigInt)]
#[serde(transparent)]
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
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = BigInt)]
#[serde(transparent)]
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

#[derive(Debug, Clone, Default, Queryable, Selectable)]
#[diesel(table_name = crate::schema::media_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct MediaStateRaw {
    pub initial_moderation_request_accepted: bool,
}

impl MediaStateRaw {
    pub fn current_moderation_request_is_initial(&self) -> bool {
        !self.initial_moderation_request_accepted
    }
}

/// Version UUID for public profile content.
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
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Binary)]
pub struct ProfileContentVersion {
    v: uuid::Uuid,
}

impl ProfileContentVersion {
    pub(crate) fn new(version: uuid::Uuid) -> Self {
        Self { v: version }
    }

    pub fn new_random() -> Self {
        let version = uuid::Uuid::new_v4();
        Self { v: version }
    }

    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.v
    }
}

diesel_uuid_wrapper!(ProfileContentVersion);
