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

use std::{fmt::Debug, num::NonZeroU16, sync::Arc};

use model_server_data::{CellData, CellDataProvider, CellState, LocationIndexKey};
use nalgebra::{DMatrix, Dyn, VecStorage};
use tracing::error;

use super::area::LocationIndexArea;

// Finland's area is 338 462 square kilometer, so this is most likely
// good enough value as the iterator does not go all squares one by one.
const INDEX_ITERATOR_COUNT_LIMIT: u32 = 350_000;

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

#[derive(Debug, Copy, Clone, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Default)]
struct IndexLimitCoordinates {
    x: isize,
    y: isize,
}

impl IndexLimitCoordinates {
    fn new(key: LocationIndexKey) -> Self {
        Self {
            x: key.x as isize,
            y: key.y as isize,
        }
    }
}

impl From<LocationIndexKey> for IndexLimitCoordinates {
    fn from(value: LocationIndexKey) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Default)]
struct IndexLimit {
    top_left: IndexLimitCoordinates,
    bottom_right: IndexLimitCoordinates,
}

#[derive(Debug, Clone, Default)]
struct IndexLimitInner(pub IndexLimit);

impl IndexLimitInner {
    fn is_inside(&self, x: isize, y: isize) -> bool {
        // Use > and < operators to make index cells at the limit border
        // accessible when min and max distance limits have the same value.
        x > self.0.top_left.x  &&
        x < self.0.bottom_right.x &&
        y > self.0.top_left.y &&
        y < self.0.bottom_right.y
    }
}

#[derive(Debug, Clone, Default)]
struct IndexLimitOuter(pub IndexLimit);

impl IndexLimitOuter {
    fn is_outside(&self, x: isize, y: isize) -> bool {
        (x < self.0.top_left.x || x > self.0.bottom_right.x) &&
        (y < self.0.top_left.y || y > self.0.bottom_right.y)
    }
}

/// Updates when round changes.
#[derive(Debug, Clone)]
struct CurrentMaxIndexes {
    /// Top side max area index.
    top: isize,
    /// Bottom side max area index.
    bottom: isize,
    /// Left side max area index.
    left: isize,
    /// Right side max area index.
    right: isize,
}

/// State which does not change after iterator is created.
#[derive(Debug, Clone)]
struct InitialState {
    y: isize,
    x: isize,
    limit_inner: Option<IndexLimitInner>,
    limit_outer: IndexLimitOuter,
}

enum CreateRoundStateResult {
    Continue(RoundState),
    AllIterated,
}

enum MoveForwardResult {
    /// Nothing happened. Move to next round.
    Completed,
    CheckProfilesAndMoveForward,
}

/// Cursor round specific state.
#[derive(Debug, Clone)]
struct RoundState {
    x: isize,
    y: isize,
    /// Move direction for cursor
    direction: Direction,
    max_indexes: CurrentMaxIndexes,
}

impl RoundState {
    /// Round number must be 0 or greater.
    ///
    /// When round number is 0 the [Self::move_forward] returns directly
    /// [RoundResult::Completed].
    fn create(
        initial: &InitialState,
        round: isize,
        index: &impl ReadIndex,
    ) -> CreateRoundStateResult {
        let top = initial.y - round;
        let bottom = initial.y + round;
        let left = initial.x - round;
        let right = initial.x + round;

        if initial.limit_outer.is_outside(left, top) &&
            initial.limit_outer.is_outside(right, bottom) {
            return CreateRoundStateResult::AllIterated;
        }

        let max_indexes = CurrentMaxIndexes {
            top: top.max(0),
            bottom: bottom.min(index.last_row_index() as isize),
            left: left.max(0),
            right: right.min(index.last_column_index() as isize),
        };

        let state = Self {
            x: max_indexes.right,
            y: if round == 0 {
                max_indexes.top
            } else {
                max_indexes.top + 1
            },
            max_indexes,
            direction: Direction::Down,
        };

        CreateRoundStateResult::Continue(state)
    }

    fn current_position(&self) -> LocationIndexKey {
        LocationIndexKey { y: self.y as u16, x: self.x as u16 }
    }

    fn is_round_complete(&self) -> bool {
        self.max_indexes.right == self.x
            && self.max_indexes.top == self.y
            && self.direction == Direction::Down
    }

