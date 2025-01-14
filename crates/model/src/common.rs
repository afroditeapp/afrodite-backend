use base64::Engine;
use chrono::NaiveDate;
use diesel::{
    prelude::*,
    sql_types::{BigInt, Binary},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper, diesel_uuid_wrapper};
use utils::random_bytes::random_128_bits;
use utoipa::{IntoParams, ToSchema};

use crate::{
    schema_sqlite_types::Integer, Account, AccountCreatedTime, AccountStateContainer, ContentProcessingId, ContentProcessingState, MessageNumber, ProfileVisibility
};

pub mod sync_version;
pub use sync_version::*;

pub mod version;
pub use version::*;

pub mod push_notifications;
pub use push_notifications::*;

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
    /// Account state, profile visibility or permissions changed.
    AccountStateChanged,
    NewMessageReceived,
    ReceivedLikesChanged,
    ReceivedBlocksChanged,
    SentLikesChanged,
    SentBlocksChanged,
    MatchesChanged,
    /// New latest viewed message number changed
    /// Data: latest_viewed_message_changed
    LatestViewedMessageChanged,
    /// Data: content_processing_state_changed
    ContentProcessingStateChanged,
    AvailableProfileAttributesChanged,
    ProfileChanged,
    NewsCountChanged,
    InitialContentModerationCompleted,
    MediaContentChanged,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct LatestViewedMessageChanged {
    /// Account id of message viewer
    pub viewer: AccountId,
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
    /// Data for event LatestViewedMessageChanged
    latest_viewed_message_changed: Option<LatestViewedMessageChanged>,
    /// Data for event ContentProcessingStateChanged
    content_processing_state_changed: Option<ContentProcessingStateChanged>,
}

/// Internal data type for events.
///
/// If data is not included in the event it might be too large to send
/// over WebSocket as it might block more important events for some time
/// depending on network connection speed.
#[derive(Debug, Clone)]
pub enum EventToClientInternal {
    /// Account state, profile visibility or permissions changed.
    AccountStateChanged,
    LatestViewedMessageChanged(LatestViewedMessageChanged),
    ContentProcessingStateChanged(ContentProcessingStateChanged),
    NewMessageReceived,
    ReceivedLikesChanged,
    ReceivedBlocksChanged,
    SentLikesChanged,
    SentBlocksChanged,
    MatchesChanged,
    AvailableProfileAttributesChanged,
    ProfileChanged,
    NewsChanged,
    InitialContentModerationCompleted,
    MediaContentChanged,
}

impl From<&EventToClientInternal> for EventType {
    fn from(value: &EventToClientInternal) -> Self {
        use EventToClientInternal::*;
        match value {
            LatestViewedMessageChanged(_) => Self::LatestViewedMessageChanged,
            ContentProcessingStateChanged(_) => Self::ContentProcessingStateChanged,
            AccountStateChanged => Self::AccountStateChanged,
            NewMessageReceived => Self::NewMessageReceived,
            ReceivedLikesChanged => Self::ReceivedLikesChanged,
            ReceivedBlocksChanged => Self::ReceivedBlocksChanged,
            SentLikesChanged => Self::SentLikesChanged,
            SentBlocksChanged => Self::SentBlocksChanged,
            MatchesChanged => Self::MatchesChanged,
            AvailableProfileAttributesChanged => Self::AvailableProfileAttributesChanged,
            ProfileChanged => Self::ProfileChanged,
            NewsChanged => Self::NewsCountChanged,
            InitialContentModerationCompleted => Self::InitialContentModerationCompleted,
            MediaContentChanged => Self::MediaContentChanged,
        }
    }
}

