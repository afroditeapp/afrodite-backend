use api_client::models::Location;
use config::file::LocationConfig;

pub trait LocationConfigUtils {
    fn generate_random_location(&self) -> Location;
    fn middle_of_area(&self) -> Location;
}

impl LocationConfigUtils for LocationConfig {
    fn generate_random_location(&self) -> Location {
        let x_precent = rand::random::<u32>() as f64 / u32::MAX as f64;
        let y_precent = rand::random::<u32>() as f64 / u32::MAX as f64;

        let y_len = self.latitude_top_left - self.latitude_bottom_right;
        let x_len = self.longitude_bottom_right - self.longitude_top_left;

        let random_latitude = self.latitude_bottom_right + (y_len * y_precent);
        let random_longitude = self.longitude_bottom_right + (x_len * x_precent);

        Location::new(random_latitude, random_longitude)
    }

    fn middle_of_area(&self) -> Location {
        let x_precent = 0.5;
        let y_precent = 0.5;

        let y_len = self.latitude_top_left - self.latitude_bottom_right;
        let x_len = self.longitude_bottom_right - self.longitude_top_left;

        let latitude = self.latitude_bottom_right + (y_len * y_precent);
        let longitude = self.longitude_bottom_right + (x_len * x_precent);

        Location::new(latitude, longitude)
    }
}
