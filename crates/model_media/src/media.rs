use diesel::{prelude::*, sql_types::{BigInt, Text}, AsExpression, FromSqlRow};
use model::{sync_version_wrappers, ContentId, ProfileContentVersion, UnixTime};
use model_server_data::{ContentSlot, MediaContentType};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper, diesel_string_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::{
    schema_sqlite_types::Integer, AccountIdDb
};

mod map;
pub use map::*;

mod content;
pub use content::*;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct SlotId {
    pub slot_id: u8,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ContentInfo {
    pub cid: ContentId,
    pub ctype: MediaContentType,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ContentInfoWithFd {
    pub cid: ContentId,
    pub ctype: MediaContentType,
    /// Face detected
    pub fd: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ContentInfoDetailed {
    pub cid: ContentId,
    pub ctype: MediaContentType,
    pub state: ContentModerationState,
    pub slot: Option<ContentSlot>,
    pub secure_capture: bool,
    /// Face detected
    pub fd: bool,
}

/// Content moderation states
///
/// The states grouped like this:
///
/// - InSlot, If user uploads new content to slot the current will be removed.
/// - InModeration, Content is in moderation. User can not remove the content.
/// - ModeratedAsAccepted, Content is moderated as accepted.
///   User can not remove the content until specific time elapses.
/// - ModeratedAsRejected, Content is moderated as rejected.
///   Content deleting is possible.
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
pub enum ContentModerationState {
    InSlot = 0,
    /// InModeration
    WaitingBotOrHumanModeration = 1,
    /// InModeration
    WaitingHumanModeration = 2,
    /// ModeratedAsAccepted
    AcceptedByBot = 3,
    /// ModeratedAsAccepted
    AcceptedByHuman = 4,
    /// ModeratedAsRejected
    RejectedByBot = 5,
    /// ModeratedAsRejected
    RejectedByHuman = 6,
}

impl ContentModerationState {
    pub fn is_rejected(&self) -> bool {
        match self {
            Self::RejectedByBot | Self::RejectedByHuman => true,
            Self::InSlot
            | Self::WaitingBotOrHumanModeration
            | Self::WaitingHumanModeration
            | Self::AcceptedByBot
            | Self::AcceptedByHuman => false,
        }
    }

    pub fn is_accepted(&self) -> bool {
        match self {
            Self::AcceptedByBot | Self::AcceptedByHuman => true,
            Self::InSlot
            | Self::WaitingBotOrHumanModeration
            | Self::WaitingHumanModeration
            | Self::RejectedByBot
            | Self::RejectedByHuman => false,
        }
    }

    pub fn is_moderated(&self) -> bool {
        self.is_rejected() || self.is_accepted()
    }

    pub fn is_in_moderation(&self) -> bool {
        match self {
            Self::WaitingBotOrHumanModeration | Self::WaitingHumanModeration => true,
            Self::InSlot
            | Self::RejectedByBot
            | Self::RejectedByHuman
            | Self::AcceptedByBot
            | Self::AcceptedByHuman => false,
        }
    }

    pub fn is_in_slot(&self) -> bool {
        match self {
            Self::InSlot => true,
            Self::WaitingBotOrHumanModeration
            | Self::WaitingHumanModeration
            | Self::RejectedByBot
            | Self::RejectedByHuman
            | Self::AcceptedByBot
            | Self::AcceptedByHuman => false,
        }
    }
}

impl Default for ContentModerationState {
    fn default() -> Self {
        Self::InSlot
    }
}

diesel_i64_try_from!(ContentModerationState);

#[derive(
    Debug,
    Serialize,
    Deserialize,
    ToSchema,
    Clone,
    Eq,
    Hash,
    PartialEq,
    IntoParams,
    Copy,
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct ContentModerationRejectedReasonCategory {
    pub value: i64,
}

impl ContentModerationRejectedReasonCategory {
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.value
    }
}

diesel_i64_wrapper!(ContentModerationRejectedReasonCategory);

#[derive(
    Debug,
    Deserialize,
    Serialize,
    ToSchema,
    Clone,
    Eq,
    Hash,
    PartialEq,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Text)]
