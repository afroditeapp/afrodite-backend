use std::{collections::HashMap, sync::Arc};

use config::file::ClientVersionTrackingConfig;
use model::ClientVersion;
use tokio::sync::Mutex;

pub enum TrackingResult {
    Disabled,
    Invalid,
    Tracked,
}

#[derive(Debug)]
struct State {
    pub versions: HashMap<ClientVersion, i64>,
}

#[derive(Debug)]
pub struct ClientVersionTracker {
    state: Arc<Mutex<State>>,
    config: Option<ClientVersionTrackingConfig>,
}

impl Clone for ClientVersionTracker {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            config: self.config.clone(),
        }
    }
}

impl ClientVersionTracker {
    pub(crate) fn new(config: Option<ClientVersionTrackingConfig>) -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                versions: HashMap::new(),
            })),
            config,
        }
    }

    pub async fn get_current_state_and_reset(&self) -> HashMap<ClientVersion, i64> {
        let mut lock = self.state.lock().await;
        std::mem::take(&mut lock.versions)
    }

    pub async fn track_version(&self, version: ClientVersion) -> TrackingResult {
        let Some(config) = &self.config else {
            return TrackingResult::Disabled;
        };

        if !config.is_valid(version) {
            return TrackingResult::Invalid;
        }

        let mut lock = self.state.lock().await;
        if let Some(value) = lock.versions.get_mut(&version) {
            *value += 1;
        } else {
            lock.versions.insert(version, 1);
        }

        TrackingResult::Tracked
    }
}
