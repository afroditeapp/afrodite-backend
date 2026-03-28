use std::sync::Arc;

use simple_backend::perf::websocket::{self, ConnectionTracker};
use tokio::sync::Notify;

#[derive(Clone)]
pub struct AdminBotStatusManagerData {
    notify: Arc<Notify>,
}

impl AdminBotStatusManagerData {
    pub(crate) fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn create_tracker(&self) -> AdminBotStatusTracker {
        AdminBotStatusTracker::new(self.notify.clone())
    }

    pub fn trigger_update(&self) {
        self.notify.notify_one();
    }

    pub async fn wait_update_trigger(&self) {
        self.notify.notified().await;
    }
}

pub struct AdminBotStatusTracker {
    counter: Option<ConnectionTracker>,
    notify: Arc<Notify>,
}

impl AdminBotStatusTracker {
    fn new(notify: Arc<Notify>) -> Self {
        let tracker = Some(websocket::AdminBotConnections::create().into());
        notify.notify_one();
        Self {
            counter: tracker,
            notify,
        }
    }
}

impl Drop for AdminBotStatusTracker {
    fn drop(&mut self) {
        drop(self.counter.take());
        self.notify.notify_one();
    }
}
