use diesel::{prelude::*, Associations};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_try_from, diesel_string_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::{
    schema_sqlite_types::{Integer, Text}, AccessToken, AccountId, AccountIdDb, AccountIdInternal, AccountStateRelatedSharedState, AccountSyncVersion, RefreshToken
};

mod demo;
pub use demo::*;

// TODO(prod): Also add info what sign in with service is used?

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, PartialEq)]
pub struct LoginResult {
    pub account: AuthPair,

    /// If None profile microservice is disabled.
    pub profile: Option<AuthPair>,

    /// If None media microservice is disabled.
    pub media: Option<AuthPair>,

    /// Account ID of current account.
    pub account_id: AccountId,

    /// Current email of current account.
    pub email: Option<EmailAddress>,
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
    capabilities: Capabilities,
    visibility: ProfileVisibility,
    sync_version: AccountSyncVersion,
}

impl Account {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_from_internal_types(
        capabilities: Capabilities,
        shared_state: AccountStateRelatedSharedState,
    ) -> Self {
        Self {
            state: shared_state.account_state_number,
            capabilities,
            visibility: shared_state.profile_visibility(),
            sync_version: shared_state.sync_version,
        }
    }

    pub fn new_from(
        capabilities: Capabilities,
        state: AccountState,
        visibility: ProfileVisibility,
        sync_version: AccountSyncVersion,
    ) -> Self {
        Self {
            capabilities,
            state,
            visibility,
            sync_version,
        }
    }

    pub fn state(&self) -> AccountState {
        self.state
    }

    pub fn capablities(&self) -> Capabilities {
        self.capabilities.clone()
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

macro_rules! define_capablities {
    ($( $( #[doc = $text:literal] )* $name:ident , )* ) => {

        #[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default, PartialEq, Eq, Queryable, Selectable, Insertable, AsChangeset)]
        #[diesel(table_name = crate::schema::account_capabilities)]
        #[diesel(check_for_backend(crate::Db))]
        pub struct Capabilities {
            $(
                $(#[doc = $text])?
                #[serde(default, skip_serializing_if = "std::ops::Not::not")] // Skips false
                #[schema(default = false)]
                pub $name: bool,
            )*
        }

        impl Capabilities {
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

define_capablities!(
    admin_modify_capabilities,
    admin_moderate_profiles,
    admin_moderate_images,
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
);

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
    AsChangeset,
)]
#[diesel(table_name = crate::schema::account_setup)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccountSetup {
    birthdate: String,
}

impl AccountSetup {
    pub fn is_invalid(&self) -> bool {
        self.birthdate.is_empty()
    }
}

// TODO(prod): Birthdate validation

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

pub const ACCOUNT_GLOBAL_STATE_ROW_TYPE: i64 = 0;

/// Global state for account component
#[derive(Debug, Default, Clone, PartialEq, Queryable, Selectable)]
#[diesel(table_name = crate::schema::account_global_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccountGlobalState {
    pub admin_access_granted_count: i64,
}
