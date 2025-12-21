use chrono::NaiveDate;
use diesel::prelude::*;
use model::{ClientType, ClientVersion, EmailLoginToken, NewsSyncVersion, UnixTime};
use model_server_data::{
    AppleAccountId, AuthPair, EmailAddress, GoogleAccountId, PublicationId, SignInWithInfo,
};
use model_server_state::DemoAccountToken;
use serde::{Deserialize, Serialize};
use utils::time::age_in_years_from_birthdate;
use utoipa::{IntoParams, ToSchema};

use crate::{AccountId, AccountIdDb};

mod email;
pub use email::*;

mod news;
pub use news::*;

mod ban;
pub use ban::*;

mod report;
pub use report::*;

mod client_features;
pub use client_features::*;

#[derive(Debug, Default, Deserialize, Serialize, ToSchema, Clone, PartialEq)]
pub struct LoginResult {
    /// If `None`, the client is unsupported.
    #[serde(skip_serializing_if = "Option::is_none")]
    tokens: Option<AuthPair>,

    /// Account ID of current account. If `None`, the client is unsupported.
    #[serde(skip_serializing_if = "Option::is_none")]
    aid: Option<AccountId>,

    /// Current email of current account. If `None`, if email address is not
    /// set or the client version is unsupported.
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<EmailAddress>,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error: bool,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_unsupported_client: bool,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_sign_in_with_email_unverified: bool,

    /// This might be true, when registering new account using
    /// sign in with login method.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_email_already_used: bool,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_account_locked: bool,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_invalid_email_login_token: bool,
}

impl LoginResult {
    pub fn ok(tokens: AuthPair, aid: AccountId, email: Option<EmailAddress>) -> Self {
        Self {
            tokens: Some(tokens),
            aid: Some(aid),
            email,
            ..Default::default()
        }
    }

    pub fn error_unsupported_client() -> Self {
        Self {
            error: true,
            error_unsupported_client: true,
            ..Default::default()
        }
    }

    pub fn error_sign_in_with_email_unverified() -> Self {
        Self {
            error: true,
            error_sign_in_with_email_unverified: true,
            ..Default::default()
        }
    }

    pub fn error_email_already_used() -> Self {
        Self {
            error: true,
            error_email_already_used: true,
            ..Default::default()
        }
    }

    pub fn error_account_locked() -> Self {
        Self {
            error: true,
            error_account_locked: true,
            ..Default::default()
        }
    }

    pub fn error_invalid_email_login_token() -> Self {
        Self {
            error: true,
            error_invalid_email_login_token: true,
            ..Default::default()
        }
    }

