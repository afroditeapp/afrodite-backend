use std::{collections::HashMap, sync::Arc};

use model::ClientVersion;
use tokio::sync::Mutex;

#[derive(Debug)]
struct State {
    pub versions: HashMap<ClientVersion, i64>,
}

#[derive(Debug)]
pub struct ClientVersionTracker {
    state: Arc<Mutex<State>>,
}

impl Clone for ClientVersionTracker {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl ClientVersionTracker {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                versions: HashMap::new(),
            }))
        }
    }

    pub async fn get_current_state_and_reset(&self) -> HashMap<ClientVersion, i64> {
        let mut lock = self.state.lock().await;
        std::mem::take(&mut lock.versions)
    }

    pub async fn track_version(&self, version: ClientVersion) {
        let mut lock = self.state.lock().await;
        if let Some(value) = lock.versions.get_mut(&version) {
            *value += 1;
        } else {
            lock.versions.insert(version, 1);
        }
    }
}
