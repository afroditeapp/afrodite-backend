use std::num::{NonZeroU8, NonZeroU16};

use config::file::LocationConfig;
use model_server_data::CellData;

use super::coordinates::{CoordinateManager, ZOOM_LEVEL_AND_TILE_LENGTH};

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
        let size = format!("Location index size: {width}x{height}, ");
        let bytes = format!("bytes: {}, ", format_size_in_bytes(byte_count));
        let zoom = format!("zoom: {}, ", coordinates.zoom_level());
        let len = format!(
            "tile side length: {:.2} km",
            coordinates.tile_side_length_km()
        );
        if whitespace_padding {
            format!("{size:<35}{bytes:<20}{zoom:<10}{len}",)
        } else {
            format!("{size}{bytes}{zoom}{len}",)
        }
    }

    pub fn create_all(&self) -> String {
        let mut info = String::new();
        for (_, tile_length) in ZOOM_LEVEL_AND_TILE_LENGTH {
            let tile_length_u64 = *tile_length as u64;
            let converted =
                TryInto::<u8>::try_into(tile_length_u64).and_then(TryInto::<NonZeroU8>::try_into);
            match converted {
                Ok(length) => {
                    info.push_str(&self.create_one_internal(length, true));
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
    format!("{size:.2} {unit}")
}