    /// Move position according to cell next index information.
    ///
    /// Returns error if there is no next new position.
    fn move_forward(&mut self, state: CellState) -> MoveForwardResult {
        if self.is_round_complete() {
            return MoveForwardResult::Completed;
        }

        match self.direction {
            Direction::Up => {
                self.y = state.next_up().max(self.max_indexes.top);
                if self.y == self.max_indexes.top {
                    self.direction = Direction::Right;
                }
            }
            Direction::Down => {
                self.y = state.next_down().min(self.max_indexes.bottom);
                if self.y == self.max_indexes.bottom {
                    self.direction = Direction::Left;
                }
            }
            Direction::Left => {
                self.x = state.next_left().max(self.max_indexes.left);
                if self.x == self.max_indexes.left {
                    self.direction = Direction::Up;
                }
            }
            Direction::Right => {
                self.x = state.next_right().min(self.max_indexes.right);
                if self.x == self.max_indexes.right {
                    self.direction = Direction::Down;
                }
            }
        }

        MoveForwardResult::CheckProfilesAndMoveForward
    }
}

/// Iterator for location index
///
/// Start moving cursor from one cell and enlarge area clockwise.
/// Each cursor round starts from one cell down of top right corner of the specific
/// cursor round. The rounds ends to the top right corner of the round.
///
/// Border area of the index must be empty from profiles as cursor
/// can be on that area multiple times.
#[derive(Debug, Clone)]
pub struct LocationIndexIteratorState {
    initial_state: InitialState,
    round: RoundState,
    /// How many rounds cursor has moved. Checking initial position counts one.
    current_round: isize,
    completed: bool,
}

impl LocationIndexIteratorState {
    pub fn completed() -> Self {
        Self {
            initial_state: InitialState {
                x: 0,
                y: 0,
                limit_inner: None,
                limit_outer: IndexLimitOuter::default(),
            },
            round: RoundState {
                x: 0,
                y: 0,
                direction: Direction::Down,
                max_indexes: CurrentMaxIndexes {
                    top: 0,
                    bottom: 0,
                    left: 0,
                    right: 0,
                }
            },
            current_round: 0,
            completed: true,
        }
    }

    pub fn new(
        area: &LocationIndexArea,
        random_start_position: bool,
        index: &impl ReadIndex,
    ) -> Self {
        let start_position = area.index_iterator_start_location(random_start_position);
        let x = start_position.x as isize;
        let y = start_position.y as isize;
        let initial_state = InitialState {
            x,
            y,
            limit_inner: area.area_inner().as_ref().map(|a| IndexLimitInner(IndexLimit {
                top_left: a.top_left.into(),
                bottom_right: a.bottom_right.into(),
            })),
            limit_outer: IndexLimitOuter(IndexLimit {
                top_left: area.area_outer().top_left.into(),
                bottom_right: area.area_outer().bottom_right.into(),
            }),
        };
        match RoundState::create(&initial_state, 0, index) {
            CreateRoundStateResult::AllIterated => Self::completed(),
            CreateRoundStateResult::Continue(round) =>
                Self {
                    round,
                    initial_state,
                    current_round: 0,
                    completed: false,
                }
        }
    }

    pub fn into_iterator<T: ReadIndex>(self, reader: T) -> LocationIndexIterator<T> {
        LocationIndexIterator::new(self, reader)
    }

    pub fn get_current_position_if_contains_profiles(&self, state: &CellState) -> Option<LocationIndexKey> {
        if !state.profiles() {
            return None;
        }

        // Make area inside inner limit appear empty
        if let Some(limit) = &self.initial_state.limit_inner {
            if limit.is_inside(self.round.x, self.round.y) {
                return None;
            }
        }

        // Make area outside outer limit appear empty
        if self.initial_state.limit_outer.is_outside(self.round.x, self.round.y) {
            return None;
        }

        Some(self.round.current_position())
    }

    /// Get next cell where are profiles.
    fn next(&mut self, index: &impl ReadIndex) -> Option<LocationIndexKey> {
        if self.completed {
            return None;
        }

        let mut count_iterations = 0;

        loop {
            let state = self.current_cell_state(index);
            let Some(state) = state else {
                // This should not happen as all coordinates should point to
                // a valid location.
                error!("Out of bounds location index access detected");
                self.completed = true;
                return None;
            };
            let data_position = self.get_current_position_if_contains_profiles(&state);

            match self.round.move_forward(state) {
                MoveForwardResult::CheckProfilesAndMoveForward => (),
                MoveForwardResult::Completed => {
                    self.current_round += 1;
                     match RoundState::create(&self.initial_state, self.current_round, index) {
                        CreateRoundStateResult::AllIterated => {
                            self.completed = true;
                            return data_position;
                        }
                        CreateRoundStateResult::Continue(round) =>
                            self.round = round,
                    }
                }
            }

            if data_position.is_some() {
                return data_position;
            }

            if count_iterations >= INDEX_ITERATOR_COUNT_LIMIT {
                error!(
                    "Location index iterator max count {} reached. This is a bug.",
                    count_iterations,
                );
                self.completed = true;
                return None;
            } else {
                count_iterations += 1;
            }
        }
    }

