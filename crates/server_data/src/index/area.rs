use model_server_data::{LocationIndexKey, LocationInternal};
use rand::Rng;

use super::CoordinateManager;

#[derive(Debug, Clone, Default)]
pub struct IndexArea {
    pub top_left: LocationIndexKey,
    pub bottom_right: LocationIndexKey
}

impl IndexArea {
    pub fn new(
        manager: &CoordinateManager,
        location: LocationInternal,
        distance: i64,
    ) -> Self {
        let distance = distance as f64;
        Self {
            top_left: manager.location_to_index_key(location.move_kilometers(distance, -distance)),
            bottom_right: manager.location_to_index_key(location.move_kilometers(-distance, distance)),
        }
    }

    pub fn max_area(
        width: u16,
        height: u16,
    ) -> Self {
        Self {
            top_left: LocationIndexKey { y: 0, x: 0 },
            bottom_right: LocationIndexKey { y: height - 1, x: width - 1 },
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct LocationIndexArea {
    pub area_inner: Option<IndexArea>,
    pub area_outer: IndexArea,
    pub profile_location: LocationIndexKey,
}

impl LocationIndexArea {
    pub fn max_area(
        profile_location: LocationIndexKey,
        width: u16,
        height: u16,
    ) -> Self {
        Self {
            area_inner: None,
            area_outer: IndexArea {
                top_left: LocationIndexKey { y: 0, x: 0 },
                bottom_right: LocationIndexKey { y: height - 1, x: width - 1 },
            },
            profile_location,
        }
    }

    pub fn index_iterator_start_location(&self, random: bool) -> LocationIndexKey {
        if random {
            let y = rand::thread_rng().gen_range(self.area_outer.top_left.y..=self.area_outer.bottom_right.y);
            let x = rand::thread_rng().gen_range(self.area_outer.top_left.x..=self.area_outer.bottom_right.x);
            LocationIndexKey {
                y,
                x,
            }
        } else {
            self.profile_location
        }
    }

    pub fn profile_location(&self) -> LocationIndexKey {
        self.profile_location
    }

    pub fn with_max_area(
        &self,
        width: u16,
        height: u16,
    ) -> Self {
        Self::max_area(
            self.profile_location,
            width,
            height,
        )
    }
}
