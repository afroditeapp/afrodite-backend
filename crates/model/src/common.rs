use base64::Engine;
use chrono::NaiveDate;
use diesel::{
    AsExpression, FromSqlRow,
    prelude::*,
    sql_types::{BigInt, Binary},
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{
    ScheduledMaintenanceStatus, UnixTime, diesel_i64_wrapper, diesel_uuid_wrapper,
};
use utils::random_bytes::random_128_bits;
use utoipa::{IntoParams, ToSchema};

use crate::{
    Account, AccountStateContainer, ContentProcessingId, ContentProcessingState,
    InitialSetupCompletedTime, IpAddressInternal, ProfileVisibility,
};

pub mod api_usage;
pub use api_usage::*;

pub mod data_export;
pub use data_export::*;

pub mod sync_version;
pub use sync_version::*;

pub mod version;
pub use version::*;

pub mod push_notifications;
pub use push_notifications::*;

pub mod report;
pub use report::*;

pub mod client_config;
pub use client_config::*;

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
    /// Data: content_processing_state_changed
    ContentProcessingStateChanged,
    ClientConfigChanged,
    ProfileChanged,
    NewsCountChanged,
    MediaContentModerationCompleted,
    MediaContentChanged,
    DailyLikesLeftChanged,
    ScheduledMaintenanceStatus,
    ProfileStringModerationCompleted,
    AutomaticProfileSearchCompleted,
    AdminNotification,
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
    /// Data for event ContentProcessingStateChanged
    content_processing_state_changed: Option<ContentProcessingStateChanged>,
    /// Data for event ScheduledMaintenanceStatus
    scheduled_maintenance_status: Option<ScheduledMaintenanceStatus>,
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
    ContentProcessingStateChanged(ContentProcessingStateChanged),
    NewMessageReceived,
    ReceivedLikesChanged,
    ClientConfigChanged,
    ProfileChanged,
    NewsChanged,
    MediaContentModerationCompleted,
    MediaContentChanged,
    DailyLikesLeftChanged,
    ScheduledMaintenanceStatus(ScheduledMaintenanceStatus),
    ProfileStringModerationCompleted,
    AutomaticProfileSearchCompleted,
    AdminNotification,
}

impl From<&EventToClientInternal> for EventType {
    fn from(value: &EventToClientInternal) -> Self {
        use EventToClientInternal::*;
        match value {
            ContentProcessingStateChanged(_) => Self::ContentProcessingStateChanged,
            AccountStateChanged => Self::AccountStateChanged,
            NewMessageReceived => Self::NewMessageReceived,
            ReceivedLikesChanged => Self::ReceivedLikesChanged,
            ClientConfigChanged => Self::ClientConfigChanged,
            ProfileChanged => Self::ProfileChanged,
            NewsChanged => Self::NewsCountChanged,
            MediaContentModerationCompleted => Self::MediaContentModerationCompleted,
            MediaContentChanged => Self::MediaContentChanged,
            DailyLikesLeftChanged => Self::DailyLikesLeftChanged,
            ScheduledMaintenanceStatus(_) => Self::ScheduledMaintenanceStatus,
            ProfileStringModerationCompleted => Self::ProfileStringModerationCompleted,
            AutomaticProfileSearchCompleted => Self::AutomaticProfileSearchCompleted,
            AdminNotification => Self::AdminNotification,
        }
    }
}

