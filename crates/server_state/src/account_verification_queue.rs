use std::collections::{HashMap, VecDeque};

use model::{AccountId, AccountIdInternal, AccountVerificationQueueItem};
use server_data::event::EventManagerWithCacheReference;
use tokio::sync::RwLock;
use tracing::warn;

#[derive(Debug)]
pub enum AccountVerificationQueueAddError {
    AlreadyQueued,
    QueueFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountVerificationQueueRemoveNextError {
    QueueEmpty,
    AccountIdMismatch,
}

#[derive(Default)]
struct QueueData {
    queue: VecDeque<AccountId>,
    items: HashMap<AccountId, AccountVerificationQueueItem>,
}

#[derive(Default)]
pub struct AccountVerificationQueueData {
    data: RwLock<QueueData>,
}

impl AccountVerificationQueueData {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn add(
        &self,
        account_id: AccountIdInternal,
        item: AccountVerificationQueueItem,
        max_queue_length: u16,
    ) -> Result<(), AccountVerificationQueueAddError> {
        let mut write = self.data.write().await;
        let account_id = account_id.as_id();

        if write.items.contains_key(&account_id) {
            return Err(AccountVerificationQueueAddError::AlreadyQueued);
        }

        if write.queue.len() >= usize::from(max_queue_length) {
            return Err(AccountVerificationQueueAddError::QueueFull);
        }

        write.queue.push_back(account_id);
        write.items.insert(account_id, item);

        Ok(())
    }

    pub async fn queue_position(&self, account_id: AccountIdInternal) -> Option<u32> {
        let read = self.data.read().await;
        let account_id = account_id.as_id();

        // Check that account has item in queue
        read.items.get(&account_id)?;

        read.queue
            .iter()
            .position(|v| *v == account_id)
            .and_then(|i| u32::try_from(i + 1).ok())
    }

    pub async fn next_item(&self) -> Option<(AccountId, AccountVerificationQueueItem)> {
        let read = self.data.read().await;

        let account_id = *read.queue.front()?;
        let item = read.items.get(&account_id)?;

        Some((account_id, item.clone()))
    }

    pub async fn remove_next_item(
        &self,
        expected_account_id: AccountIdInternal,
        event_manager: &EventManagerWithCacheReference<'_>,
    ) -> Result<(), AccountVerificationQueueRemoveNextError> {
        let mut write = self.data.write().await;
        let expected_account_id = expected_account_id.as_id();

        let Some(next_account_id) = write.queue.front().copied() else {
            return Err(AccountVerificationQueueRemoveNextError::QueueEmpty);
        };

        if next_account_id != expected_account_id {
            return Err(AccountVerificationQueueRemoveNextError::AccountIdMismatch);
        }

        write.queue.pop_front();
        write.items.remove(&next_account_id);

        let queue_position_change_for_expected_account = [(next_account_id, Option::<u8>::None)];
        let queue_position_changes_for_other_accounts = write
            .queue
            .iter()
            .take(10)
            .enumerate()
            .filter_map(|(index, account_id)| {
                u8::try_from(index + 1)
                    .ok()
                    .map(|queue_position| (*account_id, Some(queue_position)))
            });
        let all_queue_position_changes = queue_position_change_for_expected_account
            .into_iter()
            .chain(queue_position_changes_for_other_accounts);

        for (account_id, queue_position) in all_queue_position_changes {
            let result = event_manager
                .send_connected_event(
                    account_id,
                    model::EventToClientInternal::AccountVerificationQueuePositionChanged {
                        queue_position,
                    },
                )
                .await;
            if result.is_err() {
                warn!("Sending AccountVerificationQueuePositionChanged event failed");
            }
        }

        // Make sure that update events are sequential
        drop(write);

        Ok(())
    }
}
