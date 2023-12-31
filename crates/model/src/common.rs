use std::error::Error;

use base64::Engine;
use diesel::{
    prelude::*,
    sql_types::{BigInt, Binary},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use simple_backend_utils::current_unix_time;
use utoipa::{IntoParams, ToSchema};

use simple_backend_model::{diesel_i64_wrapper, diesel_uuid_wrapper, diesel_i64_try_from};
use crate::{
    AccountState, Capabilities, MessageNumber, schema_sqlite_types::Integer, ContentProcessingId, ContentProcessingState,
};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct BackendVersion {
    /// Backend code version.
    pub backend_code_version: String,
    /// Semver version of the backend.
    pub backend_version: String,
    /// Semver version of the protocol used by the backend.
    pub protocol_version: String,
}

/// Identifier for event.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub enum EventType {
    /// New account state for client.
    /// Data: account_state
    AccountStateChanged,
    /// New capabilities for client.
    /// Data: capabilities
    AccountCapabilitiesChanged,
    NewMessageReceived,
    LikesChanged,
    ReceivedBlocksChanged,
    /// New latest viewed message number changed
    /// Data: latest_viewed_message_changed
    LatestViewedMessageChanged,
    /// Data: content_processing_state_changed
    ContentProcessingStateChanged,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LatestViewedMessageChanged {
    /// Account id of message viewer
    pub account_id_viewer: AccountId,
    /// New value for latest vieqed message
    pub new_latest_viewed_message: MessageNumber,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ContentProcessingStateChanged {
    pub id: ContentProcessingId,
    pub new_state: ContentProcessingState,
}

/// Event to client which is sent through websocket.
///
/// This is not an enum to make generated API bindings more easier to
/// use.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct EventToClient {
    event: EventType,
    /// Data for event AccountStateChanged
    account_state: Option<AccountState>,
    /// Data for event AccountCapabilitiesChanged
    capabilities: Option<Capabilities>,
    /// Data for event LatestViewedMessageChanged
    latest_viewed_message_changed: Option<LatestViewedMessageChanged>,
    /// Data for event ContentProcessingStateChanged
    content_processing_state_changed: Option<ContentProcessingStateChanged>,
}

pub enum EventToClientInternal {
    /// New account state for client
    AccountStateChanged {
        state: AccountState,
    },
    /// New capabilities for client
    AccountCapabilitiesChanged {
        capabilities: Capabilities,
    },
    NewMessageReceived,
    LikesChanged,
    ReceivedBlocksChanged,
    LatestViewedMessageChanged(LatestViewedMessageChanged),
    ContentProcessingStateChanged(ContentProcessingStateChanged),
}

impl From<EventToClientInternal> for EventToClient {
    fn from(internal: EventToClientInternal) -> Self {
        let mut value = Self {
            event: EventType::AccountStateChanged,
            account_state: None,
            capabilities: None,
            latest_viewed_message_changed: None,
            content_processing_state_changed: None,
        };

        match internal {
            EventToClientInternal::AccountStateChanged { state } => {
                value.event = EventType::AccountStateChanged;
                value.account_state = Some(state);
            }
            EventToClientInternal::AccountCapabilitiesChanged { capabilities } => {
                value.event = EventType::AccountCapabilitiesChanged;
                value.capabilities = Some(capabilities);
            }
            EventToClientInternal::NewMessageReceived => {
                value.event = EventType::NewMessageReceived;
            }
            EventToClientInternal::LikesChanged => {
                value.event = EventType::LikesChanged;
            }
            EventToClientInternal::ReceivedBlocksChanged => {
                value.event = EventType::ReceivedBlocksChanged;
            }
            EventToClientInternal::LatestViewedMessageChanged(latest_viewed_message_changed) => {
                value.event = EventType::LatestViewedMessageChanged;
                value.latest_viewed_message_changed = Some(latest_viewed_message_changed);
            }
            EventToClientInternal::ContentProcessingStateChanged(data) => {
                value.event = EventType::ContentProcessingStateChanged;
                value.content_processing_state_changed = Some(data);
            }
        }

        value
    }
}

#[derive(Debug, Clone)]
pub enum NotificationEvent {
    NewMessageReceived,
    LikesChanged,
    ReceivedBlocksChanged,
}

impl From<NotificationEvent> for EventToClientInternal {
    fn from(event: NotificationEvent) -> Self {
        match event {
            NotificationEvent::NewMessageReceived => EventToClientInternal::NewMessageReceived,
            NotificationEvent::LikesChanged => EventToClientInternal::LikesChanged,
            NotificationEvent::ReceivedBlocksChanged => {
                EventToClientInternal::ReceivedBlocksChanged
            }
        }
    }
}

/// Used with database
#[derive(
    Debug,
    Serialize,
    Deserialize,
    ToSchema,
    Clone,
    Eq,
    Hash,
    PartialEq,
    Copy,
    Queryable,
    Identifiable,
    Selectable,
)]
#[diesel(table_name = crate::schema::account_id)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccountIdInternal {
    pub id: AccountIdDb,
    pub uuid: AccountId,
}

