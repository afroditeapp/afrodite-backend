use diesel::{deserialize::FromSqlRow, expression::AsExpression, prelude::*, sql_types::BigInt};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::{
    schema_sqlite_types::Integer, AccountId, AccountStateRelatedSharedState, AccountSyncVersion,
    ProfileAge,
};

mod news;
pub use news::*;

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
    admin_moderate_profile_content,
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

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Deserialize)]
pub enum EmailMessages {
    AccountRegistered,
}

impl EmailMessages {
    pub const VARIANTS: &'static [EmailMessages] = &[EmailMessages::AccountRegistered];
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct AccessibleAccount {
    pub aid: AccountId,
    pub name: Option<String>,
    #[schema(value_type = Option<i64>)]
    pub age: Option<ProfileAge>,
}
