use std::{collections::HashMap, sync::Arc};

use model::{AccountIdDb, AccountIdInternal, ContentProcessingState, ContentSlot};
use tokio::sync::{Mutex, OwnedMutexGuard, RwLock, oneshot};

use crate::event::EventManagerWithCacheReference;

mod process;
mod upload;

use process::ProcessManagerData;
pub use process::{ContentProcessingReceiver, Data, ProcessingState, UploadInfo};
use upload::UploadManagerData;
pub use upload::UploadPermit;

pub struct ContentProcessingOngoing;

#[derive(Debug)]
pub enum ProcessingPhase {
    Idle,
    Uploading {
        /// Drop to cancel upload
        cancel_sender: Option<oneshot::Sender<()>>,
        completed_receiver: oneshot::Receiver<()>,
    },
    Processing,
}

pub struct ContentProcessingManagerData {
    upload: UploadManagerData,
    process: ProcessManagerData,
    processing_locks: RwLock<HashMap<AccountIdDb, Arc<Mutex<ProcessingPhase>>>>,
}

impl ContentProcessingManagerData {
    pub fn new() -> (Self, ContentProcessingReceiver) {
        let (process, receiver) = ProcessManagerData::new();
        (
            Self {
                upload: UploadManagerData::new(),
                process,
                processing_locks: RwLock::new(HashMap::new()),
            },
            receiver,
        )
    }

    async fn processing_phase_lock(
        &self,
        account_id: AccountIdInternal,
    ) -> OwnedMutexGuard<ProcessingPhase> {
        let mut write = self.processing_locks.write().await;
        if let Some(v) = write.get(account_id.as_db_id()) {
            v.clone().lock_owned().await
        } else {
            let value = Arc::new(Mutex::new(ProcessingPhase::Idle));
            write.insert(*account_id.as_db_id(), value.clone());
            value.lock_owned().await
        }
    }

    pub fn data(&self) -> &RwLock<Data> {
        self.process.data()
    }

    pub async fn begin_upload(
        &self,
        content_owner: AccountIdInternal,
    ) -> std::result::Result<UploadPermit, ContentProcessingOngoing> {
        self.upload
            .begin_upload(self.processing_phase_lock(content_owner).await)
            .await
    }

    pub async fn queue_new_content(
        &self,
        content_owner: AccountIdInternal,
        slot: ContentSlot,
        upload_info: UploadInfo,
        new_content_params: model_server_data::NewContentParams,
    ) {
        let processing_phase_lock = self.processing_phase_lock(content_owner).await;
        self.process
            .queue_new_content(
                content_owner,
                slot,
                upload_info,
                new_content_params,
                processing_phase_lock,
            )
            .await;
    }

    pub async fn pop_from_queue(
        &self,
        events: EventManagerWithCacheReference<'_>,
    ) -> Option<ProcessingState> {
        self.process.pop_from_queue(events).await
    }

    pub async fn set_processing_phase_idle(&self, account_id: AccountIdInternal) {
        let mut lock = self.processing_phase_lock(account_id).await;
        *lock = ProcessingPhase::Idle;
    }

    pub async fn get_current_state(
        &self,
        account_id: AccountIdInternal,
    ) -> Option<ContentProcessingState> {
        self.process.get_current_state(account_id).await
    }
}