impl AccountIdInternal {
    pub fn new(id: AccountIdDb, uuid: AccountId) -> Self {
        Self { id, uuid }
    }

    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.uuid.account_id
    }

    pub fn as_db_id(&self) -> &AccountIdDb {
        &self.id
    }

    pub fn into_db_id(self) -> AccountIdDb {
        self.id
    }

    pub fn row_id(&self) -> i64 {
        self.id.0
    }

    pub fn as_id(&self) -> AccountId {
        self.uuid
    }
}

impl From<AccountIdInternal> for uuid::Uuid {
    fn from(value: AccountIdInternal) -> Self {
        value.uuid.into()
    }
}

impl From<AccountIdInternal> for AccountId {
    fn from(value: AccountIdInternal) -> Self {
        value.as_id()
    }
}

impl std::fmt::Display for AccountIdInternal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

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
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = Binary)]
pub struct AccountId {
    pub account_id: uuid::Uuid,
}

impl AccountId {
    pub fn new(account_id: uuid::Uuid) -> Self {
        Self { account_id }
    }

    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.account_id
    }

    pub fn to_string(&self) -> String {
        self.account_id.hyphenated().to_string()
    }
}

impl From<AccountId> for uuid::Uuid {
    fn from(value: AccountId) -> Self {
        value.account_id
    }
}

diesel_uuid_wrapper!(AccountId);

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Selectable, Queryable)]
#[diesel(table_name = crate::schema::access_token)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccessTokenRaw {
    pub token: Option<String>,
}

/// This is just a random string.
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct AccessToken {
    /// API token which server generates.
    access_token: String,
}

impl AccessToken {
    pub fn generate_new() -> Self {
        Self {
            access_token: uuid::Uuid::new_v4().simple().to_string(),
        }
    }

    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }

    pub fn into_string(self) -> String {
        self.access_token
    }

    pub fn as_str(&self) -> &str {
        &self.access_token
    }
}

#[derive(Debug, Selectable, Queryable)]
#[diesel(table_name = crate::schema::refresh_token)]
#[diesel(check_for_backend(crate::Db))]
pub struct RefreshTokenRaw {
    pub token: Option<Vec<u8>>,
}

/// This is just a really long random number which is Base64 encoded.
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct RefreshToken {
    token: String,
}

impl RefreshToken {
    pub fn generate_new_with_bytes() -> (Self, Vec<u8>) {
        let mut token = Vec::new();

        // TODO: use longer refresh token
        for _ in 1..=2 {
            token.extend(uuid::Uuid::new_v4().to_bytes_le())
        }

        (Self::from_bytes(&token), token)
    }

    pub fn generate_new() -> Self {
        let (token, _bytes) = Self::generate_new_with_bytes();
        token
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            token: base64::engine::general_purpose::STANDARD.encode(data),
        }
    }

    /// String must be base64 encoded
    /// TODO: add checks?
    pub fn from_string(token: String) -> Self {
        Self { token }
    }

    /// Base64 string
    pub fn into_string(self) -> String {
        self.token
    }

    /// Base64 string
    pub fn as_str(&self) -> &str {
        &self.token
    }

    pub fn bytes(&self) -> Result<Vec<u8>, base64::DecodeError> {
        base64::engine::general_purpose::STANDARD.decode(&self.token)
    }
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
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct AccountIdDb(pub i64);

impl AccountIdDb {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    pub fn as_i64(&self) -> &i64 {
        &self.0
    }
}

diesel_i64_wrapper!(AccountIdDb);

#[derive(Debug, Clone, Default, Queryable, Selectable)]
#[diesel(table_name = crate::schema::shared_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct SharedStateInternal {
    pub is_profile_public: bool,
    pub account_state_number: i64,
}

#[derive(Debug, Clone, Default)]
pub struct SharedState {
    pub is_profile_public: bool,
    pub account_state: AccountState,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::next_queue_number)]
#[diesel(check_for_backend(crate::Db))]
pub struct NextQueueNumbersRaw {
    pub queue_type_number: NextQueueNumberType,
    /// Next unused queue number
    pub next_number: i64,
}

#[derive(Debug, Clone, Copy, diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = Integer)]
pub enum NextQueueNumberType {
    MediaModeration = 0,
    InitialMediaModeration = 1,
}

impl TryFrom<i64> for NextQueueNumberType {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let number_type = match value {
            0 => Self::MediaModeration,
            1 => Self::InitialMediaModeration,
            value => return Err(format!("Unknown QueueNumberType value {}", value)),
        };

        Ok(number_type)
    }
}

diesel_i64_try_from!(NextQueueNumberType);

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::queue_entry)]
#[diesel(check_for_backend(crate::Db))]
pub struct QueueEntryRaw {
    pub queue_number: i64,
    pub queue_type_number: NextQueueNumberType,
    pub account_id: AccountIdDb,
}

// #[derive(Debug, Serialize, Deserialize, Clone, Copy, sqlx::Type, PartialEq, Eq, Hash, FromSqlRow, AsExpression)]
// #[diesel(sql_type = BigInt)]
// #[serde(transparent)]
// #[sqlx(transparent)]
// pub struct DbId(pub i64);

// impl DbId {
//     pub fn new(id: i64) -> Self {
//         Self(id)
//     }

//     pub fn as_i64(&self) -> &i64 {
//         &self.0
//     }
// }

// diesel_i64_wrapper!(DbId);

// TODO: Add UnixTime to unix time fields
