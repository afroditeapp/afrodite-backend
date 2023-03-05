use serde::{Deserialize, Serialize, de::Error};
use utoipa::{IntoParams, ToSchema};

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
            account_id: id.simple().to_string(),
            light: AccountIdLight { account_id: id }
        }
    }

    pub fn parse(account_id: String) -> Result<Self, uuid::Error> {
        match uuid::Uuid::try_parse(&account_id) {
            Ok(light) => Ok(Self {
                account_id: light.as_simple().to_string(),
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

    pub fn formatter(&self) -> uuid::fmt::Simple {
        self.light.account_id.simple()
    }
}

impl <'de> Deserialize<'de> for AccountId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {


        #[derive(Deserialize)]
        pub struct AccountIdRaw {
            account_id: String,
        }

        let raw = AccountIdRaw::deserialize(deserializer)?;

        AccountId::parse(raw.account_id)
            .map_err(|_| D::Error::custom("Is not an UUID"))
    }
}

/// AccoutId which is internally Uuid object.
/// Consumes less memory.
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Eq, Hash, PartialEq, IntoParams, Copy)]
pub struct AccountIdLight {
    account_id: uuid::Uuid,
}

impl AccountIdLight {
    pub fn to_full(&self) -> AccountId {
        AccountId { account_id: self.account_id.simple().to_string(), light: self.clone() }
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

#[derive(Debug, Deserialize, Serialize, ToSchema)]
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
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub enum AccountState {
    InitialSetup,
    Normal,
    Banned,
    PendingDeletion,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Default)]
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
