use std::num::NonZeroU8;

use config::file::LocationConfig;
use model_server_data::{LocationIndexKey, LocationInternal, MaxDistanceKm, MinDistanceKm};

use super::{area::{IndexArea, LocationIndexArea}, data::LocationIndex};

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
pub const ZOOM_LEVEL_AND_TILE_LENGHT: &[(u8, f64)] = &[
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
    pub fn new(config: LocationConfig) -> Self {
        let (zoom_level, tile_side_length_km) =
            find_nearest_zoom_level(config.index_cell_square_km);
        Self {
            zoom_level,
            tile_side_length_km,
            config,
            longitude_one_km_in_degrees: calculate_longitude_one_km_in_degrees(),
        }
    }

    pub fn zoom_level(&self) -> u8 {
        self.zoom_level
    }

    pub fn tile_side_length_km(&self) -> f64 {
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

    pub fn height(&self) -> u16 {
        let y_start = calculate_tile_y(self.config.latitude_top_left, self.zoom_level);
        u32::max(1, self.y_max_tile() - y_start) as u16
    }

    pub fn width(&self) -> u16 {
        let x_start = calculate_tile_x(self.config.longitude_top_left, self.zoom_level);
        u32::max(1, self.x_max_tile() - x_start) as u16
    }

    pub fn to_index_area(
        &self,
        location: LocationInternal,
        min_distance: Option<MinDistanceKm>,
        max_distance: Option<MaxDistanceKm>,
        index: &LocationIndex,
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

        LocationIndexArea::new(
            area_inner,
            area_outer,
            profile_location,
            index,
        )
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
