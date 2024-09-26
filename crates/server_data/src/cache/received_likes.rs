use std::num::NonZeroU64;

use model::{ReceivedLikesIteratorSessionId, ReceivedLikesIteratorSessionIdInternal, UnixTime};


#[derive(Debug, Clone, Copy)]
pub enum ReceivedLikesIteratorState {
    /// First page should contain all likes made at `first_like_time` and
    /// first page of likes made at `first_like_time - 1` and before that.
    FirstPage {
        first_like_time: UnixTime,
        /// Reset time (first_like_time) from previous iterator reset
        reset_time_previous: Option<UnixTime>,
    },
    NextPages {
        time_value: UnixTime,
        /// Zero page is already handled when state is [ReceivedLikesIteratorState::FirstPage]
        page: NonZeroU64,
        /// Reset time (first_like_time) from previous iterator reset
        reset_time_previous: Option<UnixTime>,
    }
}

impl ReceivedLikesIteratorState {
    fn new(reset_time: UnixTime, previous_reset_time: Option<UnixTime>) -> Self {
        Self::FirstPage {
            first_like_time: reset_time,
            reset_time_previous: previous_reset_time,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::FirstPage { first_like_time, reset_time_previous } => Self::NextPages {
                time_value: first_like_time.decrement(),
                page: NonZeroU64::MIN,
                reset_time_previous,
            },
            Self::NextPages { time_value, page, reset_time_previous } => Self::NextPages {
                time_value,
                page: page.saturating_add(1),
                reset_time_previous,
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
    pub fn reset(
        &mut self,
        reset_time: UnixTime,
        reset_time_previous: Option<UnixTime>,
    ) -> ReceivedLikesIteratorSessionIdInternal {
        let id = ReceivedLikesIteratorSessionIdInternal::create_random();
        self.state = Some(InternalState {
            id,
            state: ReceivedLikesIteratorState::new(reset_time, reset_time_previous),
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
