use serde::{Deserialize, Serialize, de::Error};
use utoipa::{IntoParams, ToSchema};

/// AccountId is an UUID string. Server will generate an UUID string when
/// generating a new AccountId.
#[derive(Debug, Serialize, ToSchema, Clone, Eq, Hash, PartialEq, IntoParams)]
pub struct AccountId {
    // String representation is used a lot in server code, so
    // it is better than using Uuid type directly.
    account_id: String,
}

impl AccountId {
    pub fn generate_new() -> Self {
        Self {
            account_id: uuid::Uuid::new_v4().simple().to_string(),
        }
    }

    pub fn parse(account_id: String) -> Result<Self, uuid::Error> {
        match uuid::Uuid::try_parse(&account_id) {
            Ok(_) => Ok(Self { account_id }),
            Err(e) => Err(e),
        }
    }

    pub fn into_string(self) -> String {
        self.account_id
    }

    pub fn as_str(&self) -> &str {
        &self.account_id
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
