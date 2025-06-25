use chrono::NaiveDate;
use diesel::{Associations, prelude::*};
use model::{ClientType, ClientVersion, UnixTime};
use model_server_data::{
    AppleAccountId, AuthPair, EmailAddress, GoogleAccountId, PublicationId, SignInWithInfo,
};
use model_server_state::DemoModeToken;
use serde::{Deserialize, Serialize};
use utils::time::age_in_years_from_birthdate;
use utoipa::{IntoParams, ToSchema};

use crate::{AccountId, AccountIdDb, AccountIdInternal};

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

// TODO(prod): Also add info what sign in with service is used?

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, PartialEq)]
pub struct LoginResult {
    /// If `None`, the client is unsupported.
    pub account: Option<AuthPair>,

    /// If `None`, profile microservice is disabled or the version client is
    /// unsupported.
    pub profile: Option<AuthPair>,

    /// If `None`, media microservice is disabled or the client version is
    /// unsupported.
    pub media: Option<AuthPair>,

    /// Account ID of current account. If `None`, the client is unsupported.
    pub aid: Option<AccountId>,

    /// Current email of current account. If `None`, if email address is not
    /// set or the client version is unsupported.
    pub email: Option<EmailAddress>,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub error_unsupported_client: bool,
}

impl LoginResult {
    pub fn error_unsupported_client() -> Self {
        Self {
            account: None,
            profile: None,
            media: None,
            aid: None,
            email: None,
            error_unsupported_client: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct ClientInfo {
    pub client_type: ClientType,
    pub client_version: ClientVersion,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct DemoModeLoginToAccount {
    pub token: DemoModeToken,
    pub aid: AccountId,
    pub client_info: ClientInfo,
}

#[derive(Debug, Clone, Default, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::schema::account)]
#[diesel(check_for_backend(crate::Db))]
#[diesel(treat_none_as_null = true)]
pub struct AccountInternal {
    pub email: Option<EmailAddress>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct AccountData {
    pub email: Option<EmailAddress>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default, PartialEq, Eq)]
pub struct SetAccountSetup {
    /// String date with "YYYY-MM-DD" format.
    ///
    /// This is not required at the moment to reduce sensitive user data.
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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default, PartialEq, Eq)]
pub struct LatestBirthdate {
    #[schema(value_type = Option<String>)]
    pub birthdate: Option<NaiveDate>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, IntoParams)]
pub struct BooleanSetting {
    pub value: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetAccountDeletionRequestResult {
    pub automatic_deletion_allowed: Option<UnixTime>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetAccountBanTimeResult {
    /// If `None` the account is not banned.
    pub banned_until: Option<UnixTime>,
    pub reason_category: Option<AccountBanReasonCategory>,
    pub reason_details: Option<AccountBanReasonDetails>,
}

// TODO(prod): Add nonce support to Sign in with Google?

#[derive(Deserialize, ToSchema)]
pub struct SignInWithLoginInfo {
    pub client_info: ClientInfo,
    pub apple: Option<SignInWithAppleInfo>,
    pub google_token: Option<String>,
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

// TODO(prod): Remove unused belongs_to?

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Associations)]
#[diesel(belongs_to(AccountIdInternal, foreign_key = account_id))]
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
    pub const ACCOUNT_GLOBAL_STATE_ROW_TYPE: i64 = 0;
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct RemoteBotLogin {
    pub aid: AccountId,
    pub password: String,
}
