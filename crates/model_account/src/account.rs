use diesel::prelude::*;
use model::{
    ClientLanguage, ClientType, ClientVersion, EmailLoginToken, NewsSyncVersion, UnixTime,
};
use model_server_data::{
    AppleAccountId, AuthPair, EmailAddress, GoogleAccountId, PublicationId, SignInWithInfo,
};
use model_server_state::DemoAccountToken;
use serde::{Deserialize, Serialize};
use simple_backend_model::AppAttestation;
use utoipa::{IntoParams, ToSchema};

use crate::{AccountId, AccountIdDb, AccountVerificationErrorFlagsValue, VerificationMethod};

mod association;
pub use association::*;

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

pub mod verification;
pub use verification::{
    AccountVerificationDataInternal, AccountVerificationQueueStatus,
    PostAccountVerificationQueueItem, PostAccountVerificationQueueItemResult, PostAgeVerification,
    PostAgeVerificationResult,
};

#[derive(Debug, Default, Deserialize, Serialize, ToSchema, Clone, PartialEq)]
pub struct LoginResult {
    /// If `None`, the client is unsupported.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    tokens: Option<AuthPair>,

    /// Account ID of current account. If `None`, the client is unsupported.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    aid: Option<AccountId>,

    /// Current email of current account. If `None`, if email address is not
    /// set or the client version is unsupported.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
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

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_registration_platform_disabled: bool,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_registration_all_platforms_disabled: bool,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_login_platform_disabled: bool,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_login_all_platforms_disabled: bool,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_app_attestation_failed: bool,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_app_attestation_device_integrity: bool,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_app_attestation_app_integrity: bool,
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

    pub fn error_registration_platform_disabled() -> Self {
        Self {
            error: true,
            error_registration_platform_disabled: true,
            ..Default::default()
        }
    }

    pub fn error_registration_all_platforms_disabled() -> Self {
        Self {
            error: true,
            error_registration_all_platforms_disabled: true,
            ..Default::default()
        }
    }

    pub fn error_login_platform_disabled() -> Self {
        Self {
            error: true,
            error_login_platform_disabled: true,
            ..Default::default()
        }
    }

    pub fn error_login_all_platforms_disabled() -> Self {
        Self {
            error: true,
            error_login_all_platforms_disabled: true,
            ..Default::default()
        }
    }

    pub fn error_app_attestation_failed() -> Self {
        Self {
            error: true,
            error_app_attestation_failed: true,
            ..Default::default()
        }
    }

    pub fn error_app_attestation_device_integrity() -> Self {
        Self {
            error: true,
            error_app_attestation_device_integrity: true,
            ..Default::default()
        }
    }

    pub fn error_app_attestation_app_integrity() -> Self {
        Self {
            error: true,
            error_app_attestation_app_integrity: true,
            ..Default::default()
        }
    }

