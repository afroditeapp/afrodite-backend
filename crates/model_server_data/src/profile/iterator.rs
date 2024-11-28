
use model::{AccountId, NextNumberStorage, ProfileContentVersion};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{LastSeenTime, ProfileVersion};



/// Session ID type for profile iterator so that client can detect
/// server restarts and ask user to refresh profiles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProfileIteratorSessionIdInternal {
    id: i64,
}

impl ProfileIteratorSessionIdInternal {
    /// Current implementation uses i64. Only requirement for this
    /// type is that next one should be different than the previous.
    pub fn create(storage: &mut NextNumberStorage) -> Self {
        Self {
            id: storage.get_and_increment(),
        }
    }
}

/// Session ID type for profile iterator so that client can detect
/// server restarts and ask user to refresh profiles.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ProfileIteratorSessionId {
    id: i64,
}

impl From<ProfileIteratorSessionIdInternal> for ProfileIteratorSessionId {
    fn from(value: ProfileIteratorSessionIdInternal) -> Self {
        Self {
            id: value.id,
        }
    }
}



#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, PartialEq)]
pub struct ProfileLink {
    id: AccountId,
    version: ProfileVersion,
    /// This is optional because media component owns it.
    content_version: Option<ProfileContentVersion>,
    /// If the last seen time is not None, then it is Unix timestamp or -1 if
    /// the profile is currently online.
    last_seen_time: Option<LastSeenTime>,
}

impl ProfileLink {
    pub fn new(
        id: AccountId,
        version: ProfileVersion,
        content_version: Option<ProfileContentVersion>,
        last_seen_time: Option<LastSeenTime>,
    ) -> Self {
        Self {
            id,
            version,
            content_version,
            last_seen_time,
        }
    }

    pub fn last_seen_time(&self) -> Option<LastSeenTime> {
        self.last_seen_time
    }

    pub fn set_last_seen_time(&mut self, value: LastSeenTime) {
        self.last_seen_time = Some(value);
    }
}
