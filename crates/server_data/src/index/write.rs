
use std::sync::Arc;

use model_server_data::{CellDataProvider, LocationIndexKey};
use tracing::error;

use super::data::{LocationIndex, ReadIndex};


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
        if self.index.data()[key].profiles() {
            return;
        }

        if key.x == 0 || key.x == self.index.last_x_index() ||
            key.y == 0 || key.y == self.index.last_y_index() {
                // This should not happen as profile location coordinates
                // are clamped to correct area.
                error!("Marking location index border area cell to have profile is not allowed");
                return;
            }

        self.index.data()[key].set_profiles(true);

        // Update right side of row
        for c in self.index.data().row(key.y()).iter().skip(key.x() + 1) {
            c.set_next_left(key.x());

            if c.profiles() {
                break;
            }
        }

        // Update left side of row
        for c in self
            .index
            .data()
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
        for c in self.index.data().column(key.x()).iter().skip(key.y() + 1) {
            c.set_next_up(key.y());

            if c.profiles() {
                break;
            }
        }

        // Update top side of column
        for c in self
            .index
            .data()
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
        if !self.index.data()[key].profiles() {
            return;
        }

        let cell = &self.index.data()[key];
        cell.set_profiles(false);

        let next_right = cell.next_right();
        let next_left = cell.next_left();
        let next_up = cell.next_up();
        let next_down = cell.next_down();

        // Update right side of row
        for c in self.index.data().row(key.y()).iter().skip(key.x() + 1) {
            c.set_next_left(next_left);

            if c.profiles() {
                break;
            }
        }

        // Update left side of row
        for c in self
            .index
            .data()
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
        for c in self.index.data().column(key.x()).iter().skip(key.y() + 1) {
            c.set_next_up(next_up);

            if c.profiles() {
                break;
            }
        }

        // Update top side of column
        for c in self
            .index
            .data()
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

    fn index_for_updater() -> LocationIndex {
        LocationIndex::new(3.try_into().unwrap(), 3.try_into().unwrap())
    }

    #[test]
    fn simple_index_update() {
        let index: Arc<_> = index_for_updater().into();
        let mut updater = IndexUpdater::new(index.clone());
        updater.flag_cell_to_have_profiles(LocationIndexKey { x: 1, y: 1 });

        let test_cell = |key: (usize, usize), up: usize, down: usize, left: usize, right: usize| {
            assert!(index.data()[key].next_up() == up);
            assert!(index.data()[key].next_down() == down);
            assert!(index.data()[key].next_left() == left);
            assert!(index.data()[key].next_right() == right);
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
            assert!(index.data()[key].next_up() == up);
            assert!(index.data()[key].next_down() == down);
            assert!(index.data()[key].next_left() == left);
            assert!(index.data()[key].next_right() == right);
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