    pub fn aid(&self) -> Option<AccountId> {
        self.aid
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct ClientInfo {
    pub client_type: ClientType,
    pub client_version: ClientVersion,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct DemoAccountLoginToAccount {
    pub token: DemoAccountToken,
    pub aid: AccountId,
    pub client_info: ClientInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct RequestEmailLoginToken {
    pub email: EmailAddress,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct RequestEmailLoginTokenResult {
    /// Client token to be used together with the email token.
    /// Always returned to prevent email enumeration attacks.
    pub client_token: EmailLoginToken,
    /// Token validity duration in seconds
    pub token_validity_seconds: i64,
    /// Minimum wait duration between token requests in seconds
    pub resend_wait_seconds: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct EmailLogin {
    pub client_info: ClientInfo,
    pub client_token: EmailLoginToken,
    pub email_token: EmailLoginToken,
}

#[derive(Debug, Clone, Default, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::account_email_address_state)]
#[diesel(check_for_backend(crate::Db))]
#[diesel(treat_none_as_null = true)]
pub struct EmailAddressStateInternal {
    pub email: Option<EmailAddress>,
    pub email_change: Option<EmailAddress>,
    pub email_change_unix_time: Option<UnixTime>,
    pub email_change_verification_token: Option<Vec<u8>>,
    pub email_change_verified: bool,
    pub email_login_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmailLoginTokens {
    pub client_token: Option<Vec<u8>>,
    pub email_token: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct EmailAddressState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<EmailAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_change: Option<EmailAddress>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub email_change_verified: bool,
    /// API route handler sets this value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_change_completion_time: Option<UnixTime>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    #[schema(default = true)]
    pub email_login_enabled: bool,
}

fn default_true() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct InitEmailChange {
    pub new_email: EmailAddress,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default, PartialEq, Eq)]
pub struct SetAccountSetup {
    /// String date with "YYYY-MM-DD" format.
    ///
    /// This is not required at the moment to reduce sensitive user data.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub birthdate: Option<NaiveDate>,
    pub is_adult: bool,
}

impl SetAccountSetup {
    pub fn is_valid(&self) -> bool {
        let birthdate_is_valid = if let Some(birthdate) = self.birthdate {
            let age = age_in_years_from_birthdate(birthdate);
            18 <= age && age <= 150
        } else {
            true
        };

        birthdate_is_valid && self.is_adult
    }
}

#[derive(
    Debug,
    Clone,
    Deserialize,
    Serialize,
    ToSchema,
    Default,
    PartialEq,
    Eq,
    Queryable,
    Selectable,
    Insertable,
)]
#[diesel(table_name = crate::schema::account_setup)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccountSetup {
    #[schema(value_type = Option<String>)]
    #[serde(skip_serializing_if = "Option::is_none")]
    birthdate: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_adult: Option<bool>,
}

impl AccountSetup {
    pub fn is_valid(&self) -> bool {
        self.is_adult == Some(true)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, IntoParams)]
pub struct BooleanSetting {
    pub value: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetAccountDeletionRequestResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automatic_deletion_allowed: Option<UnixTime>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetAccountBanTimeResult {
    /// If `None` the account is not banned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banned_until: Option<UnixTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_category: Option<AccountBanReasonCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_details: Option<AccountBanReasonDetails>,
}

#[derive(Deserialize, ToSchema)]
pub struct SignInWithLoginInfo {
    pub client_info: ClientInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apple: Option<SignInWithAppleInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google: Option<SignInWithGoogleInfo>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub disable_registering: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct SignInWithAppleInfo {
    pub token: String,
    /// Base64 URL (with possible padding) encoded nonce.
    ///
    /// The token contains Base64 URL (with possible padding) encoded SHA-256
    /// of the nonce.
    pub nonce: String,
}

#[derive(Deserialize, ToSchema)]
pub struct SignInWithGoogleInfo {
    pub token: String,
    /// Base64 URL (with possible padding) encoded nonce.
    ///
    /// The token contains Base64 URL (with possible padding) encoded SHA-256
    /// of the nonce.
    pub nonce: String,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable)]
#[diesel(table_name = crate::schema::sign_in_with_info)]
#[diesel(check_for_backend(crate::Db))]
pub struct SignInWithInfoRaw {
    pub account_id: AccountIdDb,
    pub google_account_id: Option<GoogleAccountId>,
    pub apple_account_id: Option<AppleAccountId>,
}

impl From<SignInWithInfoRaw> for SignInWithInfo {
    fn from(raw: SignInWithInfoRaw) -> Self {
        Self {
            google_account_id: raw.google_account_id,
            apple_account_id: raw.apple_account_id,
        }
    }
}

/// Used only for user data export
#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::account_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccountStateTableRaw {
    account_deletion_request_unix_time: Option<UnixTime>,
    account_banned_reason_category: Option<i16>,
    account_banned_reason_details: Option<AccountBanReasonDetails>,
    account_banned_until_unix_time: Option<UnixTime>,
    account_banned_state_change_unix_time: Option<UnixTime>,
    news_sync_version: NewsSyncVersion,
    unread_news_count: i64,
    account_created_unix_time: UnixTime,
    account_locked: bool,
}

/// Global state for account component
#[derive(Debug, Default, Clone, PartialEq, Queryable, Selectable)]
#[diesel(table_name = crate::schema::account_global_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccountGlobalState {
    pub admin_access_granted_count: i64,
    pub next_news_publication_id: PublicationId,
}

impl AccountGlobalState {
    /// Key for the only row in the table
    pub const ACCOUNT_GLOBAL_STATE_ROW_TYPE: i32 = 0;
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct RemoteBotLogin {
    pub aid: AccountId,
    pub password: String,
}
