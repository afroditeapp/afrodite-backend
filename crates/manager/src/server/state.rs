//! State storage
//!

use super::mount::MountState;

pub struct State {
    pub mount_state: MountState,
}

impl State {
    pub fn new() -> Self {
        Self {
            mount_state: MountState::new(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

pub struct StateStorage {
    state: tokio::sync::Mutex<State>,
}

impl StateStorage {
    pub fn new() -> Self {
        Self {
            state: tokio::sync::Mutex::new(State::new()),
        }
    }

    pub async fn get<T>(&self, action: impl Fn(&State) -> T) -> T {
        let state = self.state.lock().await;
        action(&state)
    }

    pub async fn modify<T>(&self, action: impl Fn(&mut State) -> T) -> T {
        let mut state = self.state.lock().await;
        action(&mut state)
    }
}

impl Default for StateStorage {
    fn default() -> Self {
        Self::new()
    }
}
