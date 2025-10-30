use chrono::NaiveDate;
use diesel::prelude::*;
use model::{ClientType, ClientVersion, NewsSyncVersion, UnixTime};
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

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, PartialEq)]
pub struct LoginResult {
    /// If `None`, the client is unsupported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<AuthPair>,

    /// Account ID of current account. If `None`, the client is unsupported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aid: Option<AccountId>,

    /// Current email of current account. If `None`, if email address is not
    /// set or the client version is unsupported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<EmailAddress>,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_unsupported_client: bool,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_sign_in_with_email_unverified: bool,
}

impl LoginResult {
    pub fn error_unsupported_client() -> Self {
        Self {
            tokens: None,
            aid: None,
            email: None,
            error_unsupported_client: true,
            error_sign_in_with_email_unverified: false,
        }
    }

    pub fn error_sign_in_with_email_unverified() -> Self {
        Self {
            tokens: None,
            aid: None,
            email: None,
            error_unsupported_client: false,
            error_sign_in_with_email_unverified: true,
        }
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

#[derive(Debug, Clone, Default, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::account)]
#[diesel(check_for_backend(crate::Db))]
#[diesel(treat_none_as_null = true)]
pub struct AccountInternal {
    pub email: Option<EmailAddress>,
    pub email_verification_token: Option<Vec<u8>>,
    pub email_verification_token_unix_time: Option<UnixTime>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct AccountData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<EmailAddress>,
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
    birthdate: Option<NaiveDate>,
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
    next_client_id: i64,
    account_deletion_request_unix_time: Option<UnixTime>,
    account_banned_reason_category: Option<i16>,
    account_banned_reason_details: Option<AccountBanReasonDetails>,
    account_banned_until_unix_time: Option<UnixTime>,
    account_banned_state_change_unix_time: Option<UnixTime>,
    news_sync_version: NewsSyncVersion,
    unread_news_count: i64,
    account_created_unix_time: UnixTime,
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