    pub fn aid(&self) -> Option<AccountId> {
        self.aid
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct ClientInfo {
    pub client_type: ClientType,
    pub client_version: ClientVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub app_attestation: Option<AppAttestation>,
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
    /// Use this to bypass [LoginResult::error_email_registration_ip_address_limit_reached]
    /// when user wants to login to existing account.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub login_only: bool,
    pub client_type: ClientType,
    /// Preferred language for token emails.
    /// If `None`, the default language is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub language: Option<ClientLanguage>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct RequestEmailLoginTokenResult {
    /// Client token to be used together with the email token.
    client_token: Option<EmailLoginToken>,
    /// Token validity duration in seconds
    token_validity_seconds: Option<i64>,
    /// Minimum wait duration between token requests in seconds
    resend_wait_seconds: Option<i64>,
    /// Maximum number of email login tokens that can be sent per month.
    email_login_emails_per_month: Option<i64>,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error: bool,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_email_registration_ip_address_limit_reached: bool,

    /// This is true when the daily email registration limit has been
    /// reached. The client should guide the user to wait 24 hours or use
    /// another login method for account registration.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_email_registration_limit_reached: bool,

    /// This is true when email registration has been disabled for the
    /// client platform the user is using.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_email_registration_platform_disabled: bool,

    /// This is true when email registration has been disabled for all
    /// client platforms.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_email_registration_all_platforms_disabled: bool,

    /// This is true when the email domain is not accepted for email
    /// registration. This can happen when the domain is in the blocklist
    /// or not in the allowlist configured by the server admin.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_email_registration_domain_not_accepted: bool,
}

impl RequestEmailLoginTokenResult {
    pub fn successful(
        client_token: EmailLoginToken,
        token_validity_seconds: i64,
        resend_wait_seconds: i64,
        email_login_emails_per_month: i64,
    ) -> Self {
        Self {
            client_token: Some(client_token),
            token_validity_seconds: Some(token_validity_seconds),
            resend_wait_seconds: Some(resend_wait_seconds),
            email_login_emails_per_month: Some(email_login_emails_per_month),
            ..Default::default()
        }
    }

    pub fn error_email_registration_ip_address_limit_reached() -> Self {
        Self {
            error: true,
            error_email_registration_ip_address_limit_reached: true,
            ..Default::default()
        }
    }

    pub fn error_email_registration_limit_reached() -> Self {
        Self {
            error: true,
            error_email_registration_limit_reached: true,
            ..Default::default()
        }
    }

    pub fn error_email_registration_platform_disabled() -> Self {
        Self {
            error: true,
            error_email_registration_platform_disabled: true,
            ..Default::default()
        }
    }

    pub fn error_email_registration_all_platforms_disabled() -> Self {
        Self {
            error: true,
            error_email_registration_all_platforms_disabled: true,
            ..Default::default()
        }
    }

    pub fn error_email_registration_domain_not_accepted() -> Self {
        Self {
            error: true,
            error_email_registration_domain_not_accepted: true,
            ..Default::default()
        }
    }
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
    pub email_login_enabled: bool,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = crate::schema::account_email_change)]
#[diesel(check_for_backend(crate::Db))]
pub struct EmailChange {
    pub email_change: EmailAddress,
    pub email_change_unix_time: UnixTime,
    pub email_change_verification_token: Vec<u8>,
    pub email_change_verified: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmailLoginTokens {
    pub client_token: Option<Vec<u8>>,
    pub email_token: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct EmailAddressState {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub email: Option<EmailAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub email_change: Option<EmailAddress>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub email_change_verified: bool,
    /// API route handler sets this value
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub email_change_completion_time: Option<UnixTime>,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    #[schema(default = true)]
    pub email_login_enabled: bool,
}

impl EmailAddressState {
    pub fn new(internal: EmailAddressStateInternal, email_change: Option<EmailChange>) -> Self {
        Self {
            email: internal.email,
            email_change: email_change.as_ref().map(|v| v.email_change.clone()),
            email_change_verified: email_change
                .map(|v| v.email_change_verified)
                .unwrap_or(false),
            email_change_completion_time: None,
            email_login_enabled: internal.email_login_enabled,
        }
    }
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, IntoParams)]
pub struct BooleanSetting {
    pub value: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetAccountDeletionRequestResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub automatic_deletion_allowed: Option<UnixTime>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetAccountBanTimeResult {
    /// If `None` the account is not banned.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub banned_until: Option<UnixTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub admin_type: Option<AccountBannedAdminType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub reason_category: Option<AccountBanReasonCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub reason_details: Option<AccountBanReasonDetails>,
}

#[derive(Deserialize, ToSchema)]
pub struct SignInWithLoginInfo {
    pub client_info: ClientInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub apple: Option<SignInWithAppleInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub google: Option<SignInWithGoogleInfo>,
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

#[derive(Deserialize, ToSchema)]
pub struct PutSignInWithApple {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub apple: Option<SignInWithAppleInfo>,
}

#[derive(Deserialize, ToSchema)]
pub struct PutSignInWithGoogle {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub google: Option<SignInWithGoogleInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct SignInWithState {
    pub apple: bool,
    pub google: bool,
}

/// Result of a sign in with Apple or Google association change.
#[derive(Debug, Default, Clone, Deserialize, Serialize, ToSchema)]
pub struct PutSignInWithResult {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    error_history_limit_reached: bool,
}

impl PutSignInWithResult {
    pub fn ok() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn error_history_limit_reached() -> Self {
        Self {
            error: true,
            error_history_limit_reached: true,
        }
    }
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
    account_banned_admin_type_number: Option<AccountBannedAdminType>,
    account_banned_until_unix_time: Option<UnixTime>,
    account_banned_state_change_unix_time: Option<UnixTime>,
    news_sync_version: NewsSyncVersion,
    unread_news_count: i64,
    account_created_unix_time: UnixTime,
    account_locked: bool,
    account_verification_method: Option<VerificationMethod>,
    account_verification_unix_time: Option<UnixTime>,
    account_verification_error_flags: AccountVerificationErrorFlagsValue,
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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct RemoteBotPassword {
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct BotAccount {
    pub aid: AccountId,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetBotsResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub admin: Option<BotAccount>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<BotAccount>,
}
