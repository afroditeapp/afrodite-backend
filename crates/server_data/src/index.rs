use std::{
    collections::HashMap,
    mem::size_of,
    num::{NonZeroU16, NonZeroU8},
    sync::Arc,
};

use area::{IndexArea, LocationIndexArea};
use config::{file::LocationConfig, Config};
use error_stack::ResultExt;
use location::{IndexSize, ReadIndex};
use model::{AccountId, UnixTime};
use model_server_data::{
    CellData, Location, LocationIndexKey, LocationIndexProfileData, LocationInternal, MaxDistanceKm, MinDistanceKm, ProfileLink, ProfileQueryMakerDetails
};
use server_common::data::index::IndexError;
use tokio::sync::RwLock;
use tracing::info;

use self::location::{IndexUpdater, LocationIndex, LocationIndexIteratorState};
use crate::{cache::LastSeenTimeUpdated, db_manager::InternalWriting};

pub mod location;
pub mod area;

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

pub struct LocationIndexInfoCreator {
    config: LocationConfig,
}

impl LocationIndexInfoCreator {
    pub fn new(config: LocationConfig) -> Self {
        Self { config }
    }

    pub fn create_one(&self, index_cell_square_km: NonZeroU8) -> String {
        self.create_one_internal(index_cell_square_km, false)
    }

    #[allow(clippy::format_in_format_args)]
    fn create_one_internal(
        &self,
        index_cell_square_km: NonZeroU8,
        whitespace_padding: bool,
    ) -> String {
        let mut location = self.config.clone();
        location.index_cell_square_km = index_cell_square_km;
        let coordinates = CoordinateManager::new(location);
        let (width, height): (NonZeroU16, NonZeroU16) = (
            coordinates.width().try_into().unwrap(),
            coordinates.height().try_into().unwrap(),
        );
        let byte_count = width.get() as usize * height.get() as usize * size_of::<CellData>();
        let size = format!("Location index size: {}x{}, ", width, height);
        let bytes = format!("bytes: {}, ", format_size_in_bytes(byte_count));
        let zoom = format!("zoom: {}, ", coordinates.zoom_level());
        let len = format!(
            "tile side length: {:.2} km",
            coordinates.tile_side_length_km()
        );
        if whitespace_padding {
            format!("{:<35}{:<20}{:<10}{}", size, bytes, zoom, len,)
        } else {
            format!("{}{}{}{}", size, bytes, zoom, len,)
        }
    }

