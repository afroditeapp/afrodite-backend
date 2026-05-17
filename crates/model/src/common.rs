use base64::Engine;
use diesel::{
    AsExpression, FromSqlRow,
    prelude::*,
    sql_types::{BigInt, Binary, SmallInt},
};
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use simple_backend_model::{
    ScheduledMaintenanceStatus, UnixTime, diesel_i64_wrapper, diesel_uuid_wrapper,
};
use simple_backend_utils::{UuidBase64Url, time::DurationValue};
use utils::random_bytes::random_128_bits;
use utoipa::{IntoParams, ToSchema};

use crate::{
    Account, AccountStateContainer, ContentProcessingStateInternal, InitialSetupCompletedTime,
    IpAddressInternal, OnlineStatusUpdate, ProfileLink, ProfileVisibility,
};

pub mod api_usage;
pub use api_usage::*;

pub mod data_export;
pub use data_export::*;

pub mod notification;
pub use notification::*;

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

pub mod websocket;
pub use websocket::*;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ServerVersion {
    /// Server code version.
    pub server_code_version: String,
    /// Semver version of the server.
    pub server_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ManualServerMaintenanceInfoForAnotherServer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<crate::StringResource>,
}

#[derive(Debug, Clone)]
pub struct ContentProcessingStateChanged {
    pub processing_id_from_client: u8,
    pub new_state: ContentProcessingStateInternal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum ResponseResetProfilePagingStatus {
    Success = 0,
    RateLimited = 1,
    InternalServerError = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum ResponseNextProfilePageStatus {
    Success = 0,
    InvalidIteratorSessionId = 1,
    RateLimited = 2,
    InternalServerError = 3,
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
    PendingChatNotificationsChanged,
    PendingAppNotificationsChanged,
    WebSocketConnectionAttemptsRemaining {
        remaining: u8,
    },
    AppUpdateAvailable,
    ReceivedLikesChanged,
    ClientConfigChanged,
    ProfileChanged,
    ResponseResetProfilePaging {
        request_id: u8,
        status: ResponseResetProfilePagingStatus,
        iterator_session_id: Option<i64>,
    },
    ResponseNextProfilePage {
        request_id: u8,
        status: ResponseNextProfilePageStatus,
        profiles: Vec<ProfileLink>,
    },
    ResponseAutomaticProfileSearchResetProfilePaging {
        request_id: u8,
        status: ResponseResetProfilePagingStatus,
        iterator_session_id: Option<i64>,
    },
    ResponseAutomaticProfileSearchNextProfilePage {
        request_id: u8,
        status: ResponseNextProfilePageStatus,
        profiles: Vec<ProfileLink>,
    },
    NewsChanged,
    MediaContentChanged,
    AccountVerificationQueuePositionChanged {
        queue_position: Option<u8>,
    },
    DailyLikesLeftChanged,
    ScheduledMaintenanceStatus(ScheduledMaintenanceStatus),
    AdminBotNotification(crate::AdminBotNotificationTypes),
    RequestAdminBotConfigWarnings {
        request_id: u8,
    },
    PushNotificationInfoChanged,
    TypingStart(AccountId),
    TypingStop(AccountId),
    OnlineStatusUpdated(OnlineStatusUpdate),
    MessageDeliveryInfoChanged,
    LatestSeenMessageChanged,
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

/// This is quaranteed to not be reused for another account
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

    pub fn new_base_64_url(account_id: simple_backend_utils::UuidBase64Url) -> Self {
        Self { aid: account_id }
    }

    pub fn for_debugging_only_zero() -> Self {
        Self {
            aid: simple_backend_utils::UuidBase64Url::for_debugging_only_zero(),
        }
    }
}

impl TryFrom<simple_backend_utils::UuidBase64Url> for AccountId {
    type Error = String;

    fn try_from(aid: simple_backend_utils::UuidBase64Url) -> Result<Self, Self::Error> {
        Ok(Self { aid })
    }
}

impl TryFrom<String> for AccountId {
    type Error = String;

    fn try_from(aid: String) -> Result<Self, Self::Error> {
        Ok(Self {
            aid: UuidBase64Url::from_text(&aid)?,
        })
    }
}

impl AsRef<simple_backend_utils::UuidBase64Url> for AccountId {
    fn as_ref(&self) -> &simple_backend_utils::UuidBase64Url {
        &self.aid
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
    pub access_token_previous: Option<AccessToken>,
    pub access_token_ip_address: IpAddressInternal,
    pub access_token_ip_address_previous: Option<IpAddressInternal>,
    pub refresh_token: RefreshToken,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccessTokenType {
    Current,
    Previous,
}

impl LoginSession {
    fn is_ip_valid(&self, ip: std::net::IpAddr) -> bool {
        self.access_token_ip_address.to_ip_addr() == ip
            || self
                .access_token_ip_address_previous
                .map(|a| a.to_ip_addr() == ip)
                .unwrap_or(false)
    }

    fn is_access_token_valid(&self, access_token_type: AccessTokenType, websocket: bool) -> bool {
        let expires_in = match access_token_type {
            AccessTokenType::Current => {
                if websocket {
                    DurationValue::from_days(7)
                } else {
                    DurationValue::from_days(14)
                }
            }
            AccessTokenType::Previous => {
                if self.access_token_previous.is_some() {
                    DurationValue::from_seconds(60)
                } else {
                    return false;
                }
            }
        };

        !self
            .access_token_unix_time
            .ut
            .duration_value_elapsed(expires_in)
    }

    pub fn is_valid(
        &self,
        ip: std::net::IpAddr,
        access_token_type: AccessTokenType,
        websocket: bool,
    ) -> bool {
        self.is_ip_valid(ip) && self.is_access_token_valid(access_token_type, websocket)
    }
}

/// AccessToken is used as a short lived token for API access.
///
/// The token is 256 bit random value which is base64url encoded
/// without padding. The previous format is used because
/// the token is transferred as HTTP header value.
/// The token length in characters is 43.
///
/// OWASP recommends at least 128 bit session IDs.
/// https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html
#[derive(Debug, Deserialize, Serialize, ToSchema, IntoParams, Clone, Eq, Hash, PartialEq)]
pub struct AccessToken {
    /// Base64 URL safe without padding
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

    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            token: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data),
        }
    }

    pub fn bytes(&self) -> Result<Vec<u8>, base64::DecodeError> {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&self.token)
    }
}

