
use diesel::{
    sql_types::{Binary, BigInt},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper, diesel_uuid_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::{schema_sqlite_types::Integer, EnumParsingError};

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
    pub cid: simple_backend_utils::UuidBase64Url,
}

diesel_uuid_wrapper!(ContentId);

impl ContentId {
    fn new_base_64_url(content_id: simple_backend_utils::UuidBase64Url) -> Self {
        Self { cid: content_id }
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

/// Content ID which is queued to be processed
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct ContentProcessingId {
    id: simple_backend_utils::UuidBase64Url,
}

impl ContentProcessingId {
    pub fn new_random_id() -> Self {
        Self { id: simple_backend_utils::UuidBase64Url::new_random_id() }
    }

    pub fn to_content_id(&self) -> ContentId {
        ContentId::new_base_64_url(self.id)
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

    pub fn change_to_completed(
        &mut self,
        content_id: ContentId,
        face_detected: bool,
    ) {
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
}

/// Subset of NextQueueNumberType containing only moderation queue types.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema)]
pub enum ModerationQueueType {
    MediaModeration,
    InitialMediaModeration,
}

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
        Self { v: simple_backend_utils::UuidBase64Url::new_random_id() }
    }

    fn diesel_uuid_wrapper_as_uuid(&self) -> &simple_backend_utils::UuidBase64Url {
        &self.v
    }
}

diesel_uuid_wrapper!(ProfileContentVersion);
