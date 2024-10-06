use model::{MatchId, MatchesIteratorSessionId, MatchesIteratorSessionIdInternal};

use super::db_iterator::DbIteratorState;

#[derive(Debug, Clone, Copy)]
pub struct MatchesIteratorState {
    pub id_at_reset: MatchId,
    db_iterator: DbIteratorState,
}

impl MatchesIteratorState {
    fn new(id_at_reset: MatchId) -> Self {
        Self {
            id_at_reset,
            db_iterator: DbIteratorState::new(),
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
}

#[derive(Debug, Clone, Copy)]
struct InternalState {
    id: MatchesIteratorSessionIdInternal,
    state: MatchesIteratorState,
}

#[derive(Debug, Default)]
pub struct MatchesIterator {
    state: Option<InternalState>,
}

impl MatchesIterator {
    pub fn reset(
        &mut self,
        match_id: MatchId,
    ) -> MatchesIteratorSessionIdInternal {
        let id = MatchesIteratorSessionIdInternal::create_random();
        self.state = Some(InternalState {
            id,
            state: MatchesIteratorState::new(match_id),
        });
        id
    }

    /// If return value is None, then reset the iterator and try again with
    /// new session ID.
    pub fn get_and_increment(&mut self, id: MatchesIteratorSessionId) -> Option<MatchesIteratorState> {
        let mut state = self.state?;
        let current_id: MatchesIteratorSessionId = state.id.into();
        if current_id != id {
            return None;
        }
        let current = state.state;
        state.state = state.state.next();
        self.state = Some(state);
        Some(current)
    }
}