impl From<EventToClientInternal> for EventToClient {
    fn from(internal: EventToClientInternal) -> Self {
        let mut value = Self {
            event: (&internal).into(),
            latest_viewed_message_changed: None,
            content_processing_state_changed: None,
        };

        use EventToClientInternal::*;

        match internal {
            LatestViewedMessageChanged(v) => value.latest_viewed_message_changed = Some(v),
            ContentProcessingStateChanged(v) => value.content_processing_state_changed = Some(v),
            AccountStateChanged
            | NewMessageReceived
            | ReceivedLikesChanged
            | ReceivedBlocksChanged
            | SentLikesChanged
            | SentBlocksChanged
            | MatchesChanged
            | AvailableProfileAttributesChanged
            | ProfileChanged
            | NewsChanged
            | InitialContentModerationCompleted
            | MediaContentChanged => (),
        }

        value
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NotificationEvent {
    NewMessageReceived,
    ReceivedLikesChanged,
    InitialContentModerationCompleted,
    NewsChanged,
}

impl From<NotificationEvent> for EventToClientInternal {
    fn from(event: NotificationEvent) -> Self {
        match event {
            NotificationEvent::NewMessageReceived => EventToClientInternal::NewMessageReceived,
            NotificationEvent::ReceivedLikesChanged => EventToClientInternal::ReceivedLikesChanged,
            NotificationEvent::InitialContentModerationCompleted => {
                EventToClientInternal::InitialContentModerationCompleted
            }
            NotificationEvent::NewsChanged => EventToClientInternal::NewsChanged,
        }
    }
}

/// Used with database
#[derive(
    Debug,
    Serialize,
    Deserialize,
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
    pub aid: simple_backend_utils::UuidBase64Url,
}

impl AccountId {
    pub fn new_random() -> Self {
        Self {
            aid: simple_backend_utils::UuidBase64Url::new_random_id(),
        }
    }

    fn diesel_uuid_wrapper_new(aid: simple_backend_utils::UuidBase64Url) -> Self {
        Self { aid }
    }

    fn diesel_uuid_wrapper_as_uuid(&self) -> &simple_backend_utils::UuidBase64Url {
        &self.aid
    }

    pub fn new_base_64_url(account_id: simple_backend_utils::UuidBase64Url) -> Self {
        Self { aid: account_id }
    }

    pub fn for_debugging_only_zero() -> Self {
        Self {
            aid: simple_backend_utils::UuidBase64Url::for_debugging_only_zero(),
        }
    }
}

diesel_uuid_wrapper!(AccountId);

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.aid)
    }
}

#[derive(Debug, Selectable, Queryable)]
#[diesel(table_name = crate::schema::access_token)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccessTokenRaw {
    pub token: Option<String>,
}

/// AccessToken is used as a short lived token for API access.
///
/// The token is 256 bit random value which is base64url encoded
/// without padding. The previous format is used because
/// the token is transferred as HTTP header value.
/// The token lenght in characters is 43.
///
/// OWASP recommends at least 128 bit session IDs.
/// https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct AccessToken {
    /// API token which server generates.
    access_token: String,
}

impl AccessToken {
    pub fn generate_new_with_bytes() -> (Self, Vec<u8>) {
        // Generate 256 bit token
        let mut token = Vec::new();
        for _ in 1..=2 {
            token.extend(random_128_bits())
        }
        let access_token = Self {
            access_token: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&token),
        };
        (access_token, token)
    }

    pub fn generate_new() -> Self {
        Self::generate_new_with_bytes().0
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

/// Refresh token is long lived token used for getting new access tokens.
///
/// Refresh token is 3072 bit value which is Base64 encoded.
/// The token lenght in characters is 512.
///
/// Why 3072 bits? Microsoft LinkedIn API uses about 500 character refresh
/// tokens and 3072 bits is near that value.
///
/// https://learn.microsoft.com/en-us/linkedin/shared/authentication/programmatic-refresh-tokens
///
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct RefreshToken {
    /// Base64 encoded random number.
    token: String,
}

impl RefreshToken {
    pub fn generate_new_with_bytes() -> (Self, Vec<u8>) {
        let mut token = Vec::new();

        for _ in 1..=24 {
            token.extend(random_128_bits())
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
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = BigInt)]
#[serde(transparent)]
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

#[derive(Debug, Clone, Default, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::shared_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct SharedStateRaw {
    pub account_state_initial_setup_completed: bool,
    pub account_state_banned: bool,
    pub account_state_pending_deletion: bool,
    pub profile_visibility_state_number: ProfileVisibility,
    pub sync_version: AccountSyncVersion,
    pub unlimited_likes: bool,
    pub birthdate: Option<NaiveDate>,
    pub is_bot_account: bool,
    pub account_created_unix_time: AccountCreatedTime,
}

#[derive(Debug, Clone, Default, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::shared_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccountStateRelatedSharedState {
    pub profile_visibility_state_number: ProfileVisibility,
    pub account_state_initial_setup_completed: bool,
    pub account_state_banned: bool,
    pub account_state_pending_deletion: bool,
    pub sync_version: AccountSyncVersion,
}

