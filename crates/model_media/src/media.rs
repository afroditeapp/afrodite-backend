use diesel::{AsExpression, FromSqlRow, prelude::*, sql_types::BigInt};
use model::{
    ContentId, ContentIdDb, ContentSlot, ProfileContentVersion, UnixTime, sync_version_wrappers,
};
use model_server_data::{MediaContentType, ProfileContentEditedTime};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::{AccountIdDb, schema_sqlite_types::Integer};

mod map;
pub use map::*;

mod content;
pub use content::*;

mod report;
pub use report::*;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct SlotId {
    pub slot_id: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ContentInfo {
    pub cid: ContentId,
    /// Default value is not set to API doc as the API doc will then have
    /// "oneOf" property and Dart code generator does not support it.
    ///
    /// Default value is [MediaContentType::JpegImage].
    #[serde(
        default = "value_jpeg_image",
        skip_serializing_if = "value_is_jpeg_image"
    )]
    #[schema(value_type = Option<MediaContentType>)]
    pub ctype: MediaContentType,
    #[serde(default = "value_bool_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    /// Accepted
    pub a: bool,
    #[serde(default = "value_bool_false", skip_serializing_if = "value_is_false")]
    #[schema(default = false)]
    /// Primary content
    ///
    /// The first profile content is not primary content when admin
    /// deletes the first profile content and the second content does
    /// not have face detected.
    pub p: bool,
}

fn value_bool_true() -> bool {
    true
}

fn value_bool_false() -> bool {
    false
}

fn value_jpeg_image() -> MediaContentType {
    MediaContentType::JpegImage
}

fn value_is_true(v: &bool) -> bool {
    *v
}

fn value_is_false(v: &bool) -> bool {
    !*v
}

fn value_is_jpeg_image(v: &MediaContentType) -> bool {
    *v == MediaContentType::JpegImage
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ContentInfoWithFd {
    pub cid: ContentId,
    pub ctype: MediaContentType,
    /// Face detected
    pub fd: bool,
    pub state: ContentModerationState,
    pub rejected_reason_category: Option<MediaContentModerationRejectedReasonCategory>,
    pub rejected_reason_details: MediaContentModerationRejectedReasonDetails,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ContentInfoDetailed {
    pub cid: ContentId,
    pub ctype: MediaContentType,
    pub state: ContentModerationState,
    pub slot: Option<ContentSlot>,
    pub secure_capture: bool,
    /// Face detected
    pub fd: bool,
    pub usage_start_time: Option<UnixTime>,
    pub usage_end_time: Option<UnixTime>,
    pub rejected_reason_category: Option<MediaContentModerationRejectedReasonCategory>,
    pub rejected_reason_details: MediaContentModerationRejectedReasonDetails,
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

#[derive(Debug, Clone, PartialEq, Queryable, Selectable)]
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
    pub moderation_rejected_reason_category: Option<MediaContentModerationRejectedReasonCategory>,
    pub moderation_rejected_reason_details: MediaContentModerationRejectedReasonDetails,
    pub moderation_moderator_account_id: Option<AccountIdDb>,
    pub usage_start_unix_time: Option<UnixTime>,
    pub usage_end_unix_time: Option<UnixTime>,
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

    pub fn removable_by_user(&self, remove_wait_time: u32) -> bool {
        if self.usage_start_unix_time.is_some() {
            return false;
        }

        if let Some(usage_end_time) = self.usage_end_unix_time {
            let removing_allowed_time = *usage_end_time.as_i64() + remove_wait_time as i64;
            let current_time = UnixTime::current_time();
            *current_time.as_i64() > removing_allowed_time
        } else {
            true
        }
    }
}

impl From<MediaContentRaw> for ContentId {
    fn from(value: MediaContentRaw) -> Self {
        value.uuid
    }
}

impl From<MediaContentRaw> for ContentInfoWithFd {
    fn from(value: MediaContentRaw) -> Self {
        ContentInfoWithFd {
            cid: value.uuid,
            ctype: value.content_type_number,
            fd: value.face_detected,
            state: value.state(),
            rejected_reason_category: value.moderation_rejected_reason_category,
            rejected_reason_details: value.moderation_rejected_reason_details,
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
            usage_end_time: value.usage_end_unix_time,
            usage_start_time: value.usage_start_unix_time,
            rejected_reason_category: value.moderation_rejected_reason_category,
            rejected_reason_details: value.moderation_rejected_reason_details,
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

#[derive(Debug, Clone, PartialEq)]
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

    pub fn iter_current_profile_content_info(&self) -> impl Iterator<Item = ContentInfo> + '_ {
        self.iter_current_profile_content()
            .enumerate()
            .map(|(i, v)| ContentInfo {
                cid: v.content_id(),
                ctype: v.content_type(),
                a: v.state().is_accepted(),
                p: (i == 0 && v.face_detected),
            })
    }

    pub fn iter_current_profile_content_info_fd(
        &self,
    ) -> impl Iterator<Item = ContentInfoWithFd> + '_ {
        self.iter_current_profile_content()
            .map(|v| ContentInfoWithFd {
                cid: v.content_id(),
                ctype: v.content_type(),
                fd: v.face_detected,
                state: v.state(),
                rejected_reason_category: v.moderation_rejected_reason_category,
                rejected_reason_details: v.moderation_rejected_reason_details.clone(),
            })
    }
}

/// Update normal or pending profile content
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SetProfileContent {
    /// Primary profile image which is shown in grid view.
    ///
    /// One content ID is required.
    ///
    /// Max item count is 6. Extra items are ignored.
    pub c: Vec<ContentId>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
}

impl SetProfileContent {
    pub fn iter(&self) -> impl Iterator<Item = ContentId> + '_ {
        self.c.iter().copied()
    }
}

/// Current content in public profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct ProfileContent {
    /// Primary profile image which is shown in grid view.
    pub c: Vec<ContentInfo>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
}