    fn current_cell_state(&self, index: &impl ReadIndex) -> Option<CellState> {
        self.current_cell(index)
            .map(|cell| cell.state())
    }

    fn current_cell<'a, A: ReadIndex>(&self, index: &'a A) -> Option<&'a A::C> {
        let x = self.round.x.try_into().ok()?;
        let y = self.round.y.try_into().ok()?;
        index.get_cell_data(x, y)
    }
}

impl <T: ReadIndex> From<LocationIndexIterator<T>> for LocationIndexIteratorState {
    fn from(value: LocationIndexIterator<T>) -> Self {
        value.state
    }
}

pub struct LocationIndexIterator<T: ReadIndex> {
    state: LocationIndexIteratorState,
    area: T,
}

impl <T: ReadIndex> LocationIndexIterator<T> {
    fn new(
        state: LocationIndexIteratorState,
        area: T,
    ) -> Self {
        Self {
            state,
            area,
        }
    }

    #[cfg(test)]
    /// Return next index key as (x, y) tuple.
    pub fn next_raw(&mut self) -> Option<(u16, u16)> {
        self.state.next(&self.area).map(|v| (v.x, v.y))
    }
}

impl <T: ReadIndex> Iterator for LocationIndexIterator<T> {
    type Item = LocationIndexKey;

    /// Get next cell where are profiles.
    ///
    /// If None then there is not any more cells with profiles.
    fn next(&mut self) -> Option<LocationIndexKey> {
        self.state.next(&self.area)
    }
}

/// Update index.
///
/// Create only one IndexUpdater as it modifies the LocationIndex.
pub struct IndexUpdater {
    index: Arc<LocationIndex>,
}

impl IndexUpdater {
    pub fn new(index: Arc<LocationIndex>) -> Self {
        Self { index }
    }

    pub fn flag_cell_to_have_profiles(&mut self, key: LocationIndexKey) {
        if self.index.data[key].profiles() {
            return;
        }

        if key.x == 0 || key.x == self.index.last_column_index() as u16 ||
            key.y == 0 || key.y == self.index.last_row_index() as u16 {
                // This should not happen as profile location coordinates
                // are clamped to correct area.
                error!("Marking location index border area cell to have profile is not allowed");
                return;
            }

        self.index.data[key].set_profiles(true);

        // Update right side of row
        for c in self.index.data.row(key.y()).iter().skip(key.x() + 1) {
            c.set_next_left(key.x());

            if c.profiles() {
                break;
            }
        }

        // Update left side of row
        for c in self
            .index
            .data
            .row(key.y())
            .iter()
            .rev()
            .skip(self.index.width() - key.x())
        {
            c.set_next_right(key.x());

            if c.profiles() {
                break;
            }
        }

        // Update bottom side of column
        for c in self.index.data.column(key.x()).iter().skip(key.y() + 1) {
            c.set_next_up(key.y());

            if c.profiles() {
                break;
            }
        }

        // Update top side of column
        for c in self
            .index
            .data
            .column(key.x())
            .iter()
            .rev()
            .skip(self.index.height() - key.y())
        {
            c.set_next_down(key.y());

            if c.profiles() {
                break;
            }
        }
    }

