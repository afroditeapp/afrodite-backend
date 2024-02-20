use base64::Engine;
use diesel::{
    prelude::*,
    sql_types::{BigInt, Binary},
    AsExpression, FromSqlRow,
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{diesel_i64_try_from, diesel_i64_wrapper, diesel_uuid_wrapper};
use utoipa::{IntoParams, ToSchema};

use crate::{
    schema_sqlite_types::Integer, Account, AccountState, Capabilities, ContentProcessingId, ContentProcessingState, MessageNumber, ModerationQueueNumber, ModerationQueueType, Profile, ProfileVisibility
};


pub struct SyncDataVersionFromClient {
    pub data_type: SyncCheckDataType,
    pub version: SyncVersionFromClient,
}

impl SyncDataVersionFromClient {
    pub fn parse_sync_data_list(data: &[u8]) -> Result<Vec<Self>, String> {
        let mut data_versions = vec![];
        for chunk in data.chunks_exact(2) {
            data_versions.push(Self::parse([chunk[0], chunk[1]])?);
        }

        Ok(data_versions)
    }

    fn parse([data_type_u8, version_u8]: [u8; 2]) -> Result<Self, String> {
        let data_type = SyncCheckDataType::try_from(data_type_u8)?;
        let version = SyncVersionFromClient(version_u8);
        Ok(Self { data_type, version })
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
)]
#[repr(u8)]
pub enum SyncCheckDataType {
    Account = 0,
    ReveivedLikes = 1,
    ReveivedBlocks = 2,
    SentLikes = 3,
    SentBlocks = 4,
    Matches = 5,
}

impl TryFrom<u8> for SyncCheckDataType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Account),
            _ => Err(format!("Unknown sync check data type {}", value)),
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
)]
pub enum SyncCheckResult {
    /// Reset version number to 0 and then sync data and version number to client.
    ResetVersionAndSync,
    /// Sync data and version number to client.
    Sync,
    /// Do nothing.
    DoNothing,
}


/// Sync version can range from [0, 255]. If server receives the value 255 from
/// client, the server does number wrapping and sets the version value to 0.
/// After that the server does full sync for the related data to client.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
)]
pub struct SyncVersionFromClient(u8);

impl SyncVersionFromClient {
    pub fn new(version: u8) -> Self {
        Self(version)
    }
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    ToSchema,
)]
#[serde(transparent)]
/// Sync version stored on the server. The value has range of [0, 255].
pub struct SyncVersion(i64);

impl SyncVersion {
    pub(crate) fn new(id: i64) -> Self {
        Self(id.clamp(0, u8::MAX as i64))
    }

    pub(crate) fn as_i64(&self) -> &i64 {
        &self.0
    }

    fn check_is_sync_required(&self, client_value: SyncVersionFromClient) -> SyncCheckResult {
        if client_value.0 >= u8::MAX {
            SyncCheckResult::ResetVersionAndSync
        } else if client_value.0 as i64 == self.0 {
            SyncCheckResult::DoNothing
        } else {
            SyncCheckResult::Sync
        }
    }

    fn increment_if_not_max_value(&self) -> SyncVersion {
        if self.0 >= u8::MAX as i64 {
            SyncVersion(u8::MAX as i64)
        } else {
            SyncVersion(self.0 + 1)
        }
    }
}

impl Default for SyncVersion {
    fn default() -> Self {
        Self::new(0)
    }
}

pub trait SyncVersionUtils: Sized + Default {
    fn sync_version(&self) -> SyncVersion;
    fn new_with_sync_version(sync_version: SyncVersion) -> Self;

    fn check_is_sync_required(&self, client_value: SyncVersionFromClient) -> SyncCheckResult {
        self.sync_version().check_is_sync_required(client_value)
    }

    fn increment_if_not_max_value(&self) -> Self {
        Self::new_with_sync_version(self.sync_version().increment_if_not_max_value())
    }

    fn increment_if_not_max_value_mut(&mut self) {
        let new = Self::new_with_sync_version(self.sync_version().increment_if_not_max_value());
        *self = new;
    }

    fn return_new_if_different(&self, new: Self) -> Option<Self> {
        if self.sync_version() != new.sync_version() {
            Some(new)
        } else {
            None
        }
    }
}

macro_rules! sync_version_wrappers {
    ( $( $name:ident ,)* ) => {
        $(
            #[derive(
                Debug,
                Serialize,
                Deserialize,
                Default,
                Clone,
                Copy,
                PartialEq,
                Eq,
                Hash,
                ToSchema,
                FromSqlRow,
                AsExpression,
            )]
            #[diesel(sql_type = BigInt)]
            #[serde(transparent)]
            pub struct $name(crate::SyncVersion);

            impl $name {
                pub fn new(id: i64) -> Self {
                    Self(crate::SyncVersion::new(id))
                }

                pub fn as_i64(&self) -> &i64 {
                    self.0.as_i64()
                }
            }

            diesel_i64_wrapper!($name);

            impl SyncVersionUtils for $name {
                fn sync_version(&self) -> crate::SyncVersion {
                    self.0
                }

                fn new_with_sync_version(sync_version: crate::SyncVersion) -> Self {
                    Self(sync_version)
                }
            }
        )*
    };
}

pub(crate) use sync_version_wrappers;

sync_version_wrappers!(
    AccountSyncVersion,
);
