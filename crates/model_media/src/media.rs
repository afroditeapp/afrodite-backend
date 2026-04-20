use diesel::{prelude::*, sql_types::SmallInt};
use model::{
    ContentId, ContentIdDb, ContentSlot, MediaVerificationStatusFlags, ProfileContentVersion,
    UnixTime, sync_version_wrappers,
};
use model_server_data::{MediaContentType, ProfileContentEditedTime};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::SimpleDieselEnum;
use utoipa::{IntoParams, ToSchema};

use crate::AccountIdDb;

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
    pub accepted: bool,
    #[serde(default = "value_bool_true", skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    /// Face detected (automatic or manual)
    pub face_detected: bool,
    /// Face verified against current security content (automatic or manual)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face_verified: Option<bool>,
}

fn value_bool_true() -> bool {
    true
}

fn value_jpeg_image() -> MediaContentType {
    MediaContentType::JpegImage
}

fn value_is_true(v: &bool) -> bool {
    *v
}

fn value_is_jpeg_image(v: &MediaContentType) -> bool {
    *v == MediaContentType::JpegImage
}

fn value_default_grid_crop_size() -> f32 {
    1.0
}

fn value_default_grid_crop_xy() -> f32 {
    0.0
}

fn value_is_default_grid_crop_size(v: &f32) -> bool {
    *v == value_default_grid_crop_size()
}