#[derive(Debug, Clone, Copy, Default, FromSqlRow, AsExpression)]
#[diesel(sql_type = BigInt)]
pub struct AccessTokenUnixTime {
    pub ut: UnixTime,
}

impl AccessTokenUnixTime {
    pub fn current_time() -> Self {
        Self {
            ut: UnixTime::current_time(),
        }
    }
}

impl TryFrom<i64> for AccessTokenUnixTime {
    type Error = String;

    fn try_from(ut: i64) -> Result<Self, Self::Error> {
        Ok(Self {
            ut: UnixTime::new(ut),
        })
    }
}

impl AsRef<i64> for AccessTokenUnixTime {
    fn as_ref(&self) -> &i64 {
        self.ut.as_i64()
    }
}

diesel_i64_wrapper!(AccessTokenUnixTime);

/// Email login token is a 128 bit token used for email-based authentication
/// where client receives one token via API and another is sent via email.
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct EmailLoginToken {
    /// Base64 URL safe without padding
    token: String,
}

impl EmailLoginToken {
    pub fn generate_new_with_bytes() -> (Self, Vec<u8>) {
        // Generate 128 bit token
        let token = random_128_bits();
        let email_token = Self {
            token: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(token),
        };
        (email_token, token.to_vec())
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

    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            token: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data),
        }
    }

    pub fn bytes(&self) -> Result<Vec<u8>, base64::DecodeError> {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&self.token)
    }
}

/// Refresh token is long lived token used for getting new access tokens.
///
/// Refresh token is 256 bit value which is Base64 encoded.
/// The token length in characters is 44.
///
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct RefreshToken {
    /// Base64 encoded random number.
    token: String,
}

impl RefreshToken {
    pub fn generate_new_with_bytes() -> (Self, Vec<u8>) {
        let mut token = Vec::new();

        for _ in 1..=2 {
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

/// This is quaranteed to not be reused for another account
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

impl TryFrom<i64> for AccountIdDb {
    type Error = String;

    fn try_from(id: i64) -> Result<Self, Self::Error> {
        Ok(Self(id))
    }
}

impl AsRef<i64> for AccountIdDb {
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}

diesel_i64_wrapper!(AccountIdDb);

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Serialize,
    TryFromPrimitive,
    simple_backend_model::SimpleDieselEnum,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = SmallInt)]
#[repr(i16)]
pub enum BotAccountType {
    Admin = 0,
    User = 1,
}

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
    pub bot_account_type_number: Option<BotAccountType>,
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
    pub email_verified: bool,
    pub age_verified: bool,
}

#[derive(Debug, Clone, Default, Queryable, Selectable)]
#[diesel(table_name = crate::schema::shared_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct OtherSharedState {
    pub unlimited_likes: bool,
    bot_account_type_number: Option<BotAccountType>,
    pub initial_setup_completed_unix_time: InitialSetupCompletedTime,
}

impl OtherSharedState {
    pub fn set_bot_account_type_number(&mut self, bot_type: BotAccountType) {
        self.bot_account_type_number = Some(bot_type);
    }

    pub fn bot_account_type_number(&self) -> Option<BotAccountType> {
        self.bot_account_type_number
    }

    pub fn is_bot(&self) -> bool {
        self.bot_account_type_number.is_some()
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
            email_verified: account.email_verified(),
            age_verified: account.age_verified(),
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
    fn access_token_length_is_256_bits() {
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
    fn refresh_token_length_is_256_bits() {
        let data_256_bit = [0u8; 256 / 8];
        let wanted_len = base64::engine::general_purpose::STANDARD
            .encode(data_256_bit)
            .len();
        let token = RefreshToken::generate_new();
        assert_eq!(token.token.len(), wanted_len);
        assert_eq!(token.token.len(), 44);
    }

    #[test]
    fn refresh_token_contains_only_allowed_characters() {
        let token = RefreshToken::generate_new();
        for c in token.token.chars() {
            assert!(is_base64_character(c));
        }
    }
}
