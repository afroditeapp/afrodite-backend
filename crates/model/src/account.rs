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

mod account_created_time;
pub use account_created_time::*;

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct Account {
    state: AccountStateContainer,
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
            state: AccountStateContainer {
                initial_setup_completed: shared_state.account_state_initial_setup_completed,
                banned: shared_state.account_state_banned,
                pending_deletion: shared_state.account_state_pending_deletion,
            },
            permissions,
            visibility: shared_state.profile_visibility(),
            sync_version: shared_state.sync_version,
        }
    }

    pub fn new_from(
        permissions: Permissions,
        state: AccountStateContainer,
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

    pub fn state_container(&self) -> AccountStateContainer {
        self.state
    }

    pub fn state(&self) -> AccountState {
        self.state.account_state()
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccountState {
    InitialSetup,
    Normal,
    Banned,
    PendingDeletion,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Eq,
)]
pub struct AccountStateContainer {
    #[serde(default, skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    pub(crate) initial_setup_completed: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub(crate) banned: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[schema(default = false)]
    pub(crate) pending_deletion: bool,
}

fn value_is_true(v: &bool) -> bool {
    *v
}

impl AccountStateContainer {
    pub fn account_state(&self) -> AccountState {
        if self.pending_deletion {
            AccountState::PendingDeletion
        } else if self.banned {
            AccountState::Banned
        } else if !self.initial_setup_completed {
            AccountState::InitialSetup
        } else {
            AccountState::Normal
        }
    }

    pub fn complete_initial_setup(&mut self) {
        self.initial_setup_completed = true;
    }

    pub fn set_pending_deletion(&mut self, value: bool) {
        self.pending_deletion = value;
    }

    pub fn set_banned(&mut self, value: bool) {
        self.banned = value;
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
    admin_delete_media_content,
    admin_delete_account,
    admin_ban_account,
    admin_request_account_deletion,
    /// View public and private profiles.
    admin_view_all_profiles,
    admin_view_private_info,
    admin_view_profile_history,
    admin_find_account_by_email,
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

    pub fn change_to_private_or_pending_private(&mut self) {
        match *self {
            Self::Public |
            Self::Private => *self = Self::Private,
            Self::PendingPublic |
            Self::PendingPrivate => *self = Self::PendingPrivate,
        };
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