impl From<EventToClientInternal> for EventToClient {
    fn from(internal: EventToClientInternal) -> Self {
        let mut value = Self {
            event: (&internal).into(),
            content_processing_state_changed: None,
            scheduled_maintenance_status: None,
        };

        use EventToClientInternal::*;

        match internal {
            ContentProcessingStateChanged(v) => value.content_processing_state_changed = Some(v),
            ScheduledMaintenanceStatus(v) => value.scheduled_maintenance_status = Some(v),
            AccountStateChanged
            | NewMessageReceived
            | ReceivedLikesChanged
            | ClientConfigChanged
            | ProfileChanged
            | NewsChanged
            | MediaContentModerationCompleted
            | MediaContentChanged
            | DailyLikesLeftChanged
            | ProfileStringModerationCompleted
            | AutomaticProfileSearchCompleted
            | AdminNotification => (),
        }

        value
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NotificationEvent {
    NewMessageReceived,
    ReceivedLikesChanged,
    MediaContentModerationCompleted,
    NewsChanged,
    ProfileStringModerationCompleted,
    AutomaticProfileSearchCompleted,
    AdminNotification,
}

impl From<NotificationEvent> for EventToClientInternal {
    fn from(event: NotificationEvent) -> Self {
        match event {
            NotificationEvent::NewMessageReceived => EventToClientInternal::NewMessageReceived,
            NotificationEvent::ReceivedLikesChanged => EventToClientInternal::ReceivedLikesChanged,
            NotificationEvent::MediaContentModerationCompleted => {
                EventToClientInternal::MediaContentModerationCompleted
            }
            NotificationEvent::NewsChanged => EventToClientInternal::NewsChanged,
            NotificationEvent::ProfileStringModerationCompleted => {
                EventToClientInternal::ProfileStringModerationCompleted
            }
            NotificationEvent::AutomaticProfileSearchCompleted => {
                EventToClientInternal::AutomaticProfileSearchCompleted
            }
            NotificationEvent::AdminNotification => EventToClientInternal::AdminNotification,
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

impl From<AccountIdInternal> for AccountIdDb {
    fn from(value: AccountIdInternal) -> Self {
        *value.as_db_id()
    }
}

impl From<AccountIdInternal> for AccountId {
    fn from(value: AccountIdInternal) -> Self {
        value.as_id()
    }
}

impl std::fmt::Display for AccountIdInternal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
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

#[derive(Debug, Clone)]
pub struct LoginSession {
    pub access_token: AccessToken,
    pub access_token_unix_time: AccessTokenUnixTime,
    pub access_token_ip_address: IpAddressInternal,
    pub refresh_token: RefreshToken,
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
    token: String,
}

impl AccessToken {
    pub fn generate_new_with_bytes() -> (Self, Vec<u8>) {
        // Generate 256 bit token
        let mut token = Vec::new();
        for _ in 1..=2 {
            token.extend(random_128_bits())
        }
        let access_token = Self {
            token: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&token),
        };
        (access_token, token)
    }

    pub fn generate_new() -> Self {
        Self::generate_new_with_bytes().0
    }

    pub fn new(token: String) -> Self {
        Self { token }
    }

    pub fn into_string(self) -> String {
        self.token
    }

    pub fn as_str(&self) -> &str {
        &self.token
    }
}

#[derive(Debug, Clone, Copy, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = BigInt)]
pub struct AccessTokenUnixTime {
    pub ut: UnixTime,
}

impl AccessTokenUnixTime {
    pub fn new(ut: i64) -> Self {
        Self {
            ut: UnixTime::new(ut),
        }
    }

    pub fn as_i64(&self) -> &i64 {
        self.ut.as_i64()
    }

    pub fn current_time() -> Self {
        Self {
            ut: UnixTime::current_time(),
        }
    }
}

diesel_i64_wrapper!(AccessTokenUnixTime);

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
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    FromSqlRow,
    AsExpression,
    ToSchema,
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
    pub initial_setup_completed_unix_time: InitialSetupCompletedTime,
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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default, PartialEq, Eq)]
pub struct LatestBirthdate {
    #[schema(value_type = Option<String>)]
    pub birthdate: Option<NaiveDate>,
}

#[derive(Debug, Clone, Default, Queryable, Selectable)]
#[diesel(table_name = crate::schema::shared_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct OtherSharedState {
    pub unlimited_likes: bool,
    birthdate: Option<NaiveDate>,
    pub is_bot_account: bool,
    pub initial_setup_completed_unix_time: InitialSetupCompletedTime,
}

impl OtherSharedState {
    pub fn latest_birthdate(&self) -> LatestBirthdate {
        LatestBirthdate {
            birthdate: self.birthdate,
        }
    }
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
            account_state_initial_setup_completed: account
                .state_container()
                .initial_setup_completed,
            account_state_banned: account.state_container().banned,
            account_state_pending_deletion: account.state_container().pending_deletion,
            sync_version: account.sync_version(),
        }
    }
}

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
        assert_eq!(token.token.len(), wanted_len);
        assert_eq!(token.token.len(), 43);
    }

    #[test]
    fn access_token_contains_only_allowed_characters() {
        let token = AccessToken::generate_new();
        for c in token.token.chars() {
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
