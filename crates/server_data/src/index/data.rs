
use std::{fmt::Debug, num::NonZeroU16};

use model_server_data::{CellData, CellDataProvider};
use nalgebra::{DMatrix, Dyn, VecStorage};

/// Max width or height for index is 0x8000, which makes possible
/// to use u15 values for indexing the matrix.
/// The u15 values are stored in [CellData].
/// Min value is 3 as index border is reserved to be empty.
pub struct IndexSize {
    value: NonZeroU16,
}

impl IndexSize {
    const MIN_SIZE: u16 = 3;
    const MAX_SIZE: u16 = 0x8000;

    /// Panics if value is less than 3 and larger than 0x8000.
    pub fn new(value: NonZeroU16) -> Self {
        if value.get() < Self::MIN_SIZE {
            panic!("Min index width or height is {}", Self::MIN_SIZE);
        }
        if value.get() > Self::MAX_SIZE {
            panic!("Max index width or height is {}", Self::MAX_SIZE);
        }
        Self {
            value,
        }
    }

    fn get(&self) -> u16 {
        self.value.get()
    }
}

impl TryFrom<u16> for IndexSize {
    type Error = String;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let non_zero = TryInto::<NonZeroU16>::try_into(value).map_err(|e| e.to_string())?;
        if value < Self::MIN_SIZE {
            Err(format!("Min index width or height is {}", Self::MIN_SIZE))
        } else if value > Self::MAX_SIZE {
            Err(format!("Max index width or height is {}", Self::MAX_SIZE))
        } else {
            Ok(Self::new(non_zero))
        }
    }
}

/// Origin (0,0) = (y, x) is at top left corner.
pub struct LocationIndex {
    data: DMatrix<CellData>,
    indexes: IndexNumbers,
}

impl LocationIndex {
    pub fn new(width: IndexSize, height: IndexSize) -> Self {
        let size = (width.get() as usize) * (height.get() as usize);
        let mut data = Vec::with_capacity(size);
        data.resize_with(size, || CellData::new(width.value, height.value));
        let storage = VecStorage::new(Dyn(height.get() as usize), Dyn(width.get() as usize), data);
        Self {
            data: DMatrix::from_data(storage),
            indexes: IndexNumbers::new(width, height),
        }
    }

    pub fn data(&self) -> &DMatrix<CellData> {
        &self.data
    }

    /// Index width. Greater than zero.
    pub fn width(&self) -> usize {
        self.data.ncols()
    }

    /// Index height. Greater than zero.
    pub fn height(&self) -> usize {
        self.data.nrows()
    }
}

impl Debug for LocationIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LocationIndex")
    }
}

pub trait ReadIndex {
    type C: CellDataProvider;
    fn get_cell_data(&self, x: u16, y: u16) -> Option<&Self::C>;
    fn last_x_index(&self) -> u16;
    fn last_y_index(&self) -> u16;
    fn last_profile_area_x_index(&self) -> u16;
    fn last_profile_area_y_index(&self) -> u16;
}

impl <T: AsRef<LocationIndex>> ReadIndex for T {
    type C = CellData;

    fn get_cell_data(&self, x: u16, y: u16) -> Option<&Self::C> {
        self.as_ref().data().get((y.into(), x.into()))
    }

    fn last_x_index(&self) -> u16 {
        self.as_ref().indexes.last_x_index
    }

    fn last_y_index(&self) -> u16 {
        self.as_ref().indexes.last_y_index
    }

    fn last_profile_area_x_index(&self) -> u16 {
        self.as_ref().indexes.last_profile_area_x_index
    }

    fn last_profile_area_y_index(&self) -> u16 {
        self.as_ref().indexes.last_profile_area_y_index
    }
}

impl AsRef<LocationIndex> for LocationIndex {
    fn as_ref(&self) -> &LocationIndex {
        self
    }
}

struct IndexNumbers {
    last_x_index: u16,
    last_y_index: u16,
    last_profile_area_x_index: u16,
    last_profile_area_y_index: u16,
}

impl IndexNumbers {
    fn new(width: IndexSize, height: IndexSize) -> Self {
        Self {
            last_x_index: width.get() - 1,
            last_y_index: height.get() - 1,
            last_profile_area_x_index: width.get() - 2,
            last_profile_area_y_index: height.get() - 2,
        }
    }
}