fn value_is_default_grid_crop_xy(v: &f32) -> bool {
    *v == value_default_grid_crop_xy()
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct MyContentInfo {
    pub cid: ContentId,
    pub ctype: MediaContentType,
    /// Face detected (automatic or manual)
    pub face_detected: bool,
    /// Face verified against current security content (automatic or manual)
    pub face_verified: Option<bool>,
    pub state: ContentModerationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_reason_category: Option<MediaContentModerationRejectedReasonCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_reason_details: Option<MediaContentModerationRejectedReasonDetails>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ContentInfoDetailed {
    pub cid: ContentId,
    pub ctype: MediaContentType,
    pub state: ContentModerationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<ContentSlot>,
    pub secure_capture: bool,
    /// Face detected (automatic)
    face_detected: bool,
    /// Manual face detected value set by admin
    #[serde(skip_serializing_if = "Option::is_none")]
    face_detected_manual: Option<bool>,
    /// Face verified against current security content (automatic or manual)
    #[serde(skip_serializing_if = "Option::is_none")]
    face_verified: Option<bool>,
    /// Manual face verified value set by admin
    #[serde(skip_serializing_if = "Option::is_none")]
    face_verified_manual: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_start_time: Option<UnixTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage_end_time: Option<UnixTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_reason_category: Option<MediaContentModerationRejectedReasonCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_reason_details: Option<MediaContentModerationRejectedReasonDetails>,
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
    SimpleDieselEnum,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = SmallInt)]
#[repr(i16)]
#[derive(Default)]
pub enum ContentModerationState {
    #[default]
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

#[derive(Debug, Clone, PartialEq, Queryable, Selectable)]
#[diesel(table_name = crate::schema::media_content)]
#[diesel(check_for_backend(crate::Db))]
pub struct MediaContentRaw {
    pub id: ContentIdDb,
    pub uuid: ContentId,
    pub account_id: AccountIdDb,
    pub secure_capture: bool,
    face_detected: bool,
    face_detected_manual: Option<bool>,
    face_verified: Option<bool>,
    face_verified_manual: Option<bool>,
    pub content_type_number: MediaContentType,
    slot_number: ContentSlot,
    pub creation_unix_time: UnixTime,
    pub initial_content: bool,
    pub moderation_state: ContentModerationState,
    pub moderation_rejected_reason_category: Option<MediaContentModerationRejectedReasonCategory>,
    pub moderation_rejected_reason_details: Option<MediaContentModerationRejectedReasonDetails>,
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

    /// Get the effective face detected value.
    /// If face_detected_manual is set, it overrides face_detected.
    pub fn effective_face_detected(&self) -> bool {
        self.face_detected_manual.unwrap_or(self.face_detected)
    }

    /// Admin set face detected value
    pub fn manual_face_detected(&self) -> Option<bool> {
        self.face_detected_manual
    }

    /// Get the effective face verified value.
    /// If face_verified_manual is set, it overrides face_verified.
    pub fn effective_face_verified(&self) -> Option<bool> {
        self.face_verified_manual.or(self.face_verified)
    }

    /// Admin set face verified value
    pub fn manual_face_verified(&self) -> Option<bool> {
        self.face_verified_manual
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

impl From<MediaContentRaw> for MyContentInfo {
    fn from(value: MediaContentRaw) -> Self {
        MyContentInfo {
            cid: value.uuid,
            ctype: value.content_type_number,
            face_detected: value.effective_face_detected(),
            face_verified: value.effective_face_verified(),
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
            face_detected: value.face_detected,
            face_detected_manual: value.face_detected_manual,
            face_verified: value.face_verified,
            face_verified_manual: value.face_verified_manual,
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
    pub security_content_set_unix_time: Option<UnixTime>,
    pub profile_content_version_uuid: ProfileContentVersion,
    pub profile_content_id_0: Option<ContentIdDb>,
    pub profile_content_id_1: Option<ContentIdDb>,
    pub profile_content_id_2: Option<ContentIdDb>,
    pub profile_content_id_3: Option<ContentIdDb>,
    pub profile_content_id_4: Option<ContentIdDb>,
    pub profile_content_id_5: Option<ContentIdDb>,
    pub grid_crop_size: f32,
    pub grid_crop_x: f32,
    pub grid_crop_y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CurrentAccountMediaInternal {
    pub security_content_id: Option<MediaContentRaw>,
    pub security_content_set_time: Option<UnixTime>,
    pub profile_content_version_uuid: ProfileContentVersion,
    pub profile_content_id_0: Option<MediaContentRaw>,
    pub profile_content_id_1: Option<MediaContentRaw>,
    pub profile_content_id_2: Option<MediaContentRaw>,
    pub profile_content_id_3: Option<MediaContentRaw>,
    pub profile_content_id_4: Option<MediaContentRaw>,
    pub profile_content_id_5: Option<MediaContentRaw>,
    pub grid_crop_size: f32,
    pub grid_crop_x: f32,
    pub grid_crop_y: f32,
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
        self.iter_current_profile_content().map(|v| ContentInfo {
            cid: v.content_id(),
            ctype: v.content_type(),
            accepted: v.state().is_accepted(),
            face_detected: v.effective_face_detected(),
            face_verified: v.effective_face_verified(),
        })
    }

    pub fn iter_current_profile_content_info_fd(&self) -> impl Iterator<Item = MyContentInfo> + '_ {
        self.iter_current_profile_content().map(|v| MyContentInfo {
            cid: v.content_id(),
            ctype: v.content_type(),
            face_detected: v.effective_face_detected(),
            face_verified: v.effective_face_verified(),
            state: v.state(),
            rejected_reason_category: v.moderation_rejected_reason_category,
            rejected_reason_details: v.moderation_rejected_reason_details.clone(),
        })
    }

    pub fn media_verification_status_flags(&self) -> MediaVerificationStatusFlags {
        let mut profile_content_exists = false;
        let mut any_face_verified = false;
        let mut all_face_verified = true;

        for content in self.iter_current_profile_content() {
            profile_content_exists = true;
            let verified = content.effective_face_verified() == Some(true);
            any_face_verified |= verified;
            all_face_verified &= verified;
        }

        let mut flags = MediaVerificationStatusFlags::empty();
        if any_face_verified {
            flags |= MediaVerificationStatusFlags::PROFILE_CONTENT_FACE_VERIFIED_ANY;
        }
        if profile_content_exists && all_face_verified {
            flags |= MediaVerificationStatusFlags::PROFILE_CONTENT_FACE_VERIFIED_ALL;
        }

        flags
    }
}

/// Update normal or pending profile content
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SetProfileContent {
    /// First image is primary profile image which is shown in grid view.
    ///
    /// One content ID is required.
    ///
    /// Max item count is 6. Extra items are ignored.
    pub content: Vec<ContentId>,
    #[serde(
        default = "value_default_grid_crop_size",
        skip_serializing_if = "value_is_default_grid_crop_size"
    )]
    #[schema(default = value_default_grid_crop_size)]
    pub grid_crop_size: f32,
    #[serde(
        default = "value_default_grid_crop_xy",
        skip_serializing_if = "value_is_default_grid_crop_xy"
    )]
    #[schema(default = value_default_grid_crop_xy)]
    pub grid_crop_x: f32,
    #[serde(
        default = "value_default_grid_crop_xy",
        skip_serializing_if = "value_is_default_grid_crop_xy"
    )]
    #[schema(default = value_default_grid_crop_xy)]
    pub grid_crop_y: f32,
}

impl SetProfileContent {
    pub fn iter(&self) -> impl Iterator<Item = ContentId> + '_ {
        self.content.iter().copied()
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct UpdateProfileContentResult {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_content_at_index_does_not_exist: Option<i64>,
}

impl UpdateProfileContentResult {
    pub fn success() -> Self {
        Self::default()
    }

