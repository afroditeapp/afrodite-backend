use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    api::model::{AccountIdLight, Location, ProfileLink},
    config::Config,
};

use self::location::{IndexUpdater, LocationIndex, LocationIndexIteratorState, LocationIndexKey};

pub mod location;

#[derive(Debug)]
pub struct LocationIndexManager {
    config: Arc<Config>,
    index: Option<Arc<LocationIndex>>,
    profiles: RwLock<HashMap<LocationIndexKey, ProfilesAtLocation>>,
    coordinates: CoordinateManager,
}

impl LocationIndexManager {
    pub fn new(config: Arc<Config>) -> Self {
        let coordinates = CoordinateManager::new(config.clone());
        let index = if config.components().profile {
            Some(
                LocationIndex::new(
                    coordinates.width().try_into().unwrap(),
                    coordinates.height().try_into().unwrap(),
                )
                .into(),
            )
        } else {
            None
        };

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

pub struct LocationIndexIteratorGetter<'a> {
    manager: &'a LocationIndexManager,
}

impl<'a> LocationIndexIteratorGetter<'a> {
    pub fn new(manager: &'a LocationIndexManager) -> Self {
        Self { manager }
    }

    pub fn get(&self) -> Option<LocationIndexIteratorHandle> {
        LocationIndexIteratorHandle::new(self.manager)
    }
}

pub struct LocationIndexWriterGetter<'a> {
    manager: &'a LocationIndexManager,
}

impl<'a> LocationIndexWriterGetter<'a> {
    pub fn new(manager: &'a LocationIndexManager) -> Self {
        Self { manager }
    }

    pub fn get(&self) -> Option<LocationIndexWriteHandle> {
        LocationIndexWriteHandle::new(self.manager)
    }
}

#[derive(Debug)]
pub struct LocationIndexIteratorHandle<'a> {
    index: &'a Arc<LocationIndex>,
    profiles: &'a RwLock<HashMap<LocationIndexKey, ProfilesAtLocation>>,
    coordinates: &'a CoordinateManager,
}

impl<'a> LocationIndexIteratorHandle<'a> {
    pub fn new(manager: &'a LocationIndexManager) -> Option<Self> {
        let index = manager.index.as_ref()?;

        Self {
            index,
            profiles: &manager.profiles,
            coordinates: &manager.coordinates,
        }
        .into()
    }

    pub async fn next_profiles(
        &self,
        previous_iterator_state: LocationIndexIteratorState,
    ) -> (LocationIndexIteratorState, Option<Vec<ProfileLink>>) {
        let mut iterator = previous_iterator_state.to_iterator(self.index.clone());
        match iterator.next() {
            None => (iterator.into(), None),
            Some(key) => match self.profiles.read().await.get(&key) {
                None => (iterator.into(), None),
                Some(profiles) => (
                    iterator.into(),
                    Some(profiles.profiles.values().map(|p| p.clone()).collect()),
                ),
            },
        }
    }

    pub fn reset_iterator(
        &self,
        previous_iterator_state: LocationIndexIteratorState,
        location: LocationIndexKey,
    ) -> LocationIndexIteratorState {
        let mut iterator = previous_iterator_state.to_iterator(self.index.clone());
        iterator.reset(location.x, location.y);
        iterator.into()
    }
}

#[derive(Debug)]
pub struct LocationIndexWriteHandle<'a> {
    index: &'a Arc<LocationIndex>,
    profiles: &'a RwLock<HashMap<LocationIndexKey, ProfilesAtLocation>>,
    coordinates: &'a CoordinateManager,
}

impl<'a> LocationIndexWriteHandle<'a> {
    pub fn new(manager: &'a LocationIndexManager) -> Option<Self> {
        let index = manager.index.as_ref()?;

        Self {
            index,
            profiles: &manager.profiles,
            coordinates: &manager.coordinates,
        }
        .into()
    }

    pub fn coordinates_to_key(&self, location: Location) -> LocationIndexKey {
        self.coordinates
            .to_index_key(location.latitude, location.longitude)
    }

