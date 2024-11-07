use chrono::NaiveDate;
use diesel::{deserialize::FromSqlRow, expression::AsExpression, prelude::*, sql_types::BigInt, Associations};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper, diesel_string_wrapper};
use utils::time::age_in_years_from_birthdate;
use utoipa::{IntoParams, ToSchema};

use crate::{
    schema_sqlite_types::{Integer, Text}, AccessToken, AccountId, AccountIdDb, AccountIdInternal, AccountStateRelatedSharedState, AccountSyncVersion, PublicKeyIdAndVersion, RefreshToken
};

mod demo;
pub use demo::*;

mod email;
pub use email::*;

mod news;
pub use news::*;

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

    /// Info about latest public keys. Client can use this value to
    /// ask if user wants to copy existing private and public key from
    /// other device. If empty, public key is not set or the client
    /// is unsupported.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[schema(default = json!([]))]
    pub latest_public_keys: Vec<PublicKeyIdAndVersion>,

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
            latest_public_keys: vec![],
            error_unsupported_client: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, ToSchema)]
pub enum ClientType {
    Android,
    Ios,
    Web,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct ClientInfo {
    pub client_type: ClientType,
    pub major_version: u16,
    pub minor_version: u16,
    pub patch_version: u16,
}

impl ClientInfo {
    pub fn is_unsupported_client(&self) -> bool {
        false
    }
}

/// AccessToken and RefreshToken
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct AuthPair {
    pub refresh: RefreshToken,
    pub access: AccessToken,
}

impl AuthPair {
    pub fn new(refresh: RefreshToken, access: AccessToken) -> Self {
        Self { refresh, access }
    }
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

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct Account {
    state: AccountState,
    permissions: Permissions,
    visibility: ProfileVisibility,
    sync_version: AccountSyncVersion,
}

impl Account {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_from_internal_types(
        permissions: Permissions,
        shared_state: AccountStateRelatedSharedState,
    ) -> Self {
        Self {
            state: shared_state.account_state_number,
            permissions,
            visibility: shared_state.profile_visibility(),
            sync_version: shared_state.sync_version,
        }
    }

    pub fn new_from(
        permissions: Permissions,
        state: AccountState,
        visibility: ProfileVisibility,
        sync_version: AccountSyncVersion,
    ) -> Self {
        Self {
            permissions,
            state,
            visibility,
            sync_version,
        }
    }

    pub fn state(&self) -> AccountState {
        self.state
    }

    pub fn permissions(&self) -> Permissions {
        self.permissions.clone()
    }

    pub fn profile_visibility(&self) -> ProfileVisibility {
        self.visibility
    }

    pub fn sync_version(&self) -> AccountSyncVersion {
        self.sync_version
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AccountStateError {
    WrongStateNumber(i64),
}
impl AccountStateError {
    pub fn wrong_state_number(number: i64) -> Self {
        Self::WrongStateNumber(number)
    }
}
impl std::fmt::Display for AccountStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountStateError::WrongStateNumber(number) => {
                write!(f, "Wrong state number: {}", number)
            }
        }
    }
}
impl std::error::Error for AccountStateError {}

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Eq,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Integer)]
pub enum AccountState {
    InitialSetup = 0,
    Normal = 1,
    Banned = 2,
    PendingDeletion = 3,
}

impl TryFrom<i64> for AccountState {
    type Error = AccountStateError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::InitialSetup),
            1 => Ok(Self::Normal),
            2 => Ok(Self::Banned),
            3 => Ok(Self::PendingDeletion),
            _ => Err(AccountStateError::WrongStateNumber(value)),
        }
    }
}

diesel_i64_try_from!(AccountState);

impl Default for AccountState {
    fn default() -> Self {
        Self::InitialSetup
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Eq,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Integer)]
pub enum ProfileVisibility {
    /// Profile is currently private and its visibility is not
    /// changed when initial moderation request will be moderated as accepted.
    PendingPrivate = 0,
    /// Profile is currently private and its visibility will
    /// change to public when initial moderation request will be moderated as
    /// accepted.
    PendingPublic = 1,
    /// Profile is currently private.
    Private = 2,
    /// Profile is currently public.
    Public = 3,
}

impl Default for ProfileVisibility {
    fn default() -> Self {
        Self::PendingPrivate
    }
}

impl ProfileVisibility {
    pub fn is_currently_public(&self) -> bool {
        *self == Self::Public
    }

    pub fn is_pending(&self) -> bool {
        *self == Self::PendingPrivate || *self == Self::PendingPublic
    }
}

impl TryFrom<i64> for ProfileVisibility {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::PendingPrivate),
            1 => Ok(Self::PendingPublic),
            2 => Ok(Self::Private),
            3 => Ok(Self::Public),
            _ => Err(format!("Unknown visibility number: {}", value)),
        }
    }
}