#[derive(Debug, Clone, Default, Queryable, Selectable)]
#[diesel(table_name = crate::schema::shared_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct OtherSharedState {
    pub unlimited_likes: bool,
    pub birthdate: Option<NaiveDate>,
    pub is_bot_account: bool,
    pub account_created_unix_time: AccountCreatedTime,
}

impl AccountStateRelatedSharedState {
    pub fn profile_visibility(&self) -> ProfileVisibility {
        self.profile_visibility_state_number
    }

    pub fn state_container(&self) -> AccountStateContainer {
        AccountStateContainer {
            initial_setup_completed: self.account_state_initial_setup_completed,
            banned: self.account_state_banned,
            pending_deletion: self.account_state_pending_deletion,
        }
    }
}

impl From<Account> for AccountStateRelatedSharedState {
    fn from(account: Account) -> Self {
        Self {
            profile_visibility_state_number: account.profile_visibility(),
            account_state_initial_setup_completed: account.state_container().initial_setup_completed,
            account_state_banned: account.state_container().banned,
            account_state_pending_deletion: account.state_container().pending_deletion,
            sync_version: account.sync_version(),
        }
    }
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
            value => return Err(format!("Unknown NextQueueNumberType value {}", value)),
        };

        Ok(number_type)
    }
}

diesel_i64_try_from!(NextQueueNumberType);

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::queue_entry)]
#[diesel(check_for_backend(crate::Db))]
pub struct QueueEntryRaw {
    pub queue_number: QueueNumber,
    pub queue_type_number: NextQueueNumberType,
    pub account_id: AccountIdDb,
}

#[derive(
    Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = BigInt)]
#[serde(transparent)]
pub struct QueueNumber(pub i64);

impl QueueNumber {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    pub fn as_i64(&self) -> &i64 {
        &self.0
    }
}

diesel_i64_wrapper!(QueueNumber);

#[cfg(test)]
mod tests {
    use base64::Engine;

    use crate::{AccessToken, RefreshToken};

    fn is_base64url_no_padding_character(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '-' || c == '_'
    }

    fn is_base64_character(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='
    }

    #[test]
    fn access_token_lenght_is_256_bits() {
        let data_256_bit = [0u8; 256 / 8];
        let wanted_len = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(data_256_bit)
            .len();
        let token = AccessToken::generate_new();
        assert_eq!(token.access_token.len(), wanted_len);
        assert_eq!(token.access_token.len(), 43);
    }

    #[test]
    fn access_token_contains_only_allowed_characters() {
        let token = AccessToken::generate_new();
        for c in token.access_token.chars() {
            assert!(is_base64url_no_padding_character(c));
        }
    }

    #[test]
    fn refresh_token_lenght_is_24_uuids() {
        let data_24_uuid = [0u8; 128 * 24 / 8];
        let wanted_len = base64::engine::general_purpose::STANDARD
            .encode(data_24_uuid)
            .len();
        let token = RefreshToken::generate_new();
        assert_eq!(token.token.len(), wanted_len);
        assert_eq!(token.token.len(), 512);
    }

    #[test]
    fn refresh_token_contains_only_allowed_characters() {
        let token = RefreshToken::generate_new();
        for c in token.token.chars() {
            assert!(is_base64_character(c));
        }
    }
}
