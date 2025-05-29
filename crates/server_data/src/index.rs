//! Profile index
//!
//! Allow iterating profiles from nearest to farthest starting from
//! some location.
//!
//! The first part of the index is a matrix [LocationIndexManager::index]
//! where each cell has jump info to next profile for each direction.
//! The cell also contains info is profile available at this location.
//!
//! After finding a cell with profiles, the cell coordinate info is
//! used to check available profiles from [LocationIndexManager::profiles].
//!
//! The profile index supports multiple readers and one writer.

use std::{
    collections::HashMap,
    sync::Arc,
};

use coordinates::LocationIndexArea;
use config::Config;
use coordinates::CoordinateManager;
use error_stack::ResultExt;
use info::LocationIndexInfoCreator;
use data::IndexSize;
use model::{AccountId, UnixTime};
use model_server_data::{
    Location, LocationIndexKey, LocationIndexProfileData, MaxDistanceKm, MinDistanceKm, ProfileLink, ProfileQueryMakerDetails
};
use read::LocationIndexIteratorState;
use server_common::data::index::IndexError;
use tokio::sync::RwLock;
use tracing::info;
use write::IndexUpdater;

use self::data::LocationIndex;
use crate::{cache::LastSeenTimeUpdated, db_manager::InternalWriting};

pub mod data;
pub mod read;
pub mod write;
pub mod coordinates;
pub mod info;

pub trait LocationWrite {
    fn location(&self) -> crate::index::LocationIndexWriteHandle<'_>;
    fn location_iterator(&self) -> crate::index::LocationIndexIteratorHandle<'_>;
}

impl<I: InternalWriting> LocationWrite for I {
    fn location(&self) -> crate::index::LocationIndexWriteHandle<'_> {
        crate::index::LocationIndexWriteHandle::new(InternalWriting::location(self))
    }

    fn location_iterator(&self) -> crate::index::LocationIndexIteratorHandle<'_> {
        LocationIndexIteratorHandle::new(InternalWriting::location(self))
    }
}

#[derive(Debug)]
pub struct LocationIndexManager {
    config: Arc<Config>,
    index: Arc<LocationIndex>,
    profiles: RwLock<HashMap<LocationIndexKey, ProfilesAtLocation>>,
    coordinates: CoordinateManager,
}

impl LocationIndexManager {
    pub fn new(config: Arc<Config>) -> Self {
        let coordinates = CoordinateManager::new(config.location().clone());
        // Create index also if profile features are disabled.
        // This way accidential index access will not crash the server.
        // The default index should not consume memory that much.
        let (width, height) = (
            coordinates.width().try_into().unwrap(),
            coordinates.height().try_into().unwrap(),
        );

        let index = LocationIndex::new(IndexSize::new(width), IndexSize::new(height)).into();

        info!(
            "{}",
            LocationIndexInfoCreator::new(config.location().clone())
                .create_one(config.location().index_cell_square_km),
        );

        Self {
            config,
            index,
            coordinates,
            profiles: RwLock::new(HashMap::new()),
        }
    }

    pub fn coordinates(&self) -> &CoordinateManager {
        &self.coordinates
    }
}

enum IteratorResultInternal {
    NoProfiles,
    TryAgain,
    MatchingProfilesFound { profiles: Vec<ProfileLink> },
}

#[derive(Debug)]
pub struct LocationIndexIteratorHandle<'a> {
    config: &'a Config,
    index: &'a Arc<LocationIndex>,
    profiles: &'a RwLock<HashMap<LocationIndexKey, ProfilesAtLocation>>,
}

impl<'a> LocationIndexIteratorHandle<'a> {
    pub fn new(manager: &'a LocationIndexManager) -> Self {
        Self {
            config: &manager.config,
            index: &manager.index,
            profiles: &manager.profiles,
        }
    }

