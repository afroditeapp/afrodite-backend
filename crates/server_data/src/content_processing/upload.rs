use tokio::sync::{OwnedMutexGuard, oneshot};

use crate::content_processing::{ContentProcessingOngoing, ProcessingPhase};

pub async fn begin_upload(
    mut lock: OwnedMutexGuard<ProcessingPhase>,
) -> std::result::Result<UploadPermit, ContentProcessingOngoing> {
    match &mut *lock {
        ProcessingPhase::Processing => {
            return Err(ContentProcessingOngoing);
        }
        ProcessingPhase::Uploading {
            cancel_sender,
            completed_receiver,
        } => {
            drop(cancel_sender.take());
            let _ = completed_receiver.await;
        }
        ProcessingPhase::Idle => (),
    };

    let (cancel_sender, cancel_receiver) = oneshot::channel();
    let (completed_sender, completed_receiver) = oneshot::channel();

    *lock = ProcessingPhase::Uploading {
        cancel_sender: Some(cancel_sender),
        completed_receiver,
    };

    Ok(UploadPermit {
        cancel_receiver,
        _completed_sender: completed_sender,
    })
}

#[derive(Debug)]
pub struct UploadPermit {
    cancel_receiver: oneshot::Receiver<()>,
    /// Dropping this will signal completed/cancelled uploading
    _completed_sender: oneshot::Sender<()>,
}

impl UploadPermit {
    pub fn cancel_receiver_mut(&mut self) -> &mut oneshot::Receiver<()> {
        &mut self.cancel_receiver
    }
}
