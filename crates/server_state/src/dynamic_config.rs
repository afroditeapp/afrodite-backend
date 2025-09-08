use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::error;

pub enum DynamicConfigEvent {
    Reload,
}

pub struct DynamicConfigEventReceiver(pub Receiver<DynamicConfigEvent>);

pub struct DynamicConfigManagerData {
    sender: Sender<DynamicConfigEvent>,
    remote_bot_login: AtomicBool,
}

impl DynamicConfigManagerData {
    pub fn new() -> (Self, DynamicConfigEventReceiver) {
        let (sender, receiver) = mpsc::channel(1);
        let receiver = DynamicConfigEventReceiver(receiver);
        let data = Self {
            sender,
            remote_bot_login: AtomicBool::default(),
        };
        (data, receiver)
    }

    pub(crate) async fn reload(&self) {
        if self.sender.send(DynamicConfigEvent::Reload).await.is_err() {
            error!("Reload event sending failed");
        }
    }

    pub(crate) fn is_remote_bot_login_enabled(&self) -> bool {
        self.remote_bot_login.load(Ordering::Relaxed)
    }

    pub(crate) fn set_remote_bot_login_enabled(&self, value: bool) {
        self.remote_bot_login.store(value, Ordering::Relaxed)
    }
}
