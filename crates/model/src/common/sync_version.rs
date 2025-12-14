use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug)]
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
        let data_type = SyncCheckDataType::try_from(data_type_u8).map_err(|e| e.to_string())?;
        let version = SyncVersionFromClient(version_u8);
        Ok(Self { data_type, version })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(u8)]
pub enum SyncCheckDataType {
    Account = 0,
    ReveivedLikes = 1,
    ClientConfig = 2,
    Profile = 3,
    News = 4,
    MediaContent = 5,
    DailyLikesLeft = 6,
    PushNotificationInfo = 7,
    /// Special value without valid [SyncVersionFromClient] informing
    /// the server that client has info that server maintenance is scheduled.
    ServerMaintenanceIsScheduled = 255,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyncVersionFromClient(u8);

impl SyncVersionFromClient {
    pub fn new(version: u8) -> Self {
        Self(version)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, ToSchema)]
/// Sync version stored on the server. The value has range of [0, 255].
pub struct SyncVersion {
    version: i16,
}

impl SyncVersion {
    pub const MAX_VALUE: i16 = u8::MAX as i16;

    pub fn new(id: i16) -> Self {
        Self {
            version: id.clamp(0, Self::MAX_VALUE),
        }
    }

    pub fn as_i16(&self) -> &i16 {
        &self.version
    }

    fn check_is_sync_required(&self, client_value: SyncVersionFromClient) -> SyncCheckResult {
        if client_value.0 as i16 >= Self::MAX_VALUE {
            SyncCheckResult::ResetVersionAndSync
        } else if client_value.0 as i16 == self.version {
            SyncCheckResult::DoNothing
        } else {
            SyncCheckResult::Sync
        }
    }

    fn increment_if_not_max_value(&self) -> Self {
        if self.version >= Self::MAX_VALUE {
            Self {
                version: Self::MAX_VALUE,
            }
        } else {
            Self {
                version: self.version + 1,
            }
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

#[macro_export]
macro_rules! sync_version_wrappers {
    (
        $(
            $( #[doc = $doc:expr] )*
            $name:ident ,
        )*
    ) => {
        $(
            $(
                #[doc = $doc]
            )*
            #[derive(
                Debug,
                serde::Serialize,
                serde::Deserialize,
                Default,
                Clone,
                Copy,
                PartialEq,
                Eq,
                Hash,
                utoipa::ToSchema,
                diesel::FromSqlRow,
                diesel::AsExpression,
            )]
            #[diesel(sql_type = diesel::sql_types::SmallInt)]
            pub struct $name {
                #[serde(flatten)]
                version: $crate::SyncVersion
            }

            impl $name {
                pub fn new(id: i16) -> Self {
                    Self { version: $crate::SyncVersion::new(id) }
                }

                pub fn as_i16(&self) -> &i16 {
                    self.version.as_i16()
                }
            }

            impl TryFrom<i16> for $name {
                type Error = String;

                fn try_from(value: i16) -> Result<Self, Self::Error> {
                    Ok(Self { version: $crate::SyncVersion::new(value) })
                }
            }

            impl AsRef<i16> for $name {
                fn as_ref(&self) -> &i16 {
                    self.version.as_i16()
                }
            }

            simple_backend_model::diesel_i16_wrapper!($name);

            impl $crate::SyncVersionUtils for $name {
                fn sync_version(&self) -> $crate::SyncVersion {
                    self.version
                }

                fn new_with_sync_version(sync_version: $crate::SyncVersion) -> Self {
                    Self { version: sync_version }
                }
            }
        )*
    };
}

sync_version_wrappers!(AccountSyncVersion, ClientConfigSyncVersion,);
