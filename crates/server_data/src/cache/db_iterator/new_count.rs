use model::NextNumberStorage;

use super::{DbIteratorState, IteratorSessionIdTrait, IteratorStartPoint};

/// Database iterator state with previous ID at reset info
/// which can be used for tracking "new item" status for returned
/// items.
#[derive(Debug, Clone, Copy)]
pub struct DbIteratorStateNewCount<T: IteratorStartPoint> {
    previous_id_at_reset: Option<T>,
    base: DbIteratorState<T>,
}

impl<T: IteratorStartPoint> DbIteratorStateNewCount<T> {
    fn new(id_at_reset: T, previous_id_at_reset: Option<T>) -> Self {
        Self {
            previous_id_at_reset,
            base: DbIteratorState::new(id_at_reset),
        }
    }

    fn next(self) -> Self {
        Self {
            previous_id_at_reset: self.previous_id_at_reset,
            base: self.base.next(),
        }
    }

    pub fn page(&self) -> u64 {
        self.base.page()
    }

    pub fn id_at_reset(&self) -> T {
        self.base.id_at_reset()
    }

    pub fn previous_id_at_reset(&self) -> Option<T> {
        self.previous_id_at_reset
    }
}

#[derive(Debug, Clone, Copy)]
struct DbIteratorStateWithSessionId<T: IteratorSessionIdTrait, U: IteratorStartPoint> {
    id: T,
    state: DbIteratorStateNewCount<U>,
}

#[derive(Debug)]
pub struct DbIteratorNewCount<T: IteratorSessionIdTrait, U: IteratorStartPoint> {
    session_id_storage: NextNumberStorage,
    state: Option<DbIteratorStateWithSessionId<T, U>>,
}

impl<T: IteratorSessionIdTrait, U: IteratorStartPoint> DbIteratorNewCount<T, U> {
    pub fn reset(
        &mut self,
        iterator_start_point: U,
        previous_iterator_start_point: Option<U>,
    ) -> T {
        let id = T::create(&mut self.session_id_storage);
        self.state = Some(DbIteratorStateWithSessionId {
            id,
            state: DbIteratorStateNewCount::new(
                iterator_start_point,
                previous_iterator_start_point,
            ),
        });
        id
    }

    /// If return value is None, then reset the iterator and try again with
    /// new session ID.
    pub fn get_and_increment<Id: Into<T>>(&mut self, id: Id) -> Option<DbIteratorStateNewCount<U>> {
        let mut state = self.state?;
        if state.id != id.into() {
            return None;
        }
        let current = state.state;
        state.state = state.state.next();
        self.state = Some(state);
        Some(current)
    }
}

impl<T: IteratorSessionIdTrait, U: IteratorStartPoint> Default for DbIteratorNewCount<T, U> {
    fn default() -> Self {
        Self {
            session_id_storage: NextNumberStorage::default(),
            state: None,
        }
    }
}
