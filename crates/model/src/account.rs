use diesel::{
    deserialize::FromSqlRow,
    expression::AsExpression,
    prelude::*,
    sql_types::{Binary, SmallInt},
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{NonEmptyString, SimpleDieselEnum, diesel_uuid_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::{AccountId, AccountStateRelatedSharedState, AccountSyncVersion, ProfileAge};

mod news;
pub use news::*;

mod time;
pub use time::*;

mod custom_reports;
pub use custom_reports::*;

mod client_features;
pub use client_features::*;

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct Account {
    state: AccountStateContainer,
    permissions: Permissions,
    visibility: ProfileVisibility,
    #[serde(default, skip_serializing_if = "value_is_true")]
    #[schema(default = true)]
    email_verified: bool,
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
            email_verified: shared_state.email_verified,
            sync_version: shared_state.sync_version,
        }
    }

    pub fn new_from(
        permissions: Permissions,
        state: AccountStateContainer,
        visibility: ProfileVisibility,
        email_verified: bool,
        sync_version: AccountSyncVersion,
    ) -> Self {
        Self {
            permissions,
            state,
            visibility,
            email_verified,
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

    pub fn email_verified(&self) -> bool {
        self.email_verified
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccountState {
    InitialSetup,
    Normal,
    Banned,
    PendingDeletion,
}

#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
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
    admin_change_email_address,
    admin_edit_login,
    admin_edit_permissions,
    admin_edit_profile_name,
    admin_edit_max_public_key_count,
    admin_edit_media_content_face_detected_value,
    admin_export_data,
    admin_moderate_media_content,
    admin_moderate_profile_names,
    admin_moderate_profile_texts,
    admin_process_reports,
    admin_delete_media_content,
    admin_delete_account,
    admin_ban_account,
    admin_request_account_deletion,
    /// View public and private profiles.
    admin_view_all_profiles,
    admin_view_account_state,
    admin_view_account_api_usage,
    admin_view_account_ip_address_usage,
    admin_view_profile_history,
    admin_view_permissions,
    admin_view_email_address,
    admin_find_account_by_email_address,
    /// View server infrastructure related info like logs and
    /// software versions.
    admin_server_maintenance_view_info,
    admin_server_maintenance_view_backend_config,
    admin_server_maintenance_update_software,
    admin_server_maintenance_reset_data,
    admin_server_maintenance_restart_backend,
    admin_server_maintenance_save_backend_config,
    admin_server_maintenance_edit_notification,
    admin_news_create,
    admin_news_edit_all,
    admin_profile_statistics,
    admin_subscribe_admin_notifications,
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
    SimpleDieselEnum,
    diesel::FromSqlRow,
    diesel::AsExpression,
    num_enum::TryFromPrimitive,
)]
#[diesel(sql_type = SmallInt)]
#[repr(i16)]
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
            Self::Public | Self::Private => *self = Self::Private,
            Self::PendingPublic | Self::PendingPrivate => *self = Self::PendingPrivate,
        };
    }
}

/// UUID which client owns
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
pub struct ClientLocalId {
    id: simple_backend_utils::UuidBase64Url,
}

impl ClientLocalId {
    pub fn new(id: simple_backend_utils::UuidBase64Url) -> Self {
        Self { id }
    }

    pub fn id(&self) -> simple_backend_utils::UuidBase64Url {
        self.id
    }
}

impl TryFrom<simple_backend_utils::UuidBase64Url> for ClientLocalId {
    type Error = String;

    fn try_from(id: simple_backend_utils::UuidBase64Url) -> Result<Self, Self::Error> {
        Ok(Self { id })
    }
}

impl AsRef<simple_backend_utils::UuidBase64Url> for ClientLocalId {
    fn as_ref(&self) -> &simple_backend_utils::UuidBase64Url {
        &self.id
    }
}

diesel_uuid_wrapper!(ClientLocalId);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Deserialize)]
pub enum EmailMessages {
    /// Email sending code must generate and save the verification token to DB
    EmailVerification,
    NewMessage,
    NewLike,
    AccountDeletionRemainderFirst,
    AccountDeletionRemainderSecond,
    AccountDeletionRemainderThird,
    /// Email sending code must read the verification token from DB
    EmailChangeVerification,
    EmailChangeNotification,
    /// Email sending code must read the login token from DB
    EmailLoginToken,
}

impl EmailMessages {
    pub const VARIANTS: &'static [Self] = &[
        Self::EmailVerification,
        Self::NewMessage,
        Self::NewLike,
        Self::AccountDeletionRemainderFirst,
        Self::AccountDeletionRemainderSecond,
        Self::AccountDeletionRemainderThird,
        Self::EmailChangeVerification,
        Self::EmailChangeNotification,
        Self::EmailLoginToken,
    ];
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Clone)]
pub struct AccessibleAccount {
    pub aid: AccountId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<NonEmptyString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<i64>)]
    pub age: Option<ProfileAge>,
}
