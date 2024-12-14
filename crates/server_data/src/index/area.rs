use model_server_data::LocationIndexKey;
use rand::Rng;

#[derive(Debug, Clone, Default)]
pub struct LocationIndexArea {
    pub top_left: LocationIndexKey,
    pub bottom_right: LocationIndexKey,
    pub profile_location: LocationIndexKey,
}

impl LocationIndexArea {
    pub fn max_area(
        profile_location: LocationIndexKey,
        width: u16,
        height: u16,
    ) -> Self {
        Self {
            top_left: LocationIndexKey { y: 0, x: 0 },
            bottom_right: LocationIndexKey { y: height - 1, x: width - 1 },
            profile_location,
        }
    }

    pub fn index_iterator_start_location(&self, random: bool) -> LocationIndexKey {
        if random {
            let y = rand::thread_rng().gen_range(self.top_left.y..=self.bottom_right.y);
            let x = rand::thread_rng().gen_range(self.top_left.x..=self.bottom_right.x);
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
}
