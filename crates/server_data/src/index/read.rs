
use std::fmt::Debug;

use model_server_data::{CellDataProvider, CellState, LocationIndexKey};
use tracing::error;

use super::{area::LocationIndexArea, data::ReadIndex};

// Finland's area is 338 462 square kilometer, so this is most likely
// good enough value as the iterator does not go all squares one by one.
const INDEX_ITERATOR_COUNT_LIMIT: u32 = 350_000;

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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::index::{data::LocationIndex, write::IndexUpdater};

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
            self.data()[(y as usize, x as usize)].state()
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
}
