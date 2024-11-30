use diesel::sql_types::Text;
use model::{PublicKeyId, PublicKeyVersion};
use serde::{Deserialize, Serialize};
use simple_backend_model::diesel_string_wrapper;
use utoipa::ToSchema;

// TOOD(prod): Public key data lenght limit

/// Data for asymmetric encryption public key. Client defines the
/// format for the public key.
#[derive(
    Debug,
    Deserialize,
    Serialize,
    ToSchema,
    Clone,
    Eq,
    Hash,
    PartialEq,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Text)]
pub struct PublicKeyData {
    data: String,
}

impl PublicKeyData {
    pub fn new(data: String) -> Self {
        Self { data }
    }

    pub fn into_string(self) -> String {
        self.data
    }

    pub fn as_str(&self) -> &str {
        &self.data
    }
}

diesel_string_wrapper!(PublicKeyData);

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct PublicKey {
    pub id: PublicKeyId,
    pub version: PublicKeyVersion,
    pub data: PublicKeyData,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct GetPublicKey {
    pub key: Option<PublicKey>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct SetPublicKey {
    pub version: PublicKeyVersion,
    pub data: PublicKeyData,
}
