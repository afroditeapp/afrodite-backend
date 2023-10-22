use diesel::{prelude::*, Associations};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    macros::diesel_string_wrapper, AccessToken, AccountIdDb, AccountIdInternal, RefreshToken,
};

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct LoginResult {
    pub account: AuthPair,

    /// If None profile microservice is disabled.
    pub profile: Option<AuthPair>,

    /// If None media microservice is disabled.
    pub media: Option<AuthPair>,
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
    pub email: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct AccountData {
    pub email: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct Account {
    state: AccountState,
    capablities: Capabilities,
}

impl Account {
    pub fn new() -> Self {
        Self {
            state: AccountState::InitialSetup,
            capablities: Default::default(),
        }
    }

    pub fn new_from(state: AccountState, capablities: Capabilities) -> Self {
        Self { state, capablities }
    }

    pub fn state(&self) -> AccountState {
        self.state
    }

    pub fn capablities(&self) -> &Capabilities {
        &self.capablities
    }

    pub fn into_capablities(self) -> Capabilities {
        self.capablities
    }

    pub fn capablities_mut(&mut self) -> &mut Capabilities {
        &mut self.capablities
    }

    pub fn state_mut(&mut self) -> &mut AccountState {
        &mut self.state
    }

    pub fn complete_setup(&mut self) {
        if self.state == AccountState::InitialSetup {
            self.state = AccountState::Normal;
        }
    }

    pub fn add_admin_capablities(&mut self) {
        self.capablities.admin_moderate_images = true;
        self.capablities.admin_server_maintenance_view_info = true;
        self.capablities.admin_server_maintenance_view_backend_config = true;
        self.capablities.admin_server_maintenance_save_backend_config = true;
        self.capablities.admin_server_maintenance_update_software = true;
        self.capablities.admin_server_maintenance_reset_data = true;
        self.capablities.admin_server_maintenance_reboot_backend = true;
        // TOOD: Other capablities as well?
    }
}

impl Default for Account {
    fn default() -> Self {
        Self {
            state: AccountState::InitialSetup,
            capablities: Capabilities::default(),
        }
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


#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
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
    /// View public profiles. Automatically enabled once initial
    /// image moderation is complete.
    user_view_public_profiles,
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
    name: String,
    birthdate: String,
}

impl AccountSetup {
    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }

    pub fn name(&self) -> &str {
        &self.name
    }
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

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type, PartialEq)]
#[serde(transparent)]
#[sqlx(transparent)]
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
