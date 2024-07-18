use api_client::models::Location;
use config::file::LocationConfig;
use rand::RngCore;
use rand_xoshiro::Xoshiro256PlusPlus;

pub trait LocationConfigUtils {
    fn generate_random_location(&self, deterministic_rng: Option<&mut Xoshiro256PlusPlus>) -> Location;
}

impl LocationConfigUtils for LocationConfig {
    fn generate_random_location(&self, mut deterministic_rng: Option<&mut Xoshiro256PlusPlus>) -> Location {
        let mut generate_u32 = || if let Some(rng) = deterministic_rng.as_mut() {
            rng.next_u32()
        } else {
            rand::random::<u32>()
        };

        let x_precent = generate_u32() as f64 / u32::MAX as f64;
        let y_precent = generate_u32() as f64 / u32::MAX as f64;

        let y_len = self.latitude_top_left - self.latitude_bottom_right;
        let x_len = self.longitude_bottom_right - self.longitude_top_left;

        let random_latitude = self.latitude_bottom_right + (y_len * y_precent);
        let random_longitude = self.longitude_bottom_right + (x_len * x_precent);

        Location::new(random_latitude, random_longitude)
    }
}
