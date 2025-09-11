use model::{NextNumberStorage, ReceivedLikeId};
use model_server_data::{
    NewsIteratorSessionIdInternal, PublicationId, ReceivedLikesIteratorSessionIdInternal,
};

pub mod new_count;

#[derive(Debug, Clone, Copy, Default)]
pub struct DbIteratorPageState {
    pub page: u64,
}

impl DbIteratorPageState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next(self) -> Self {
        Self {
            page: self.page.saturating_add(1),
        }
    }
}

pub trait IteratorStartPoint: Clone + Copy + Into<i64> {}

#[derive(Debug, Clone, Copy)]
pub struct DbIteratorState<T: IteratorStartPoint> {
    id_at_reset: T,
    db_iterator: DbIteratorPageState,
}

impl<T: IteratorStartPoint> DbIteratorState<T> {
    fn new(id_at_reset: T) -> Self {
        Self {
            id_at_reset,
            db_iterator: DbIteratorPageState::new(),
        }
    }

    fn next(self) -> Self {
        Self {
            id_at_reset: self.id_at_reset,
            db_iterator: self.db_iterator.next(),
        }
    }

    pub fn page(&self) -> u64 {
        self.db_iterator.page
    }

    pub fn id_at_reset(&self) -> T {
        self.id_at_reset
    }
}

pub trait IteratorSessionIdTrait: Clone + Copy + PartialEq {
    fn create(storage: &mut NextNumberStorage) -> Self;
}

#[derive(Debug, Clone, Copy)]
struct DbIteratorStateWithSessionId<T: IteratorSessionIdTrait, U: IteratorStartPoint> {
    id: T,
    state: DbIteratorState<U>,
}

#[derive(Debug)]
pub struct DbIterator<T: IteratorSessionIdTrait, U: IteratorStartPoint> {
    session_id_storage: NextNumberStorage,
    state: Option<DbIteratorStateWithSessionId<T, U>>,
}

impl<T: IteratorSessionIdTrait, U: IteratorStartPoint> DbIterator<T, U> {
    pub fn reset(&mut self, iterator_start_point: U) -> T {
        let id = T::create(&mut self.session_id_storage);
        self.state = Some(DbIteratorStateWithSessionId {
            id,
            state: DbIteratorState::new(iterator_start_point),
        });
        id
    }

    /// If return value is None, then reset the iterator and try again with
    /// new session ID.
    pub fn get_and_increment<Id: Into<T>>(&mut self, id: Id) -> Option<DbIteratorState<U>> {
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

impl<T: IteratorSessionIdTrait, U: IteratorStartPoint> Default for DbIterator<T, U> {
    fn default() -> Self {
        Self {
            session_id_storage: NextNumberStorage::default(),
            state: None,
        }
    }
}

// Account

impl IteratorSessionIdTrait for NewsIteratorSessionIdInternal {
    fn create(storage: &mut NextNumberStorage) -> Self {
        NewsIteratorSessionIdInternal::create(storage)
    }
}

impl IteratorStartPoint for PublicationId {}

// Chat

impl IteratorSessionIdTrait for ReceivedLikesIteratorSessionIdInternal {
    fn create(storage: &mut NextNumberStorage) -> Self {
        ReceivedLikesIteratorSessionIdInternal::create(storage)
    }
}

impl IteratorStartPoint for ReceivedLikeId {}
