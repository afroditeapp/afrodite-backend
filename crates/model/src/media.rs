use diesel::{
    AsExpression, FromSqlRow,
    sql_types::{BigInt, Binary, SmallInt},
};
use serde::{Deserialize, Serialize};
pub use simple_backend_model::{ImageProcessingDynamicConfig, ImageProcessingWarnings};
use simple_backend_model::{SimpleDieselEnum, diesel_i64_wrapper, diesel_uuid_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::{AccountId, AccountIdInternal};

/// media_content table primary key
#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = BigInt)]
#[serde(transparent)]
pub struct ContentIdDb(pub i64);

impl TryFrom<i64> for ContentIdDb {
    type Error = String;

    fn try_from(id: i64) -> Result<Self, Self::Error> {
        Ok(Self(id))
    }
}

impl AsRef<i64> for ContentIdDb {
    fn as_ref(&self) -> &i64 {
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

impl TryFrom<simple_backend_utils::UuidBase64Url> for ContentId {
    type Error = String;

    fn try_from(cid: simple_backend_utils::UuidBase64Url) -> Result<Self, Self::Error> {
        Ok(Self { cid })
    }
}

impl AsRef<simple_backend_utils::UuidBase64Url> for ContentId {
    fn as_ref(&self) -> &simple_backend_utils::UuidBase64Url {
        &self.cid
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
    SimpleDieselEnum,
    diesel::FromSqlRow,
    diesel::AsExpression,
    num_enum::TryFromPrimitive,
)]
#[diesel(sql_type = SmallInt)]
#[repr(i16)]
pub enum ContentSlot {
    Content0 = 0,
    Content1 = 1,
    Content2 = 2,
    Content3 = 3,
    Content4 = 4,
    Content5 = 5,
    Content6 = 6,
}

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

    pub fn server_process_id(&self) -> i64 {
        self.id
    }

    /// File name for unprocessed user uploaded content.
    pub fn raw_content_file_name(&self) -> String {
        format!("{}_{}.raw", self.id, self.slot as i64)
    }

    pub fn content_file_name(&self) -> String {
        format!("{}_{}", self.id, self.slot as i64)
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Default,
    Deserialize,
    Serialize,
    ToSchema,
    num_enum::TryFromPrimitive,
)]
#[repr(u8)]
pub enum ContentProcessingStateType {
    /// This content slot is empty.
    #[default]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentProcessingStateInternal {
    Empty,
    InQueue { wait_queue_position: i64 },
    Processing,
    Completed { content_id: ContentId, fd: bool },
    Failed,
    NsfwDetected,
}

impl ContentProcessingStateInternal {
    pub fn state_type(&self) -> ContentProcessingStateType {
        match self {
            Self::Empty => ContentProcessingStateType::Empty,
            Self::InQueue { .. } => ContentProcessingStateType::InQueue,
            Self::Processing => ContentProcessingStateType::Processing,
            Self::Completed { .. } => ContentProcessingStateType::Completed,
            Self::Failed => ContentProcessingStateType::Failed,
            Self::NsfwDetected => ContentProcessingStateType::NsfwDetected,
        }
    }

    pub fn to_external(&self) -> ContentProcessingState {
        match self {
            Self::Empty => ContentProcessingState {
                state: ContentProcessingStateType::Empty,
                wait_queue_position: None,
                cid: None,
                fd: None,
            },
            Self::InQueue {
                wait_queue_position,
            } => ContentProcessingState {
                state: ContentProcessingStateType::InQueue,
                wait_queue_position: Some(*wait_queue_position),
                cid: None,
                fd: None,
            },
            Self::Processing => ContentProcessingState {
                state: ContentProcessingStateType::Processing,
                wait_queue_position: None,
                cid: None,
                fd: None,
            },
            Self::Completed { content_id, fd } => ContentProcessingState {
                state: ContentProcessingStateType::Completed,
                wait_queue_position: None,
                cid: Some(*content_id),
                fd: Some(*fd),
            },
            Self::Failed => ContentProcessingState {
                state: ContentProcessingStateType::Failed,
                wait_queue_position: None,
                cid: None,
                fd: None,
            },
            Self::NsfwDetected => ContentProcessingState {
                state: ContentProcessingStateType::NsfwDetected,
                wait_queue_position: None,
                cid: None,
                fd: None,
            },
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct ContentProcessingState {
    pub state: ContentProcessingStateType,
    /// Current position in processing queue.
    ///
    /// If ProcessingContentId is added to empty queue, then
    /// this will be 1.
    ///
    /// Use i64 as Dart has only signed integers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_queue_position: Option<i64>,
    /// Content ID of the processed content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<ContentId>,
    /// Face detected info of the processed content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fd: Option<bool>,
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

    pub fn new_random() -> Self {
        Self {
            v: simple_backend_utils::UuidBase64Url::new_random_id(),
        }
    }
}

impl TryFrom<simple_backend_utils::UuidBase64Url> for ProfileContentVersion {
    type Error = String;

    fn try_from(v: simple_backend_utils::UuidBase64Url) -> Result<Self, Self::Error> {
        Ok(Self { v })
    }
}

impl AsRef<simple_backend_utils::UuidBase64Url> for ProfileContentVersion {
    fn as_ref(&self) -> &simple_backend_utils::UuidBase64Url {
        &self.v
    }
}

diesel_uuid_wrapper!(ProfileContentVersion);
