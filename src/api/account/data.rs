use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

/// Used with database
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Eq, Hash, PartialEq, Copy)]
pub struct AccountIdInternal {
    pub account_id: uuid::Uuid,
    pub account_row_id: i64,
}

impl AccountIdInternal {
    pub fn as_uuid(&self) -> uuid::Uuid {
        self.account_id
    }

    pub fn row_id(&self) -> i64 {
        self.account_row_id
    }

    pub fn as_light(&self) -> AccountIdLight {
        AccountIdLight {
            account_id: self.account_id,
        }
    }
}

impl From<AccountIdInternal> for uuid::Uuid {
    fn from(value: AccountIdInternal) -> Self {
        value.account_id
    }
}

/// AccountId which is internally Uuid object.
/// Consumes less memory.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Eq, Hash, PartialEq, IntoParams, Copy)]
pub struct AccountIdLight {
    pub account_id: uuid::Uuid,
}

impl std::fmt::Display for AccountIdLight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Test"))
    }
}

impl AccountIdLight {
    pub fn new(account_id: uuid::Uuid) -> Self {
        Self { account_id }
    }

    pub fn as_uuid(&self) -> uuid::Uuid {
        self.account_id
    }

    pub fn to_string(&self) -> String {
        self.account_id.hyphenated().to_string()
    }
}

impl From<AccountIdLight> for uuid::Uuid {
    fn from(value: AccountIdLight) -> Self {
        value.account_id
    }
}

/// This is just a random string.
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct ApiKey {
    /// API token which server generates.
    api_key: String,
}

impl ApiKey {
    pub fn generate_new() -> Self {
        Self {
            api_key: uuid::Uuid::new_v4().simple().to_string(),
        }
    }

    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub fn into_string(self) -> String {
        self.api_key
    }

    pub fn as_str(&self) -> &str {
        &self.api_key
    }
}

// /// This is just a really long random string.
// #[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
// pub struct RefreshToken {
//     token: String,
// }

// impl RefreshToken {
//     pub fn generate_new() -> Self {
//         let mut token = String::new();

//         for _ in 1..=124 {
//             token.push_str(uuid::Uuid::new_v4().simple().to_string().as_str())
//         }

//         Self {
//             token
//         }
//     }

//     pub fn from_string(token: String) -> Self {
//         Self { token }
//     }

//     pub fn into_string(self) -> String {
//         self.token
//     }

//     pub fn as_str(&self) -> &str {
//         &self.token
//     }
// }

// /// ApiKey and RefreshToken
// #[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
// pub struct AuthPair {
//     refresh: RefreshToken,
//     key: ApiKey,
// }

// impl AuthPair {
//     pub fn new(refresh: RefreshToken, key: ApiKey) -> Self {
//         Self {
//             refresh,
//             key,
//         }
//     }
// }

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

    pub fn complete_setup(&mut self) {
        if self.state == AccountState::InitialSetup {
            self.state = AccountState::Normal;
        }
    }

    pub fn add_admin_capablities(&mut self) {
        self.capablities.admin_moderate_images = true;
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub enum AccountState {
    InitialSetup,
    Normal,
    Banned,
    PendingDeletion,
}

macro_rules! define_capablities {
    ($( $(#[doc = $text:literal ])? $name:ident , )* ) => {

        #[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default, PartialEq, Eq)]
        pub struct Capabilities {
            $(
                $(#[doc = $text])?
                #[serde(default, skip_serializing_if = "std::ops::Not::not")] // Skips false
                pub $name: bool,
            )*
        }

    };
}

define_capablities!(
    admin_modify_capablities,
    admin_setup_possible,
    admin_moderate_profiles,
    admin_moderate_images,
    /// View public and private profiles.
    admin_view_all_profiles,
    admin_view_private_info,
    admin_view_profile_history,
    admin_ban_profile,
    banned_edit_profile,
    /// View public profiles
    view_public_profiles,
);

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default, PartialEq, Eq)]
pub struct AccountSetup {
    name: String,
    email: String,
}

impl AccountSetup {
    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }

    pub fn email(&self) -> &str {
        &self.email
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq, IntoParams)]
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