    pub fn create_all(&self) -> String {
        let mut info = String::new();
        for (_, tile_lenght) in ZOOM_LEVEL_AND_TILE_LENGHT {
            let tile_lenght_u64 = *tile_lenght as u64;
            let converted =
                TryInto::<u8>::try_into(tile_lenght_u64).and_then(TryInto::<NonZeroU8>::try_into);
            match converted {
                Ok(lenght) => {
                    info.push_str(&self.create_one_internal(lenght, true));
                    info.push('\n');
                }
                Err(_) => continue,
            }
        }

        // Pop final newline
        info.pop();

        info
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

fn format_size_in_bytes(size: usize) -> String {
    let mut size = size as f64;
    let mut unit = 0;
    while size > 1024.0 && unit < 3 {
        size /= 1024.0;
        unit += 1;
    }
    let unit = match unit {
        0 => "B",
        1 => "KiB",
        2 => "MiB",
        3 => "GiB",
        _ => "error",
    };
    format!("{:.2} {}", size, unit)
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

    pub fn index_width(&self) -> u16 {
        self.index.width() as u16
    }

    pub fn index_height(&self) -> u16 {
        self.index.height() as u16
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
            .to_index_area(location.into(), min_distance, max_distance)
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

/// OpenStreetMap zoom levels and map tile side length in kilometers.
/// Data is from GitHub Codepilot.
const ZOOM_LEVEL_AND_TILE_LENGHT: &[(u8, f64)] = &[
    (9, 305.0),
    (10, 153.0),
    (11, 76.5),
    (12, 38.2),
    (13, 19.1),
    (14, 9.55),
    (15, 4.77),
    (16, 2.39),
    (17, 1.19),
];

fn find_nearest_zoom_level(square_km: NonZeroU8) -> (u8, f64) {
    let square_km = square_km.get() as f64;
    let (mut nearest_zoom_level, mut nearest_distance) = ZOOM_LEVEL_AND_TILE_LENGHT[0];
    let mut nearest_tile_lenght = nearest_distance;
    for (zoom_level, tile_length) in ZOOM_LEVEL_AND_TILE_LENGHT {
        let distance = (square_km - tile_length).abs();
        if distance < nearest_distance {
            nearest_distance = distance;
            nearest_zoom_level = *zoom_level;
            nearest_tile_lenght = *tile_length;
        }
    }
    (nearest_zoom_level, nearest_tile_lenght)
}

// https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames#Lon./lat._to_tile_numbers
// n = 2 ^ zoom
// xtile = n * ((lon_deg + 180) / 360)
// ytile = n * (1 - (log(tan(lat_rad) + sec(lat_rad)) / π)) / 2

fn calculate_tile_x(longitude_deg: f64, zoom_level: u8) -> u32 {
    let n = 2.0_f64.powi(zoom_level as i32);
    let x = n * ((longitude_deg + 180.0) / 360.0);
    x as u32
}

fn calculate_tile_y(latitude_deg: f64, zoom_level: u8) -> u32 {
    let n = 2.0_f64.powi(zoom_level as i32);
    let latitude_rad = latitude_deg.to_radians();
    let y = n
        * (1.0 - (latitude_rad.tan() + (1.0 / latitude_rad.cos())).ln() / std::f64::consts::PI)
        / 2.0;
    y as u32
}

#[derive(Debug)]
pub struct CoordinateManager {
    pub config: LocationConfig,
    pub longitude_one_km_in_degrees: f64,
    pub zoom_level: u8,
    pub tile_side_length_km: f64,
}

impl CoordinateManager {
    fn new(config: LocationConfig) -> Self {
        let (zoom_level, tile_side_length_km) =
            find_nearest_zoom_level(config.index_cell_square_km);
        Self {
            zoom_level,
            tile_side_length_km,
            config,
            longitude_one_km_in_degrees: calculate_longitude_one_km_in_degrees(),
        }
    }

    fn zoom_level(&self) -> u8 {
        self.zoom_level
    }

    fn tile_side_length_km(&self) -> f64 {
        self.tile_side_length_km
    }

    // Max y tile number of the index area.
    fn y_max_tile(&self) -> u32 {
        calculate_tile_y(self.config.latitude_bottom_right, self.zoom_level)
    }

    // Max x tile number of the index area.
    fn x_max_tile(&self) -> u32 {
        calculate_tile_x(self.config.longitude_bottom_right, self.zoom_level)
    }

    fn height(&self) -> u16 {
        let y_start = calculate_tile_y(self.config.latitude_top_left, self.zoom_level);
        u32::max(1, self.y_max_tile() - y_start) as u16
    }

    fn width(&self) -> u16 {
        let x_start = calculate_tile_x(self.config.longitude_top_left, self.zoom_level);
        u32::max(1, self.x_max_tile() - x_start) as u16
    }

    pub fn to_index_area(
        &self,
        location: LocationInternal,
        min_distance: Option<MinDistanceKm>,
        max_distance: Option<MaxDistanceKm>,
    ) -> LocationIndexArea {
        let profile_location = self.location_to_index_key(location);

        let area_inner = min_distance.map(
            |min_distance| IndexArea::new(self, location, min_distance.value)
        );

        let area_outer = if let Some(max_distance) = max_distance {
            IndexArea::new(self, location, max_distance.value)
        } else {
            IndexArea::max_area(self.width(), self.height())
        };

        LocationIndexArea {
            area_inner,
            area_outer,
            profile_location,
        }
    }

    pub fn location_to_index_key(&self, location: LocationInternal) -> LocationIndexKey {
        LocationIndexKey {
            y: self.calculate_index_y_key(location.latitude()),
            x: self.calculate_index_x_key(location.longitude()),
        }
    }

    fn calculate_index_x_key(&self, longitude: f64) -> u16 {
        let longitude = longitude.clamp(self.longitude_min(), self.longitude_max());

        let x_tile = calculate_tile_x(longitude, self.zoom_level);
        let x = (self.x_max_tile() - x_tile) as u16;

        let x_max = self.width() - 1;
        let x = x.clamp(0, x_max);
        x_max - x
    }

    fn calculate_index_y_key(&self, latitude: f64) -> u16 {
        let latitude = latitude.clamp(self.latitude_min(), self.latitude_max());

        let y_tile = calculate_tile_y(latitude, self.zoom_level);
        let y = (self.y_max_tile() - y_tile) as u16;

        let y_max = self.height() - 1;
        let y = y.clamp(0, y_max);
        y_max - y
    }

    fn longitude_min(&self) -> f64 {
        self.config.longitude_top_left
    }

    fn longitude_max(&self) -> f64 {
        self.config.longitude_bottom_right
    }

    fn latitude_min(&self) -> f64 {
        self.config.latitude_bottom_right
    }

    fn latitude_max(&self) -> f64 {
        self.config.latitude_top_left
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZeroU8;

    use config::file::LocationConfig;

    use super::CoordinateManager;

    fn manager() -> CoordinateManager {
        CoordinateManager::new(LocationConfig {
            latitude_top_left: 10.0,
            longitude_top_left: 0.0,
            latitude_bottom_right: 0.0,
            longitude_bottom_right: 10.0,
            index_cell_square_km: NonZeroU8::MAX,
        })
    }

    #[test]
    fn latitude_to_index_y_bottom() {
        let manager = manager();
        assert_eq!(manager.calculate_index_y_key(0.0), manager.height() - 1);
    }

    #[test]
    fn latitude_to_index_y_top() {
        let manager = manager();
        assert_eq!(manager.calculate_index_y_key(10.0), 0);
    }

    #[test]
    fn longitude_to_index_x_left() {
        let manager = manager();
        assert_eq!(manager.calculate_index_x_key(0.0), 0);
    }

    #[test]
    fn longitude_to_index_x_right() {
        let manager = manager();
        assert_eq!(manager.calculate_index_x_key(10.0), manager.width() - 1);
    }
}
