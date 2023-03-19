use serde::{de::Error, Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema, openapi::Ref};

/// AccountId is an UUID string. Server will generate an UUID string when
/// generating a new AccountId.
#[derive(Debug, ToSchema, Clone, Eq, Hash, PartialEq, IntoParams)]
pub struct AccountId {
    // String representation is used a lot in server code, so
    // it is better than using only Uuid type directly.
    /// UUID string with Simple format.
    account_id: String,
    light: AccountIdLight,
}

impl AccountId {
    pub fn generate_new() -> Self {
        let id = uuid::Uuid::new_v4();
        Self {
            account_id: id.hyphenated().to_string(),
            light: AccountIdLight { account_id: id },
        }
    }

    pub fn parse(account_id: String) -> Result<Self, uuid::Error> {
        match uuid::Uuid::try_parse(&account_id) {
            Ok(light) => Ok(Self {
                account_id: light.as_hyphenated().to_string(),
                light: AccountIdLight { account_id: light },
            }),
            Err(e) => Err(e),
        }
    }

    pub fn into_string(self) -> String {
        self.account_id
    }

    pub fn as_str(&self) -> &str {
        &self.account_id
    }

    pub fn as_light(&self) -> AccountIdLight {
        self.light
    }

    pub fn formatter(&self) -> uuid::fmt::Hyphenated {
        self.light.account_id.hyphenated()
    }
}

impl<'de> Deserialize<'de> for AccountId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        pub struct AccountIdRaw {
            account_id: String,
        }

        let raw = AccountIdRaw::deserialize(deserializer)?;

        AccountId::parse(raw.account_id).map_err(|_| D::Error::custom("Is not an UUID"))
    }
}

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
        AccountIdLight { account_id: self.account_id }
    }
}

impl From<AccountIdInternal> for uuid::Uuid {
    fn from(value: AccountIdInternal) -> Self {
        value.account_id
    }
}

/// AccoutId which is internally Uuid object.
/// Consumes less memory.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Eq, Hash, PartialEq, IntoParams, Copy)]
pub struct AccountIdLight {
    pub account_id: uuid::Uuid,
}

impl AccountIdLight {
    pub fn to_full(&self) -> AccountId {
        AccountId {
            account_id: self.account_id.hyphenated().to_string(),
            light: self.clone(),
        }
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

/// This is just a really long random string.
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct RefreshToken {
    token: String,
}

impl RefreshToken {
    pub fn generate_new() -> Self {
        let mut token = String::new();

        for _ in 1..=124 {
            token.push_str(uuid::Uuid::new_v4().simple().to_string().as_str())
        }

        Self {
            token
        }
    }

    pub fn from_string(token: String) -> Self {
        Self { token }
    }

    pub fn into_string(self) -> String {
        self.token
    }

    pub fn as_str(&self) -> &str {
        &self.token
    }
}

/// ApiKey and RefreshToken
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct AuthPair {
    refresh: RefreshToken,
    key: ApiKey,
}

impl AuthPair {
    pub fn new(refresh: RefreshToken, key: ApiKey) -> Self {
        Self {
            refresh,
            key,
        }
    }
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

    pub fn state(&self) -> AccountState {
        self.state
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

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default, PartialEq, Eq)]
pub struct Capabilities {
    admin_modify_capablities: Option<bool>,
    admin_setup_possible: Option<bool>,
    admin_moderate_profiles: Option<bool>,
    admin_moderate_images: Option<bool>,
    admin_view_private_info: Option<bool>,
    admin_view_profile_history: Option<bool>,
    admin_ban_profile: Option<bool>,
    banned_edit_profile: Option<bool>,
    view_public_profiles: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Default, PartialEq, Eq)]
pub struct AccountSetup {
    name: String,
}
