use model_server_data::{LocationIndexKey, LocationInternal};
use rand::Rng;

use super::{data::ReadIndex, CoordinateManager};

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
    area_inner: Option<IndexArea>,
    area_outer: IndexArea,
    /// This is not on the empty border area of the location index.
    profile_location: LocationIndexKey,
}

impl LocationIndexArea {
    pub fn new(
        area_inner: Option<IndexArea>,
        area_outer: IndexArea,
        mut profile_location: LocationIndexKey,
        index: &impl ReadIndex,
    ) -> Self {
        profile_location = LocationIndexKey {
            x: profile_location.x.clamp(1, (index.width() - 2) as u16),
            y: profile_location.y.clamp(1, (index.height() - 2) as u16),
        };
        Self {
            area_inner,
            area_outer,
            profile_location,
        }
    }

    pub fn max_area(
        profile_location: LocationIndexKey,
        index: &impl ReadIndex,
    ) -> Self {
        Self::new(
            None,
            IndexArea {
                top_left: LocationIndexKey { x: 0, y: 0 },
                bottom_right: LocationIndexKey {
                    x: (index.width() - 1) as u16,
                    y: (index.height() - 1) as u16,
                },
            },
            profile_location,
            index,
        )
    }

    pub fn index_iterator_start_location(&self, random: bool) -> LocationIndexKey {
        if random {
            let x = rand::thread_rng().gen_range(self.area_outer.top_left.x..=self.area_outer.bottom_right.x);
            let y = rand::thread_rng().gen_range(self.area_outer.top_left.y..=self.area_outer.bottom_right.y);
            LocationIndexKey {
                x,
                y,
            }
        } else {
            self.profile_location
        }
    }

    pub fn with_max_area(
        &self,
        index: &impl ReadIndex,
    ) -> Self {
        Self::max_area(
            self.profile_location,
            index,
        )
    }

    pub fn area_inner(&self) -> Option<&IndexArea> {
        self.area_inner.as_ref()
    }

    pub fn area_outer(&self) -> &IndexArea {
        &self.area_outer
    }

    /// This is not on the empty border area of the location index.
    pub fn profile_location(&self) -> LocationIndexKey {
        self.profile_location
    }
}