pub struct ContentModerationRejectedReasonDetails {
    value: String,
}

impl ContentModerationRejectedReasonDetails {
    pub fn new(value: String) -> Self {
        Self { value }
    }

    pub fn into_string(self) -> String {
        self.value
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

diesel_string_wrapper!(ContentModerationRejectedReasonDetails);

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::media_content)]
#[diesel(check_for_backend(crate::Db))]
pub struct MediaContentRaw {
    pub id: ContentIdDb,
    pub uuid: ContentId,
    pub account_id: AccountIdDb,
    pub secure_capture: bool,
    pub face_detected: bool,
    pub content_type_number: MediaContentType,
    slot_number: ContentSlot,
    pub creation_unix_time: UnixTime,
    pub initial_content: bool,
    pub moderation_state: ContentModerationState,
    pub moderation_rejected_reason_category: Option<ContentModerationRejectedReasonCategory>,
    pub moderation_rejected_reason_details: Option<ContentModerationRejectedReasonDetails>,
    pub moderation_moderator_account_id: Option<AccountIdDb>,
}

impl MediaContentRaw {
    pub fn slot_number(&self) -> Option<ContentSlot> {
        if self.moderation_state == ContentModerationState::InSlot {
            Some(self.slot_number)
        } else {
            None
        }
    }

    pub fn content_id(&self) -> ContentId {
        self.uuid
    }

    pub fn state(&self) -> ContentModerationState {
        self.moderation_state
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

impl From<MediaContentRaw> for ContentInfoWithFd {
    fn from(value: MediaContentRaw) -> Self {
        ContentInfoWithFd {
            cid: value.uuid,
            ctype: value.content_type_number,
            fd: value.face_detected,
        }
    }
}

impl From<MediaContentRaw> for ContentInfoDetailed {
    fn from(value: MediaContentRaw) -> Self {
        ContentInfoDetailed {
            cid: value.uuid,
            ctype: value.content_type_number,
            state: value.moderation_state,
            slot: value.slot_number(),
            secure_capture: value.secure_capture,
            fd: value.face_detected,
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
}

impl CurrentAccountMediaInternal {
    pub fn iter_all_content(&self) -> impl Iterator<Item = &MediaContentRaw> {
        self.iter_current_profile_content()
            .chain(self.security_content_id.iter())
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
        [Some(self.c0), self.c1, self.c2, self.c3, self.c4, self.c5]
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

/// Current content in public profile.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct MyProfileContent {
    /// Primary profile image which is shown in grid view.
    pub c0: Option<ContentInfoWithFd>,
    pub c1: Option<ContentInfoWithFd>,
    pub c2: Option<ContentInfoWithFd>,
    pub c3: Option<ContentInfoWithFd>,
    pub c4: Option<ContentInfoWithFd>,
    pub c5: Option<ContentInfoWithFd>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
}

impl From<CurrentAccountMediaInternal> for MyProfileContent {
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SecurityContent {
    pub c0: Option<ContentInfoWithFd>,
}

impl From<CurrentAccountMediaInternal> for SecurityContent {
    fn from(value: CurrentAccountMediaInternal) -> Self {
        Self {
            c0: value.security_content_id.map(|c| c.into()),
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
    version: Option<simple_backend_utils::UuidBase64Url>,
    /// If false profile content access is allowed when profile is set as public.
    /// If true profile content access is allowed when users are a match.
    #[serde(default)]
    is_match: bool,
}

impl GetProfileContentQueryParams {
    pub fn version(&self) -> Option<ProfileContentVersion> {
        self.version.map(ProfileContentVersion::new_base_64_url)
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
        Self { c: None, v: None }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetMyProfileContentResult {
    pub c: MyProfileContent,
    pub v: ProfileContentVersion,
    pub sv: ProfileContentSyncVersion,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct AccountContent {
    pub data: Vec<ContentInfoDetailed>,
}

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
    pub profile_content_sync_version: ProfileContentSyncVersion,
}

impl MediaStateRaw {
    pub fn current_moderation_request_is_initial(&self) -> bool {
        !self.initial_moderation_request_accepted
    }
}

sync_version_wrappers!(ProfileContentSyncVersion,);
