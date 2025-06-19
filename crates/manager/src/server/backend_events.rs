use manager_model::ServerEvent;
use tokio::sync::watch;

#[derive(Debug)]
pub struct BackendEventsHandle {
    sender: watch::Sender<Vec<ServerEvent>>,
}

impl BackendEventsHandle {
    pub fn new(initial: Vec<ServerEvent>) -> Self {
        let (sender, _) = watch::channel(initial);
        Self { sender }
    }

    pub fn send(&self, events: Vec<ServerEvent>) {
        self.sender.send_replace(events);
    }

    pub fn receiver(&self) -> watch::Receiver<Vec<ServerEvent>> {
        self.sender.subscribe()
    }
}
