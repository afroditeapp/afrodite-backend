use model::{ReceivedLikeId, ReceivedLikesIteratorSessionId, ReceivedLikesIteratorSessionIdInternal};

use super::db_iterator::DbIteratorState;

#[derive(Debug, Clone, Copy)]
pub struct ReceivedLikesIteratorState {
    pub id_at_reset: ReceivedLikeId,
    db_iterator: DbIteratorState,
    /// Received like ID value from previous iterator reset
    pub id_at_reset_previous: Option<ReceivedLikeId>,
}

impl ReceivedLikesIteratorState {
    fn new(id_at_reset: ReceivedLikeId, id_at_reset_previous: Option<ReceivedLikeId>) -> Self {
        Self {
            id_at_reset,
            db_iterator: DbIteratorState::new(),
            id_at_reset_previous,
        }
    }

    fn next(self) -> Self {
        Self {
            id_at_reset: self.id_at_reset,
            db_iterator: self.db_iterator.next(),
            id_at_reset_previous: self.id_at_reset_previous,
        }
    }

    pub fn page(&self) -> u64 {
        self.db_iterator.page
    }
}

#[derive(Debug, Clone, Copy)]
struct InternalState {
    id: ReceivedLikesIteratorSessionIdInternal,
    state: ReceivedLikesIteratorState,
}

#[derive(Debug, Default)]
pub struct ReceivedLikesIterator {
    state: Option<InternalState>,
}

impl ReceivedLikesIterator {
    pub fn reset(
        &mut self,
        id_at_reset: ReceivedLikeId,
        id_at_reset_previous: Option<ReceivedLikeId>,
    ) -> ReceivedLikesIteratorSessionIdInternal {
        let id = ReceivedLikesIteratorSessionIdInternal::create_random();
        self.state = Some(InternalState {
            id,
            state: ReceivedLikesIteratorState::new(id_at_reset, id_at_reset_previous),
        });
        id
    }

    /// If return value is None, then reset the iterator and try again with
    /// new session ID.
    pub fn get_and_increment(&mut self, id: ReceivedLikesIteratorSessionId) -> Option<ReceivedLikesIteratorState> {
        let mut state = self.state?;
        let current_id: ReceivedLikesIteratorSessionId = state.id.into();
        if current_id != id {
            return None;
        }
        let current = state.state;
        state.state = state.state.next();
        self.state = Some(state);
        Some(current)
    }
}
