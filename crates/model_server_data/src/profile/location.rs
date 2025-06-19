use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Default)]
#[serde(try_from = "f64")]
#[serde(into = "f64")]
pub struct FiniteDouble {
    value: f64,
}

impl TryFrom<f64> for FiniteDouble {
    type Error = String;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_finite() {
            Ok(Self { value })
        } else {
            Err("Value must be finite".to_string())
        }
    }
}

impl From<FiniteDouble> for f64 {
    fn from(value: FiniteDouble) -> Self {
        value.value
    }
}

/// Location in latitude and longitude.
/// The values are not NaN, infinity or negative infinity.
#[derive(
    Debug,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    ToSchema,
    PartialEq,
    Default,
    Queryable,
    Selectable,
    AsChangeset,
)]
#[diesel(table_name = crate::schema::profile_state)]
#[diesel(check_for_backend(crate::Db))]
pub struct Location {
    #[schema(value_type = f64)]
    #[diesel(deserialize_as = f64, serialize_as = f64)]
    latitude: FiniteDouble,
    #[schema(value_type = f64)]
    #[diesel(deserialize_as = f64, serialize_as = f64)]
    longitude: FiniteDouble,
}

impl Location {
    pub fn latitude(&self) -> f64 {
        self.latitude.into()
    }

    pub fn longitude(&self) -> f64 {
        self.longitude.into()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LocationInternal {
    latitude: f64,
    longitude: f64,
}

impl LocationInternal {
    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    pub fn move_kilometers(&self, dy: f64, dx: f64) -> Self {
        // https://stackoverflow.com/questions/7477003/calculating-new-longitude-latitude-from-old-n-meters
        use std::f64::consts::PI;
        const R_EARTH: f64 = 6378.0;
        let new_latitude = self.latitude + (dy / R_EARTH) * (180.0 / PI);
        let new_longitude =
            self.longitude + (dx / R_EARTH) * (180.0 / PI) / (self.latitude * (PI / 180.0));
        let new_latitude = new_latitude.clamp(-90.0, 90.0);
        let new_longitude = new_longitude.clamp(-180.0, 180.0);
        Self {
            latitude: new_latitude,
            longitude: new_longitude,
        }
    }
}

impl From<Location> for LocationInternal {
    fn from(value: Location) -> Self {
        Self {
            latitude: value.latitude(),
            longitude: value.longitude(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::LocationInternal;

    const DEFAULT_DEGREES: f64 = 10.0;

    fn default_location() -> LocationInternal {
        LocationInternal {
            latitude: DEFAULT_DEGREES,
            longitude: DEFAULT_DEGREES,
        }
    }

    #[test]
    fn move_kilometers_no_movement() {
        let value = default_location().move_kilometers(0.0, 0.0);
        assert_eq!(value.latitude(), default_location().latitude());
        assert_eq!(value.longitude(), default_location().longitude());
    }

    #[test]
    fn move_kilometers_up() {
        let value = default_location().move_kilometers(1.0, 0.0);
        assert!(value.latitude() > default_location().latitude());
        assert_eq!(value.longitude(), default_location().longitude());
    }

    #[test]
    fn move_kilometers_down() {
        let value = default_location().move_kilometers(-1.0, 0.0);
        assert!(value.latitude() < default_location().latitude());
        assert_eq!(value.longitude(), default_location().longitude());
    }

    #[test]
    fn move_kilometers_left() {
        let value = default_location().move_kilometers(0.0, -1.0);
        assert_eq!(value.latitude(), default_location().latitude());
        assert!(value.longitude() < default_location().longitude());
    }

    #[test]
    fn move_kilometers_right() {
        let value = default_location().move_kilometers(0.0, 1.0);
        assert_eq!(value.latitude(), default_location().latitude());
        assert!(value.longitude() > default_location().longitude());
    }
}
