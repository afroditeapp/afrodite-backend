use std::num::NonZeroU64;

use model::{ReceivedLikesIteratorSessionId, ReceivedLikesIteratorSessionIdInternal, UnixTime};


#[derive(Debug, Clone, Copy)]
pub enum ReceivedLikesIteratorState {
    /// First page should contain all likes made at `first_like_time` and
    /// first page of likes made at `first_like_time - 1` and before that.
    FirstPage {
        first_like_time: UnixTime,
    },
    NextPages {
        time_value: UnixTime,
        /// Zero page is already handled when state is [ReceivedLikesIteratorState::FirstPage]
        page: NonZeroU64,
    }
}

impl ReceivedLikesIteratorState {
    fn new() -> Self {
        Self::FirstPage { first_like_time: UnixTime::current_time() }
    }

    fn next(self) -> Self {
        match self {
            Self::FirstPage { first_like_time } => Self::NextPages {
                time_value: first_like_time.decrement(),
                page: NonZeroU64::MIN,
            },
            Self::NextPages { time_value, page } => Self::NextPages {
                time_value,
                page: page.saturating_add(1),
            }
        }
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
    pub fn reset(&mut self) -> ReceivedLikesIteratorSessionIdInternal {
        let id = ReceivedLikesIteratorSessionIdInternal::create_random();
        self.state = Some(InternalState {
            id,
            state: ReceivedLikesIteratorState::new(),
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