    pub fn error_content_does_not_exist(index: i64) -> Self {
        Self {
            error: true,
            error_content_at_index_does_not_exist: Some(index),
        }
    }
}

/// Value for profile verification status flags.
///
/// - PROFILE_CONTENT_FACE_VERIFIED_ANY = 0x1. At least one current profile
///   picture has effective face verified value true.
/// - PROFILE_CONTENT_FACE_VERIFIED_ALL = 0x2. All current profile pictures
///   have effective face verified value true. For empty profile picture list
///   this bit must be unset.
#[derive(Debug, PartialEq, Serialize, ToSchema)]
pub struct MediaVerificationStatus {
    pub v: i16,
}

impl From<MediaVerificationStatusFlags> for MediaVerificationStatus {
    fn from(value: MediaVerificationStatusFlags) -> Self {
        Self { v: value.bits() }
    }
}

/// Current content in public profile.
#[derive(Debug, PartialEq, Serialize, ToSchema)]
pub struct ProfileContent {
    /// First image is primary profile image which is shown in grid view.
    pub content: Vec<ContentInfo>,
    pub verification_status: MediaVerificationStatus,
    #[serde(
        default = "value_default_grid_crop_size",
        skip_serializing_if = "value_is_default_grid_crop_size"
    )]
    #[schema(default = value_default_grid_crop_size)]
    pub grid_crop_size: f32,
    #[serde(
        default = "value_default_grid_crop_xy",
        skip_serializing_if = "value_is_default_grid_crop_xy"
    )]
    #[schema(default = value_default_grid_crop_xy)]
    pub grid_crop_x: f32,
    #[serde(
        default = "value_default_grid_crop_xy",
        skip_serializing_if = "value_is_default_grid_crop_xy"
    )]
    #[schema(default = value_default_grid_crop_xy)]
    pub grid_crop_y: f32,
}

impl From<CurrentAccountMediaInternal> for ProfileContent {
    fn from(value: CurrentAccountMediaInternal) -> Self {
        Self {
            content: value.iter_current_profile_content_info().collect(),
            verification_status: value.media_verification_status_flags().into(),
            grid_crop_size: value.grid_crop_size,
            grid_crop_x: value.grid_crop_x,
            grid_crop_y: value.grid_crop_y,
        }
    }
}

/// Current content in public profile.
#[derive(Debug, Serialize, ToSchema)]
pub struct MyProfileContent {
    /// First image is primary profile image which is shown in grid view.
    pub content: Vec<MyContentInfo>,
    pub verification_status: MediaVerificationStatus,
    #[serde(
        default = "value_default_grid_crop_size",
        skip_serializing_if = "value_is_default_grid_crop_size"
    )]
    #[schema(default = value_default_grid_crop_size)]
    pub grid_crop_size: f32,
    #[serde(
        default = "value_default_grid_crop_xy",
        skip_serializing_if = "value_is_default_grid_crop_xy"
    )]
    #[schema(default = value_default_grid_crop_xy)]
    pub grid_crop_x: f32,
    #[serde(
        default = "value_default_grid_crop_xy",
        skip_serializing_if = "value_is_default_grid_crop_xy"
    )]
    #[schema(default = value_default_grid_crop_xy)]
    pub grid_crop_y: f32,
}

impl From<CurrentAccountMediaInternal> for MyProfileContent {
    fn from(value: CurrentAccountMediaInternal) -> Self {
        Self {
            content: value.iter_current_profile_content_info_fd().collect(),
            verification_status: value.media_verification_status_flags().into(),
            grid_crop_size: value.grid_crop_size,
            grid_crop_x: value.grid_crop_x,
            grid_crop_y: value.grid_crop_y,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct SecurityContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c: Option<MyContentInfo>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Serialize, ToSchema)]
pub struct GetProfileContentResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ProfileContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<ProfileContentVersion>,
}

impl GetProfileContentResult {
    pub fn current_version_latest_response(version: ProfileContentVersion) -> Self {
        Self {
            content: None,
            version: Some(version),
        }
    }

    pub fn content_with_version(content: ProfileContent, version: ProfileContentVersion) -> Self {
        Self {
            content: Some(content),
            version: Some(version),
        }
    }

    pub fn empty() -> Self {
        Self {
            content: None,
            version: None,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GetMediaContentResult {
    pub profile_content: MyProfileContent,
    pub profile_content_version: ProfileContentVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_content: Option<MyContentInfo>,
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

#[derive(Debug, Clone, Default, Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::media_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct MediaStateRaw {
    pub media_content_sync_version: MediaContentSyncVersion,
    pub profile_content_edited_unix_time: ProfileContentEditedTime,
}

sync_version_wrappers!(
    /// Sync version for profile and security content
    MediaContentSyncVersion,
);
