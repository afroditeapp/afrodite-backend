use base64::Engine;
use diesel::{
    prelude::*,
    sql_types::{BigInt, Binary},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{macros::{diesel_i64_wrapper, diesel_uuid_wrapper}, AccountState, Capabilities};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct BackendVersion {
    /// Backend code version.
    pub backend_code_version: String,
    /// Semver version of the backend.
    pub backend_version: String,
    /// Semver version of the protocol used by the backend.
    pub protocol_version: String,
}

/// Identifier for event.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub enum EventType {
    /// New account state for client.
    /// Data: account_state
    AccountStateChanged,
    /// New capabilities for client.
    /// Data: capabilities
    AccountCapabilitiesChanged,
}

/// Event to client which is sent through websocket.
///
/// This is not an enum to make generated API bindings more easier to
/// use.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct EventToClient {
    event: EventType,
    /// Data for event AccountStateChanged
    account_state: Option<AccountState>,
    /// Data for event AccountCapabilitiesChanged
    capabilities: Option<Capabilities>,
}

pub enum EventToClientInternal {
    /// New account state for client
    AccountStateChanged {
        state: AccountState,
    },
    /// New capabilities for client
    AccountCapabilitiesChanged {
        capabilities: Capabilities,
    },
}

impl From<EventToClientInternal> for EventToClient {
    fn from(internal: EventToClientInternal) -> Self {
        let mut value = Self {
            event: EventType::AccountStateChanged,
            account_state: None,
            capabilities: None,
        };

        match internal {
            EventToClientInternal::AccountStateChanged { state } => {
                value.event = EventType::AccountStateChanged;
                value.account_state = Some(state);
            }
            EventToClientInternal::AccountCapabilitiesChanged { capabilities } => {
                value.event = EventType::AccountCapabilitiesChanged;
                value.capabilities = Some(capabilities);
            }
        }

        value
    }
}

/// Used with database
#[derive(
    Debug,
    Serialize,
    Deserialize,
    ToSchema,
    Clone,
    Eq,
    Hash,
    PartialEq,
    Copy,
    Queryable,
    Identifiable,
    Selectable,
)]
#[diesel(table_name = crate::schema::account_id)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccountIdInternal {
    pub id: AccountIdDb,
    pub uuid: AccountId,
}

impl AccountIdInternal {
    pub fn new(id: AccountIdDb, uuid: AccountId) -> Self {
        Self { id, uuid }
    }

    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.uuid.account_id
    }

    pub fn as_db_id(&self) -> &AccountIdDb {
        &self.id
    }

    pub fn row_id(&self) -> i64 {
        self.id.0
    }

    pub fn as_id(&self) -> AccountId {
        self.uuid
    }
}

impl From<AccountIdInternal> for uuid::Uuid {
    fn from(value: AccountIdInternal) -> Self {
        value.uuid.into()
    }
}

impl From<AccountIdInternal> for AccountId {
    fn from(value: AccountIdInternal) -> Self {
        value.as_id()
    }
}

impl std::fmt::Display for AccountIdInternal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

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
pub struct AccountId {
    pub account_id: uuid::Uuid,
}

impl AccountId {
    pub fn new(account_id: uuid::Uuid) -> Self {
        Self { account_id }
    }

    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.account_id
    }

    pub fn to_string(&self) -> String {
        self.account_id.hyphenated().to_string()
    }
}

impl From<AccountId> for uuid::Uuid {
    fn from(value: AccountId) -> Self {
        value.account_id
    }
}

diesel_uuid_wrapper!(AccountId);

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Selectable, Queryable)]
#[diesel(table_name = crate::schema::access_token)]
#[diesel(check_for_backend(crate::Db))]
pub struct AccessTokenRaw {
    pub token: Option<String>,
}

/// This is just a random string.
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct AccessToken {
    /// API token which server generates.
    access_token: String,
}

impl AccessToken {
    pub fn generate_new() -> Self {
        Self {
            access_token: uuid::Uuid::new_v4().simple().to_string(),
        }
    }

    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }

    pub fn into_string(self) -> String {
        self.access_token
    }

    pub fn as_str(&self) -> &str {
        &self.access_token
    }
}

#[derive(Debug, Selectable, Queryable)]
#[diesel(table_name = crate::schema::refresh_token)]
#[diesel(check_for_backend(crate::Db))]
pub struct RefreshTokenRaw {
    pub token: Option<Vec<u8>>,
}

/// This is just a really long random number which is Base64 encoded.
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq)]
pub struct RefreshToken {
    token: String,
}

impl RefreshToken {
    pub fn generate_new_with_bytes() -> (Self, Vec<u8>) {
        let mut token = Vec::new();

        // TODO: use longer refresh token
        for _ in 1..=2 {
            token.extend(uuid::Uuid::new_v4().to_bytes_le())
        }

        (Self::from_bytes(&token), token)
    }

    pub fn generate_new() -> Self {
        let (token, _bytes) = Self::generate_new_with_bytes();
        token
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            token: base64::engine::general_purpose::STANDARD.encode(data),
        }
    }

    /// String must be base64 encoded
    /// TODO: add checks?
    pub fn from_string(token: String) -> Self {
        Self { token }
    }

    /// Base64 string
    pub fn into_string(self) -> String {
        self.token
    }

    /// Base64 string
    pub fn as_str(&self) -> &str {
        &self.token
    }

    pub fn bytes(&self) -> Result<Vec<u8>, base64::DecodeError> {
        base64::engine::general_purpose::STANDARD.decode(&self.token)
    }
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    sqlx::Type,
    PartialEq,
    Eq,
    Hash,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct AccountIdDb(pub i64);

impl AccountIdDb {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    pub fn as_i64(&self) -> &i64 {
        &self.0
    }
}

diesel_i64_wrapper!(AccountIdDb);

// #[derive(Debug, Serialize, Deserialize, Clone, Copy, sqlx::Type, PartialEq, Eq, Hash, FromSqlRow, AsExpression)]
// #[diesel(sql_type = BigInt)]
// #[serde(transparent)]
// #[sqlx(transparent)]
// pub struct DbId(pub i64);

// impl DbId {
//     pub fn new(id: i64) -> Self {
//         Self(id)
//     }

//     pub fn as_i64(&self) -> &i64 {
//         &self.0
//     }
// }

// diesel_i64_wrapper!(DbId);
