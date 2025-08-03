use std::sync::atomic::{AtomicBool, Ordering};

pub static BACKEND_DATA_RESET_STATE: DataResetState = DataResetState::new();

pub struct DataResetState {
    is_ongoing: AtomicBool,
}

impl DataResetState {
    const fn new() -> Self {
        Self {
            is_ongoing: AtomicBool::new(false),
        }
    }

    pub fn is_ongoing(&self) -> bool {
        self.is_ongoing.load(Ordering::Relaxed)
    }

    pub fn current_value_and_set_ongoing(&self) -> bool {
        self.is_ongoing.swap(true, Ordering::Relaxed)
    }
}
