use diesel::{AsExpression, FromSqlRow, sql_types::Binary};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper, diesel_uuid_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::{
    AccountId, AccountIdInternal, NotificationIdViewed, NotificationStatus,
    schema_sqlite_types::Integer,
};

/// media_content table primary key
#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = Integer)]
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

/// Content ID for media content.
///
/// Uniqueness is guaranteed for one account so other account might
/// use the same ID for another content.
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
    pub cid: simple_backend_utils::UuidBase64Url,
}

diesel_uuid_wrapper!(ContentId);

impl ContentId {
    pub fn new_random() -> Self {
        Self {
            cid: simple_backend_utils::UuidBase64Url::new_random_id(),
        }
    }

    fn diesel_uuid_wrapper_new(cid: simple_backend_utils::UuidBase64Url) -> Self {
        Self { cid }
    }

    fn diesel_uuid_wrapper_as_uuid(&self) -> &simple_backend_utils::UuidBase64Url {
        &self.cid
    }

    /// File name for unprocessed user uploaded content.
    pub fn raw_content_file_name(&self) -> String {
        format!("{}.raw", self.cid)
    }

    pub fn content_file_name(&self) -> String {
        format!("{}", self.cid)
    }

    pub fn not_in(&self, mut iter: impl Iterator<Item = ContentId>) -> bool {
        !iter.any(|c| c == *self)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ContentIdInternal {
    aid: AccountIdInternal,
    cid: ContentId,
    cid_db: ContentIdDb,
}

impl ContentIdInternal {
    pub fn new(aid: AccountIdInternal, cid: ContentId, cid_db: ContentIdDb) -> Self {
        Self { aid, cid, cid_db }
    }

    pub fn as_db_id(&self) -> &ContentIdDb {
        &self.cid_db
    }

    pub fn account_id(&self) -> AccountId {
        self.aid.uuid
    }

    pub fn content_owner(&self) -> AccountIdInternal {
        self.aid
    }

    pub fn content_id(&self) -> ContentId {
        self.cid
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
    num_enum::TryFromPrimitive,
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

diesel_i64_try_from!(ContentSlot);

/// Content ID which is queued to be processed
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema)]
pub struct ContentProcessingId {
    aid: AccountId,
    slot: ContentSlot,
    /// Server process specific unique ID
    id: i64,
}

impl ContentProcessingId {
    pub fn new(aid: AccountId, slot: ContentSlot, id: i64) -> Self {
        Self { aid, slot, id }
    }

    /// File name for unprocessed user uploaded content.
    pub fn raw_content_file_name(&self) -> String {
        format!("{}_{}.raw", self.id, self.slot as i64)
    }

    pub fn content_file_name(&self) -> String {
        format!("{}_{}", self.id, self.slot as i64)
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
    /// NSFW detected.
    NsfwDetected = 5,
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
    /// Face detected info of the processed content.
    pub fd: Option<bool>,
}

impl ContentProcessingState {
    pub fn empty() -> Self {
        Self {
            state: ContentProcessingStateType::Empty,
            wait_queue_position: None,
            cid: None,
            fd: None,
        }
    }

    pub fn in_queue_state(wait_queue_position: u64) -> Self {
        Self {
            state: ContentProcessingStateType::InQueue,
            wait_queue_position: Some(wait_queue_position),
            cid: None,
            fd: None,
        }
    }

    pub fn change_to_processing(&mut self) {
        self.state = ContentProcessingStateType::Processing;
        self.wait_queue_position = None;
        self.cid = None;
        self.fd = None;
    }

    pub fn change_to_completed(&mut self, content_id: ContentId, face_detected: bool) {
        self.state = ContentProcessingStateType::Completed;
        self.wait_queue_position = None;
        self.cid = Some(content_id);
        self.fd = Some(face_detected);
    }

    pub fn change_to_failed(&mut self) {
        self.state = ContentProcessingStateType::Failed;
        self.wait_queue_position = None;
        self.cid = None;
        self.fd = None;
    }

    pub fn change_to_nsfw_detected(&mut self) {
        self.state = ContentProcessingStateType::NsfwDetected;
        self.wait_queue_position = None;
        self.cid = None;
        self.fd = None;
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
    v: simple_backend_utils::UuidBase64Url,
}

impl ProfileContentVersion {
    pub fn new_base_64_url(version: simple_backend_utils::UuidBase64Url) -> Self {
        Self { v: version }
    }

    fn diesel_uuid_wrapper_new(v: simple_backend_utils::UuidBase64Url) -> Self {
        Self { v }
    }

    pub fn new_random() -> Self {
        Self {
            v: simple_backend_utils::UuidBase64Url::new_random_id(),
        }
    }

    fn diesel_uuid_wrapper_as_uuid(&self) -> &simple_backend_utils::UuidBase64Url {
        &self.v
    }
}

diesel_uuid_wrapper!(ProfileContentVersion);

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, ToSchema)]
pub struct MediaContentModerationCompletedNotification {
    pub accepted: NotificationStatus,
    pub rejected: NotificationStatus,
    pub deleted: NotificationStatus,
}

impl MediaContentModerationCompletedNotification {
    pub fn notifications_viewed(&self) -> bool {
        self.accepted.notification_viewed()
            && self.rejected.notification_viewed()
            && self.deleted.notification_viewed()
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, ToSchema)]
pub struct MediaContentModerationCompletedNotificationViewed {
    pub accepted: NotificationIdViewed,
    pub rejected: NotificationIdViewed,
    pub deleted: NotificationIdViewed,
}