    pub async fn update_profile_location(
        &self,
        account_id: AccountIdLight,
        previous_key: LocationIndexKey,
        new_key: LocationIndexKey,
    ) {
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
                        let update_index = some_other_profiles_also.profiles.len() == 0;
                        some_other_profiles_also
                            .profiles
                            .insert(account_id, profile);
                        if update_index {
                            drop(profiles);
                            updater.flag_cell_to_have_profiles(new_key)
                        }
                    }
                    None => {
                        profiles.insert(new_key, ProfilesAtLocation::new(account_id, profile));
                        drop(profiles);
                        updater.flag_cell_to_have_profiles(new_key)
                    }
                }
            }

            if new_size == 0 {
                updater.remove_profile_flag_from_cell(previous_key);
            }
        }
    }

    pub async fn update_profile_link(
        &self,
        account_id: AccountIdLight,
        profile_link: ProfileLink,
        key: LocationIndexKey,
    ) {
        let mut profiles = self.profiles.write().await;
        match profiles.get_mut(&key) {
            Some(some_other_profiles_also) => {
                let update_index = some_other_profiles_also.profiles.len() == 0;
                some_other_profiles_also
                    .profiles
                    .insert(account_id, profile_link);
                if update_index {
                    drop(profiles);
                    let mut updater = IndexUpdater::new(self.index.clone());
                    updater.flag_cell_to_have_profiles(key)
                }
            }
            None => {
                profiles.insert(key, ProfilesAtLocation::new(account_id, profile_link));
                drop(profiles);
                let mut updater = IndexUpdater::new(self.index.clone());
                updater.flag_cell_to_have_profiles(key)
            }
        }
    }

    pub async fn remove_profile_link(&self, account_id: AccountIdLight, key: LocationIndexKey) {
        let mut profiles = self.profiles.write().await;
        match profiles.get_mut(&key) {
            Some(some_other_profiles_also) => {
                if some_other_profiles_also
                    .profiles
                    .remove(&account_id)
                    .is_some()
                    && some_other_profiles_also.profiles.len() == 0
                {
                    drop(profiles);
                    let mut updater = IndexUpdater::new(self.index.clone());
                    updater.remove_profile_flag_from_cell(key)
                }
            }
            None => (),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfilesAtLocation {
    profiles: HashMap<AccountIdLight, ProfileLink>,
}

impl ProfilesAtLocation {
    pub fn new(account_id: AccountIdLight, profile: ProfileLink) -> Self {
        let mut profiles = HashMap::new();
        profiles.insert(account_id, profile);
        Self { profiles }
    }
}

// https://stackoverflow.com/questions/1253499/simple-calculations-for-working-with-lat-lon-and-km-distance
pub const LATITUDE_ONE_KM_IN_DEGREES: f64 = 1.0 / 110.574;

// Lets just use middle point of Finland to approximate longitude.
// That probably makes the index squares practically larger in north and
// smaller in south. Or other way around.

pub fn calculate_longitude_one_km_in_degrees() -> f64 {
    1.0 / (111.320 * f64::cos(LATITUDE_FOR_LONGITUDE_CORRECTION.to_radians()).to_degrees())
}

/// Latitude value for longitude correction.
/// Hanko + (Nuorgam - Hanko)
const LATITUDE_FOR_LONGITUDE_CORRECTION: f64 = 59.8 + 70.1 - 59.8;

#[derive(Debug)]
pub struct CoordinateManager {
    pub config: Arc<Config>,
    pub longitude_one_km_in_degrees: f64,
}

impl CoordinateManager {
    fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            longitude_one_km_in_degrees: calculate_longitude_one_km_in_degrees(),
        }
    }

    fn height(&self) -> u16 {
        let height_degrees =
            self.config.location().latitude_top_left - self.config.location().latitude_bottom_right;
        let one_cell_height_degrees =
            LATITUDE_ONE_KM_IN_DEGREES * self.config.location().index_cell_square_km.get() as f64;
        (height_degrees / one_cell_height_degrees) as u16
    }

    fn width(&self) -> u16 {
        let width_degrees = self.config.location().longitude_bottom_right
            - self.config.location().longitude_top_left;
        let one_cell_width_degrees = self.longitude_one_km_in_degrees
            * self.config.location().index_cell_square_km.get() as f64;
        (width_degrees / one_cell_width_degrees) as u16
    }

    pub fn to_index_key(&self, latitude: f64, longitude: f64) -> LocationIndexKey {
        LocationIndexKey {
            y: self.calculate_index_y_key(latitude),
            x: self.calculate_index_x_key(longitude),
        }
    }

    fn calculate_index_x_key(&self, longitude: f64) -> u16 {
        let longitude = longitude.clamp(self.longitude_min(), self.longitude_max());
        let width_degrees = self.longitude_max() - longitude;
        let one_cell_width_degrees = self.longitude_one_km_in_degrees
            * self.config.location().index_cell_square_km.get() as f64;
        let x = (width_degrees / one_cell_width_degrees) as u16;
        x.clamp(0, self.width())
    }

    fn calculate_index_y_key(&self, latitude: f64) -> u16 {
        let latitude = latitude.clamp(self.latitude_min(), self.latitude_min());
        let height_degrees = self.latitude_max() - latitude;
        let one_cell_height_degrees =
            LATITUDE_ONE_KM_IN_DEGREES * self.config.location().index_cell_square_km.get() as f64;
        let y = (height_degrees / one_cell_height_degrees) as u16;
        y.clamp(0, self.height())
    }

    fn longitude_min(&self) -> f64 {
        self.config.location().longitude_top_left
    }

    fn longitude_max(&self) -> f64 {
        self.config.location().longitude_bottom_right
    }

    fn latitude_min(&self) -> f64 {
        self.config.location().latitude_top_left
    }

    fn latitude_max(&self) -> f64 {
        self.config.location().latitude_bottom_right
    }
}