diesel_i64_try_from!(ProfileVisibility);

macro_rules! define_permissions {
    (struct $struct_name:ident, $( $( #[doc = $text:literal] )* $name:ident , )* ) => {

        #[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default, PartialEq, Eq, Queryable, Selectable, Insertable, AsChangeset)]
        #[diesel(table_name = crate::schema::account_permissions)]
        #[diesel(check_for_backend(crate::Db))]
        pub struct $struct_name {
            $(
                $(#[doc = $text])?
                #[serde(default, skip_serializing_if = "std::ops::Not::not")] // Skips false
                #[schema(default = false)]
                pub $name: bool,
            )*
        }

        impl $struct_name {
            pub fn all_enabled() -> Self {
                Self {
                    $(
                        $name: true,
                    )*
                }
            }
        }

    };
}

define_permissions!(
    struct Permissions,
    admin_modify_permissions,
    admin_moderate_profiles,
    admin_moderate_images,
    admin_moderate_profile_names,
    admin_moderate_profile_texts,
    /// View public and private profiles.
    admin_view_all_profiles,
    admin_view_private_info,
    admin_view_profile_history,
    /// View server infrastructure related info like logs and
    /// software versions.
    admin_server_maintenance_view_info,
    admin_server_maintenance_view_backend_config,
    admin_server_maintenance_update_software,
    admin_server_maintenance_reset_data,
    admin_server_maintenance_reboot_backend,
    admin_server_maintenance_save_backend_config,
    admin_news_create,
    admin_news_edit_all,
    admin_profile_statistics,
);

impl Permissions {
    pub fn some_admin_news_permissions_granted(&self) -> bool {
        self.admin_news_create || self.admin_news_edit_all
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
)]
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

#[derive(
    Debug,
    Clone,
    Deserialize,
    Serialize,
    ToSchema,
    Default,
    PartialEq,
    Eq,
)]
pub struct LatestBirthdate {
    #[schema(value_type = Option<String>)]
    pub birthdate: Option<NaiveDate>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, IntoParams)]
pub struct BooleanSetting {
    pub value: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct DeleteStatus {
    delete_date: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct SignInWithLoginInfo {
    pub client_info: ClientInfo,
    pub apple_token: Option<String>,
    pub google_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Queryable, Selectable, Associations)]
#[diesel(belongs_to(AccountIdInternal, foreign_key = account_id))]
#[diesel(table_name = crate::schema::sign_in_with_info)]
#[diesel(check_for_backend(crate::Db))]
pub struct SignInWithInfoRaw {
    pub account_id: AccountIdDb,
    pub google_account_id: Option<GoogleAccountId>,
    pub is_bot_account: bool,
}

impl From<SignInWithInfoRaw> for SignInWithInfo {
    fn from(raw: SignInWithInfoRaw) -> Self {
        Self {
            google_account_id: raw.google_account_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SignInWithInfo {
    pub google_account_id: Option<GoogleAccountId>,
}

impl SignInWithInfo {
    pub fn google_account_id_matches_with(&self, id: &GoogleAccountId) -> bool {
        if let Some(google_account_id) = &self.google_account_id {
            google_account_id == id
        } else {
            false
        }
    }

    pub fn some_sign_in_with_method_is_set(&self) -> bool {
        self.google_account_id.is_some()
    }
}

#[derive(
    Debug, Serialize, Deserialize, Clone, PartialEq, diesel::FromSqlRow, diesel::AsExpression,
)]
#[diesel(sql_type = Text)]
#[serde(transparent)]
pub struct GoogleAccountId(pub String);

impl GoogleAccountId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

diesel_string_wrapper!(GoogleAccountId);

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    diesel::FromSqlRow,
    diesel::AsExpression,
    ToSchema,
)]
#[diesel(sql_type = Text)]
#[serde(try_from = "String")]
pub struct EmailAddress(pub String);

impl EmailAddress {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

diesel_string_wrapper!(EmailAddress);

impl TryFrom<String> for EmailAddress {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.trim() != value {
            return Err("Email address contains leading or trailing whitespace".to_string());
        }

        if value.contains('@') {
            Ok(Self(value))
        } else {
            Err("Email address does not have '@' character".to_string())
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

/// ID which client receives from server once.
/// Next value is incremented compared to previous value, so
/// in practice the ID can be used as unique ID even if it
/// can wrap.
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
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct ClientId {
    pub id: i64,
}

impl ClientId {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.id
    }

    pub fn increment(&self) -> Self {
        Self {
            id: self.id.wrapping_add(1),
        }
    }
}

diesel_i64_wrapper!(ClientId);

/// ID which client owns. This should be unique when
/// considering one client instance.
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
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct ClientLocalId {
    pub id: i64,
}

impl ClientLocalId {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.id
    }
}

diesel_i64_wrapper!(ClientLocalId);