impl From<CurrentAccountMediaInternal> for ProfileContent {
    fn from(value: CurrentAccountMediaInternal) -> Self {
        Self {
            c: value.iter_current_profile_content_info().collect(),
            grid_crop_size: value.grid_crop_size,
            grid_crop_x: value.grid_crop_x,
            grid_crop_y: value.grid_crop_y,
        }
    }
}

/// Current content in public profile.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct MyProfileContent {
    /// Primary profile image which is shown in grid view.
    pub c: Vec<ContentInfoWithFd>,
    pub grid_crop_size: Option<f64>,
    pub grid_crop_x: Option<f64>,
    pub grid_crop_y: Option<f64>,
}

impl From<CurrentAccountMediaInternal> for MyProfileContent {
    fn from(value: CurrentAccountMediaInternal) -> Self {
        Self {
            c: value.iter_current_profile_content_info_fd().collect(),
            grid_crop_size: value.grid_crop_size,
            grid_crop_x: value.grid_crop_x,
            grid_crop_y: value.grid_crop_y,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SecurityContent {
    pub c: Option<ContentInfoWithFd>,
}

impl SecurityContent {
    pub fn new(value: CurrentAccountMediaInternal) -> Self {
        Self {
            c: value.security_content_id.map(|c| c.into()),
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetMediaContentResult {
    pub profile_content: MyProfileContent,
    pub profile_content_version: ProfileContentVersion,
    pub security_content: Option<ContentInfoWithFd>,
    pub sync_version: MediaContentSyncVersion,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct AccountContent {
    pub data: Vec<ContentInfoDetailed>,
    pub max_content_count: u8,
    /// Content can be removed when
    /// - [ContentInfoDetailed::usage_end_time] and
    ///   [ContentInfoDetailed::usage_start_time] are empty
    /// - [ContentInfoDetailed::usage_end_time] is not empty
    ///   and [Self::unused_content_wait_seconds] has elapsed from the
    ///   [ContentInfoDetailed::usage_end_time]
    pub unused_content_wait_seconds: u32,
}

#[derive(Debug, Clone, Default, Queryable, Selectable)]
#[diesel(table_name = crate::schema::media_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct MediaStateRaw {
    pub initial_moderation_request_accepted: bool,
    pub media_content_sync_version: MediaContentSyncVersion,
    pub profile_content_edited_unix_time: ProfileContentEditedTime,
}

impl MediaStateRaw {
    pub fn current_moderation_request_is_initial(&self) -> bool {
        !self.initial_moderation_request_accepted
    }
}

sync_version_wrappers!(
    /// Sync version for profile and security content
    MediaContentSyncVersion,
);