    pub fn next_profiles(
        &self,
        previous_iterator_state: LocationIndexIteratorState,
        query_maker_details: &ProfileQueryMakerDetails,
    ) -> (LocationIndexIteratorState, Option<Vec<ProfileLink>>) {
        let current_time = UnixTime::current_time();
        let mut iterator_state = previous_iterator_state;
        loop {
            let (new_state, result) =
                self.next_profiles_internal(iterator_state, query_maker_details, &current_time);
            iterator_state = new_state;
            match result {
                IteratorResultInternal::NoProfiles => {
                    return (iterator_state, None);
                }
                IteratorResultInternal::MatchingProfilesFound { profiles } => {
                    return (iterator_state, Some(profiles));
                }
                IteratorResultInternal::TryAgain => {
                    continue;
                }
            }
        }
    }

    /// Iterate to next index cell which has profiles and get all matching
    /// profiles.
    fn next_profiles_internal(
        &self,
        previous_iterator_state: LocationIndexIteratorState,
        query_maker_details: &ProfileQueryMakerDetails,
        current_time: &UnixTime,
    ) -> (LocationIndexIteratorState, IteratorResultInternal) {
        let index = self.index.clone();
        let (iterator, key) = {
            let mut iterator = previous_iterator_state.into_iterator(index);
            let key = iterator.next();
            (iterator, key)
        };
        let result = match key {
            None => IteratorResultInternal::NoProfiles,
            Some(key) => match self.profiles.blocking_read().get(&key) {
                // Possible data race occurred where profile was removed
                // from the data storage when iterating the index.
                None => IteratorResultInternal::TryAgain,
                // TODO(perf): Currently all profiles in one index cell are
                // sent to client, which might cause issues if everyone will
                // set profile to same location.
                Some(profiles) => {
                    let matches: Vec<ProfileLink> = profiles
                        .profiles
                        .values()
                        .filter(|p| {
                            p.is_match(
                                query_maker_details,
                                self.config.profile_attributes(),
                                current_time,
                            )
                        })
                        .map(|p| p.to_profile_link_value())
                        .collect();
                    if matches.is_empty() {
                        IteratorResultInternal::TryAgain
                    } else {
                        IteratorResultInternal::MatchingProfilesFound { profiles: matches }
                    }
                }
            },
        };
        (iterator.into(), result)
    }

    pub fn new_iterator_state(
        &self,
        area: &LocationIndexArea,
        random: bool,
    ) -> LocationIndexIteratorState {
        LocationIndexIteratorState::new(area, random, &self.index)
    }

    pub fn index(&self) -> &LocationIndex {
        self.index
    }
}

#[derive(Debug)]
pub struct LocationIndexWriteHandle<'a> {
    index: &'a Arc<LocationIndex>,
    profiles: &'a RwLock<HashMap<LocationIndexKey, ProfilesAtLocation>>,
    coordinates: &'a CoordinateManager,
}

impl<'a> LocationIndexWriteHandle<'a> {
    pub fn new(manager: &'a LocationIndexManager) -> Self {
        Self {
            index: &manager.index,
            profiles: &manager.profiles,
            coordinates: &manager.coordinates,
        }
    }

    pub fn coordinates_to_area(
        &self,
        location: Location,
        min_distance: Option<MinDistanceKm>,
        max_distance: Option<MaxDistanceKm>,
    ) -> LocationIndexArea {
        self.coordinates
            .to_index_area(location.into(), min_distance, max_distance, self.index)
    }

