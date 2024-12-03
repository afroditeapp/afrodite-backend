use diesel::{
    sql_types::Binary,
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_uuid_wrapper;
use utoipa::{IntoParams, ToSchema};

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

/// Content ID which is queued to be processed
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
pub struct ContentProcessingId {
    id: simple_backend_utils::UuidBase64Url,
}

impl ContentProcessingId {
    pub fn new_random_id() -> Self {
        Self {
            id: simple_backend_utils::UuidBase64Url::new_random_id(),
        }
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
