use diesel::{
    deserialize::FromSqlRow,
    expression::AsExpression,
    sql_types::{BigInt, Binary, SmallInt},
};
use serde::{Deserialize, Serialize};
use simple_backend_model::{UnixTime, diesel_db_i16_is_u8_struct, diesel_uuid_wrapper};
use simple_backend_utils::diesel_i64_wrapper;
use utoipa::{IntoParams, ToSchema};

use crate::{AccountId, ProfileContentVersion};

mod attributes_schema;
pub use attributes_schema::*;

mod search;
pub use search::*;

#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    IntoParams,
    PartialEq,
    Eq,
    Hash,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = Binary)]
pub struct ProfileVersion {
    v: simple_backend_utils::UuidBase64Url,
}

impl ProfileVersion {
    pub fn new_base_64_url(version: simple_backend_utils::UuidBase64Url) -> Self {
        Self { v: version }
    }

    pub fn new_random() -> Self {
        Self {
            v: simple_backend_utils::UuidBase64Url::new_random_id(),
        }
    }
}

impl TryFrom<simple_backend_utils::UuidBase64Url> for ProfileVersion {
    type Error = String;

    fn try_from(v: simple_backend_utils::UuidBase64Url) -> Result<Self, Self::Error> {
        Ok(Self { v })
    }
}

impl AsRef<simple_backend_utils::UuidBase64Url> for ProfileVersion {
    fn as_ref(&self) -> &simple_backend_utils::UuidBase64Url {
        &self.v
    }
}

diesel_uuid_wrapper!(ProfileVersion);

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ProfileLink {
    a: AccountId,
    p: ProfileVersion,
    c: ProfileContentVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    l: Option<LastSeenTime>,
}

impl ProfileLink {
    pub fn new(
        id: AccountId,
        version: ProfileVersion,
        content_version: ProfileContentVersion,
        last_seen_time: Option<LastSeenTime>,
    ) -> Self {
        Self {
            a: id,
            p: version,
            c: content_version,
            l: last_seen_time,
        }
    }

    pub fn account_id(&self) -> AccountId {
        self.a
    }

    pub fn profile_version(&self) -> ProfileVersion {
        self.p
    }

    pub fn profile_content_version(&self) -> ProfileContentVersion {
        self.c
    }

    pub fn last_seen_time(&self) -> Option<LastSeenTime> {
        self.l
    }
}

/// Profile age value which is in inclusive range `[18, 99]`.
///
/// This serializes to i64, so this must not be added to API doc.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Eq,
    diesel::FromSqlRow,
    diesel::AsExpression,
)]
#[diesel(sql_type = SmallInt)]
#[serde(try_from = "i16")]
#[serde(into = "i16")]
pub struct ProfileAge {
    value: u8,
}

impl ProfileAge {
    pub const MIN_AGE: u8 = 18;
    pub const MAX_AGE: u8 = 99;

    pub fn new_clamped(age: u8) -> Self {
        Self {
            value: age.clamp(Self::MIN_AGE, Self::MAX_AGE),
        }
    }
    pub fn value(&self) -> u8 {
        self.value
    }
}

impl Default for ProfileAge {
    fn default() -> Self {
        Self {
            value: Self::MIN_AGE,
        }
    }
}

impl TryFrom<i16> for ProfileAge {
    type Error = String;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        if value < Self::MIN_AGE as i16 || value > Self::MAX_AGE as i16 {
            Err(format!(
                "Profile age must be in range [{}, {}]",
                Self::MIN_AGE,
                Self::MAX_AGE
            ))
        } else {
            Ok(Self { value: value as u8 })
        }
    }
}

impl From<ProfileAge> for i16 {
    fn from(value: ProfileAge) -> Self {
        value.value as i16
    }
}

diesel_db_i16_is_u8_struct!(ProfileAge);

#[derive(Debug, Clone, Copy, Default, Serialize, FromSqlRow, AsExpression)]
#[diesel(sql_type = BigInt)]
pub struct LastSeenUnixTime {
    pub ut: UnixTime,
}

impl LastSeenUnixTime {
    pub fn current_time() -> Self {
        Self {
            ut: UnixTime::current_time(),
        }
    }
}

impl TryFrom<i64> for LastSeenUnixTime {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self {
            ut: UnixTime::new(value),
        })
    }
}

impl AsRef<i64> for LastSeenUnixTime {
    fn as_ref(&self) -> &i64 {
        self.ut.as_i64()
    }
}

diesel_i64_wrapper!(LastSeenUnixTime);

/// Account's most recent disconnect time.
///
/// If the last seen time is not None, then it is Unix timestamp or -1 if
/// the profile is currently online.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct LastSeenTime(i64);

impl LastSeenTime {
    pub const ONLINE: Self = Self(-1);

    pub fn new(raw: i64) -> Self {
        Self(raw)
    }

    pub fn raw(&self) -> i64 {
        self.0
    }

    /// Return None if account is currently online.
    pub fn last_seen_unix_time(&self) -> Option<LastSeenUnixTime> {
        if *self != Self::ONLINE {
            Some(LastSeenUnixTime {
                ut: UnixTime::new(self.raw()),
            })
        } else {
            None
        }
    }
}

impl From<LastSeenUnixTime> for LastSeenTime {
    fn from(value: LastSeenUnixTime) -> Self {
        Self(value.ut.ut)
    }
}
