use diesel::{deserialize::FromSqlRow, expression::AsExpression, sql_types::{BigInt, Text}};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_wrapper, diesel_string_wrapper};
use utoipa::{IntoParams, ToSchema};


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
#[diesel(sql_type = BigInt)]
pub struct PublicKeyId {
    pub id: i64,
}

impl PublicKeyId {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.id
    }
}

diesel_i64_wrapper!(PublicKeyId);

/// Version number for asymmetric encryption public key data which
/// client defines. This allows changing client's end-to-end crypto
/// implementation.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    IntoParams,
    PartialEq,
    Default,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = BigInt)]
pub struct PublicKeyVersion {
    pub version: i64,
}

impl PublicKeyVersion {
    pub fn new(id: i64) -> Self {
        Self { version: id }
    }

    pub fn as_i64(&self) -> &i64 {
        &self.version
    }
}

diesel_i64_wrapper!(PublicKeyVersion);

// TOOD(prod): Public key data lenght limit

/// Data for asymmetric encryption public key. Client defines the
/// format for the public key.
#[derive(Debug, Deserialize, Serialize, ToSchema, Clone, Eq, Hash, PartialEq, diesel::FromSqlRow, diesel::AsExpression)]
#[diesel(sql_type = Text)]
pub struct PublicKeyData {
    data: String,
}

impl PublicKeyData {
    pub fn new(data: String) -> Self {
        Self {
            data
        }
    }

    pub fn into_string(self) -> String {
        self.data
    }

    pub fn as_str(&self) -> &str {
        &self.data
    }
}

diesel_string_wrapper!(PublicKeyData);

#[derive(
    Debug,
    Clone,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
)]
pub struct PublicKey {
    pub id: PublicKeyId,
    pub version: PublicKeyVersion,
    pub data: PublicKeyData,
}

#[derive(
    Debug,
    Clone,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
)]
pub struct GetPublicKey {
    pub key: Option<PublicKey>,
}


#[derive(
    Debug,
    Clone,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
)]
pub struct SetPublicKey {
    pub version: PublicKeyVersion,
    pub data: PublicKeyData,
}