    /// Move LocationIndexProfileData to another index location
    pub async fn update_profile_location(
        &self,
        account_id: AccountId,
        previous_key: LocationIndexKey,
        new_key: LocationIndexKey,
    ) -> error_stack::Result<(), IndexError> {
        if previous_key == new_key {
            // No update needed. If return would not be here then
            // if new_size == 0 check would make profile disappear.
            return Ok(());
        }

        let mut profiles = self.profiles.write().await;
        let data = match profiles.get_mut(&previous_key) {
            Some(p) => {
                let current_profile = p.profiles.remove(&account_id);
                Some((current_profile, p.profiles.len()))
            }
            None => None,
        };

        if let Some((current_profile, new_size)) = data {
            let mut updater = IndexUpdater::new(self.index.clone());

            if let Some(profile) = current_profile {
                match profiles.get_mut(&new_key) {
                    Some(some_other_profiles_also) => {
                        let update_index = some_other_profiles_also.profiles.is_empty();
                        some_other_profiles_also
                            .profiles
                            .insert(account_id, profile);
                        if update_index {
                            drop(profiles);
                            tokio::task::spawn_blocking(move || {
                                updater.flag_cell_to_have_profiles(new_key)
                            })
                            .await
                            .change_context(IndexError::ProfileIndex)?;
                        }
                    }
                    None => {
                        profiles.insert(new_key, ProfilesAtLocation::new(account_id, profile));
                        drop(profiles);
                        tokio::task::spawn_blocking(move || {
                            updater.flag_cell_to_have_profiles(new_key)
                        })
                        .await
                        .change_context(IndexError::ProfileIndex)?;
                    }
                }
            } else {
                // Drop before calling remove_profile_flag_from_cell, so
                // reading will not be blocked that much.
                drop(profiles);
            }

            if new_size == 0 {
                let mut updater = IndexUpdater::new(self.index.clone());
                tokio::task::spawn_blocking(move || {
                    updater.remove_profile_flag_from_cell(previous_key);
                })
                .await
                .change_context(IndexError::ProfileIndex)?;
            }
        }

        Ok(())
    }

    pub async fn update_last_seen_time(&self, account_id: AccountId, info: LastSeenTimeUpdated) {
        // TODO(perf): This is currently called also when profile does not exist
        // in location index. Most likely profile visibility check can be done
        // before creating LastSeenTimeUpdated.
        let profiles = self.profiles.read().await;
        profiles
            .get(&info.current_position)
            .and_then(|v| v.profiles.get(&account_id))
            .inspect(|data| data.update_last_seen_value(info.last_seen_time));
    }

    /// Set LocationIndexProfileData to specific index location
    pub async fn update_profile_data(
        &self,
        account_id: AccountId,
        profile_data: LocationIndexProfileData,
        key: LocationIndexKey,
    ) -> error_stack::Result<(), IndexError> {
        let mut profiles = self.profiles.write().await;
        match profiles.get_mut(&key) {
            Some(some_other_profiles_also) => {
                let update_index = some_other_profiles_also.profiles.is_empty();
                some_other_profiles_also
                    .profiles
                    .insert(account_id, profile_data);
                if update_index {
                    drop(profiles);
                    let mut updater = IndexUpdater::new(self.index.clone());
                    tokio::task::spawn_blocking(move || updater.flag_cell_to_have_profiles(key))
                        .await
                        .change_context(IndexError::ProfileIndex)?;
                }
            }
            None => {
                profiles.insert(key, ProfilesAtLocation::new(account_id, profile_data));
                drop(profiles);
                let mut updater = IndexUpdater::new(self.index.clone());
                tokio::task::spawn_blocking(move || updater.flag_cell_to_have_profiles(key))
                    .await
                    .change_context(IndexError::ProfileIndex)?;
            }
        }
        Ok(())
    }

    /// Remove LocationIndexProfileData from specific index location
    pub async fn remove_profile_data(
        &self,
        account_id: AccountId,
        key: LocationIndexKey,
    ) -> error_stack::Result<(), IndexError> {
        let mut profiles = self.profiles.write().await;
        if let Some(some_other_profiles_also) = profiles.get_mut(&key) {
            let removed = some_other_profiles_also.profiles.remove(&account_id);

            if removed.is_some() && some_other_profiles_also.profiles.is_empty() {
                profiles.remove(&key);
                drop(profiles);
                let mut updater = IndexUpdater::new(self.index.clone());
                tokio::task::spawn_blocking(move || updater.remove_profile_flag_from_cell(key))
                    .await
                    .change_context(IndexError::ProfileIndex)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ProfilesAtLocation {
    profiles: HashMap<AccountId, LocationIndexProfileData>,
}

impl ProfilesAtLocation {
    pub fn new(account_id: AccountId, profile: LocationIndexProfileData) -> Self {
        let mut profiles = HashMap::new();
        profiles.insert(account_id, profile);
        Self { profiles }
    }
}
