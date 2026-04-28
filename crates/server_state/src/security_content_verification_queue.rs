use std::collections::{HashMap, VecDeque};

use model::{AccountIdDb, AccountIdInternal, ContentId};
use tokio::sync::RwLock;

#[derive(Debug)]
pub enum SecurityContentVerificationQueueAddError {
    AlreadyQueued,
    QueueFull,
}

#[derive(Debug, Clone)]
struct SecurityContentVerificationQueueItem {
    _security_content: ContentId,
    _verification_method: String,
    _verification_data: String,
}

#[derive(Default)]
struct QueueData {
    queue: VecDeque<AccountIdDb>,
    items: HashMap<AccountIdDb, SecurityContentVerificationQueueItem>,
}

#[derive(Default)]
pub struct SecurityContentVerificationQueueData {
    data: RwLock<QueueData>,
}

impl SecurityContentVerificationQueueData {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add(
        &self,
        account_id: AccountIdInternal,
        security_content: ContentId,
        verification_method: String,
        verification_data: String,
        max_queue_length: u16,
    ) -> Result<(), SecurityContentVerificationQueueAddError> {
        let mut write = self.data.write().await;
        let account_id = account_id.into_db_id();

        if write.items.contains_key(&account_id) {
            return Err(SecurityContentVerificationQueueAddError::AlreadyQueued);
        }

        if write.queue.len() >= usize::from(max_queue_length) {
            return Err(SecurityContentVerificationQueueAddError::QueueFull);
        }

        let item = SecurityContentVerificationQueueItem {
            _security_content: security_content,
            _verification_method: verification_method,
            _verification_data: verification_data,
        };

        write.queue.push_back(account_id);
        write.items.insert(account_id, item);

        Ok(())
    }

    pub async fn queue_position(&self, account_id: AccountIdInternal) -> Option<u32> {
        let read = self.data.read().await;
        let account_id_db = account_id.into_db_id();

        // Check that account has item in queue
        read.items.get(&account_id_db)?;

        read.queue
            .iter()
            .position(|v| *v == account_id_db)
            .and_then(|i| u32::try_from(i + 1).ok())
    }
}
