//! Index for profiles
//!
//! LocationIndex
//!
//! Idea is to make matrix which has up-down lookup with atomic u16 values.
//! Those atomic values represents matrix indexes.
//!
//! Perhaps left-right lookup could be implemented as well??
//! Yes, it should be possible. Then there will be for atomic values in one cell.
//! Figure out first the up-down lookup.
//!
//! Best to use u16 for atomic numbers, so algorithm will be easier.
//! Matrix index numbers will fit to u16.
//!
//! Matrix cell should contain boolean which represents is there some profile in it.
//!
//! Initialization should happen so that border values of matrix should be used.
//!
//! Only one writer allowed at one time.
//!
//! No locks needed.
//!
//! Matrix indexes are used like a key for HashMap<(u16,u16), Vec<AccountId>>

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
}

impl LocationIndex {
    pub fn new(width: IndexSize, height: IndexSize) -> Self {
        let size = (width.get() as usize) * (height.get() as usize);
        let mut data = Vec::with_capacity(size);
        data.resize_with(size, || CellData::new(width.value, height.value));
        let storage = VecStorage::new(Dyn(height.get() as usize), Dyn(width.get() as usize), data);
        Self {
            data: DMatrix::from_data(storage),
        }
    }

    pub fn data(&self) -> &DMatrix<CellData> {
        &self.data
    }
}

impl Debug for LocationIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LocationIndex")
    }
}

pub trait ReadIndex {
    type C: CellDataProvider;

    fn get_cell_data(&self, x: usize, y: usize) -> Option<&Self::C>;

    /// Index width. Greater than zero.
    fn width(&self) -> usize;

    /// Index height. Greater than zero.
    fn height(&self) -> usize;

    /// Last y-axis index.
    fn last_row_index(&self) -> usize {
        self.height() - 1
    }

    /// Last x-axis index.
    fn last_column_index(&self) -> usize {
        self.width() - 1
    }
}

impl <T: AsRef<LocationIndex>> ReadIndex for T {
    type C = CellData;
    fn get_cell_data(&self, x: usize, y: usize) -> Option<&Self::C> {
        self.as_ref().data().get((y, x))
    }

    /// Index width. Greater than zero.
    fn width(&self) -> usize {
        self.as_ref().data().ncols()
    }

    /// Index height. Greater than zero.
    fn height(&self) -> usize {
        self.as_ref().data().nrows()
    }
}

impl AsRef<LocationIndex> for LocationIndex {
    fn as_ref(&self) -> &LocationIndex {
        self
    }
}