    pub fn remove_profile_flag_from_cell(&mut self, key: LocationIndexKey) {
        if !self.index.data[key].profiles() {
            return;
        }

        let cell = &self.index.data[key];
        cell.set_profiles(false);

        let next_right = cell.next_right();
        let next_left = cell.next_left();
        let next_up = cell.next_up();
        let next_down = cell.next_down();

        // Update right side of row
        for c in self.index.data.row(key.y()).iter().skip(key.x() + 1) {
            c.set_next_left(next_left);

            if c.profiles() {
                break;
            }
        }

        // Update left side of row
        for c in self
            .index
            .data
            .row(key.y())
            .iter()
            .rev()
            .skip(self.index.width() - key.x())
        {
            c.set_next_right(next_right);

            if c.profiles() {
                break;
            }
        }

        // Update bottom side of column
        for c in self.index.data.column(key.x()).iter().skip(key.y() + 1) {
            c.set_next_up(next_up);

            if c.profiles() {
                break;
            }
        }

        // Update top side of column
        for c in self
            .index
            .data
            .column(key.x())
            .iter()
            .rev()
            .skip(self.index.height() - key.y())
        {
            c.set_next_down(next_down);

            if c.profiles() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait IndexUtils: Sized {
        fn add_profile(self, x: u16, y: u16) -> Self;
        fn get(&self, x: u16, y: u16) -> CellState;
    }

    impl IndexUtils for Arc<LocationIndex> {
        fn add_profile(self, x: u16, y: u16) -> Self {
            let mut updater = IndexUpdater::new(self.clone());
            updater.flag_cell_to_have_profiles(LocationIndexKey { x, y });
            self
        }
        fn get(&self, x: u16, y: u16) -> CellState {
            self.data[(y as usize, x as usize)].state()
        }
    }

    fn index() -> Arc<LocationIndex> {
        Arc::new(LocationIndex::new(6.try_into().unwrap(), 11.try_into().unwrap()))
            .add_profile(1, 1)
            .add_profile(4, 1)
            .add_profile(1, 9)
            .add_profile(4, 9)
    }

    fn mirror_index() -> Arc<LocationIndex> {
        Arc::new(LocationIndex::new(11.try_into().unwrap(), 6.try_into().unwrap()))
            .add_profile(1, 1)
            .add_profile(9, 1)
            .add_profile(1, 4)
            .add_profile(9, 4)
    }

    fn max_area(x: u16, y: u16, index: &LocationIndex) -> LocationIndexArea {
        LocationIndexArea::max_area(
            LocationIndexKey { y, x },
            index,
        )
    }

    fn init_with_index(x: u16, y: u16) -> LocationIndexIterator<Arc<LocationIndex>> {
        let index = index();
        let area = max_area(x, y, &index);
        LocationIndexIterator::new(LocationIndexIteratorState::new(&area, false, &index), index.clone())
    }

    fn init_with_mirror_index(x: u16, y: u16) -> LocationIndexIterator<Arc<LocationIndex>> {
        let index = mirror_index();
        let area = max_area(x, y, &index);
        LocationIndexIterator::new(LocationIndexIteratorState::new(&area, false, &index), index)
    }

    #[test]
    fn top_left_profile() {
        let c = index().get(1, 1);
        assert!(c.next_up() == 0);
        assert!(c.next_down() == 9);
        assert!(c.next_left() == 0);
        assert!(c.next_right() == 4);
        assert!(c.profiles());
    }

    #[test]
    fn top_right_profile() {
        let c = index().get(4, 1);
        assert!(c.next_up() == 0);
        assert!(c.next_down() == 9);
        assert!(c.next_left() == 1);
        assert!(c.next_right() == 5);
        assert!(c.profiles());
    }

    #[test]
    fn bottom_left_profile() {
        let c = index().get(1, 9);
        assert!(c.next_up() == 1);
        assert!(c.next_down() == 10);
        assert!(c.next_left() == 0);
        assert!(c.next_right() == 4);
        assert!(c.profiles());
    }

    #[test]
    fn bottom_right_profile() {
        let c = index().get(4, 9);
        assert!(c.next_up() == 1);
        assert!(c.next_down() == 10);
        assert!(c.next_left() == 1);
        assert!(c.next_right() == 5);
        assert!(c.profiles());
    }

    #[test]
    fn iterator_from_top_left_profile() {
        let mut iter = init_with_index(1, 1);
        assert_eq!(iter.next_raw(), Some((1, 1)));
        assert_eq!(iter.next_raw(), Some((4, 1)));
        assert_eq!(iter.next_raw(), Some((4, 9)));
        assert_eq!(iter.next_raw(), Some((1, 9)));
        assert_eq!(iter.next_raw(), None);
    }

    #[test]
    fn iterator_from_top_right_profile() {
        let mut iter = init_with_index(4, 1);
        assert_eq!(iter.next_raw(), Some((4, 1)));
        assert_eq!(iter.next_raw(), Some((1, 1)));
        assert_eq!(iter.next_raw(), Some((4, 9)));
        assert_eq!(iter.next_raw(), Some((1, 9)));
        assert_eq!(iter.next_raw(), None);
    }

    #[test]
    fn iterator_from_bottom_right_profile() {
        let mut iter = init_with_index(4, 9);
        assert_eq!(iter.next_raw(), Some((4, 9)));
        assert_eq!(iter.next_raw(), Some((1, 9)));
        assert_eq!(iter.next_raw(), Some((1, 1)));
        assert_eq!(iter.next_raw(), Some((4, 1)));
        assert_eq!(iter.next_raw(), None);
    }

    #[test]
    fn iterator_from_bottom_left_profile() {
        let mut iter = init_with_index(1, 9);
        assert_eq!(iter.next_raw(), Some((1, 9)));
        assert_eq!(iter.next_raw(), Some((4, 9)));
        assert_eq!(iter.next_raw(), Some((1, 1)));
        assert_eq!(iter.next_raw(), Some((4, 1)));
        assert_eq!(iter.next_raw(), None);
    }

    #[test]
    fn mirror_index_iterator_from_top_left_profile() {
        let mut iter = init_with_mirror_index(1, 1);
        assert_eq!(iter.next_raw(), Some((1, 1)));
        assert_eq!(iter.next_raw(), Some((1, 4)));
        assert_eq!(iter.next_raw(), Some((9, 1)));
        assert_eq!(iter.next_raw(), Some((9, 4)));
        assert_eq!(iter.next_raw(), None);
    }

    #[test]
    fn mirror_index_iterator_from_top_right_profile() {
        let mut iter = init_with_mirror_index(9, 1);
        assert_eq!(iter.next_raw(), Some((9, 1)));
        assert_eq!(iter.next_raw(), Some((9, 4)));
        assert_eq!(iter.next_raw(), Some((1, 4)));
        assert_eq!(iter.next_raw(), Some((1, 1)));
        assert_eq!(iter.next_raw(), None);
    }

    #[test]
    fn mirror_index_iterator_from_bottom_right_profile() {
        let mut iter = init_with_mirror_index(9, 4);
        assert_eq!(iter.next_raw(), Some((9, 4)));
        assert_eq!(iter.next_raw(), Some((9, 1)));
        assert_eq!(iter.next_raw(), Some((1, 4)));
        assert_eq!(iter.next_raw(), Some((1, 1)));
        assert_eq!(iter.next_raw(), None);
    }

    #[test]
    fn mirror_index_iterator_from_bottom_left_profile() {
        let mut iter = init_with_mirror_index(1, 4);
        assert_eq!(iter.next_raw(), Some((1, 4)));
        assert_eq!(iter.next_raw(), Some((1, 1)));
        assert_eq!(iter.next_raw(), Some((9, 1)));
        assert_eq!(iter.next_raw(), Some((9, 4)));
        assert_eq!(iter.next_raw(), None);
    }

    // IndexUpdater

    fn index_for_updater() -> LocationIndex {
        LocationIndex::new(3.try_into().unwrap(), 3.try_into().unwrap())
    }

    #[test]
    fn simple_index_update() {
        let index: Arc<_> = index_for_updater().into();
        let mut updater = IndexUpdater::new(index.clone());
        updater.flag_cell_to_have_profiles(LocationIndexKey { x: 1, y: 1 });

        let test_cell = |key: (usize, usize), up: usize, down: usize, left: usize, right: usize| {
            assert!(index.data[key].next_up() == up);
            assert!(index.data[key].next_down() == down);
            assert!(index.data[key].next_left() == left);
            assert!(index.data[key].next_right() == right);
        };

        // Check middle column
        test_cell((0, 1), 0, 1, 0, 2);
        test_cell((1, 1), 0, 2, 0, 2);
        test_cell((2, 1), 1, 2, 0, 2);

        // Check middle row
        test_cell((1, 0), 0, 2, 0, 1);
        test_cell((1, 1), 0, 2, 0, 2);
        test_cell((1, 2), 0, 2, 1, 2);
    }

    #[test]
    fn simple_index_remove_test() {
        let index: Arc<_> = index_for_updater().into();
        let mut updater = IndexUpdater::new(index.clone());
        updater.flag_cell_to_have_profiles(LocationIndexKey { x: 1, y: 1 });
        updater.remove_profile_flag_from_cell(LocationIndexKey { x: 1, y: 1 });

        let test_cell = |key: (usize, usize), up: usize, down: usize, left: usize, right: usize| {
            assert!(index.data[key].next_up() == up);
            assert!(index.data[key].next_down() == down);
            assert!(index.data[key].next_left() == left);
            assert!(index.data[key].next_right() == right);
        };

        // Check middle column
        test_cell((0, 1), 0, 2, 0, 2);
        test_cell((1, 1), 0, 2, 0, 2);
        test_cell((2, 1), 0, 2, 0, 2);

        // Check middle row
        test_cell((1, 0), 0, 2, 0, 2);
        test_cell((1, 1), 0, 2, 0, 2);
        test_cell((1, 2), 0, 2, 0, 2);
    }
}
