//! Write commands that can be run concurrently also with synchronous
//! write commands.

use std::{collections::HashMap, fmt, fmt::Debug, sync::Arc};

use axum::extract::BodyStream;
use database::{
    history::write::HistoryWriteCommands,
    sqlite::{CurrentDataWriteHandle, HistoryWriteHandle},
};
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, AccountId, ContentId, ProfileLink};
use tokio::sync::{Mutex, OwnedMutexGuard, RwLock};

use super::{
    cache::DatabaseCache, file::utils::FileDir, index::LocationIndexIteratorGetter,
    RouterDatabaseWriteHandle, IntoDataError,
};
use crate::{data::DataError};

const CONCURRENT_WRITE_COMMAND_LIMIT: usize = 10;

pub struct AccountHandle;

#[derive(Default, Clone)]
pub struct AccountWriteLockManager {
    locks: Arc<RwLock<HashMap<AccountId, Arc<Mutex<AccountHandle>>>>>,
}

impl fmt::Debug for AccountWriteLockManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountWriteLockManager").finish()
    }
}

impl AccountWriteLockManager {
    pub async fn lock_account(&self, a: AccountId) -> OwnedMutexGuard<AccountHandle> {
        let mutex = {
            let mut write_lock = self.locks.write().await;
            if let Some(mutex) = write_lock.get(&a) {
                mutex.clone()
            } else {
                let mutex = Arc::new(Mutex::new(AccountHandle));
                write_lock.insert(a, mutex.clone());
                mutex
            }
        };
        mutex.lock_owned().await
    }
}

#[derive(Debug)]
pub struct ConcurrentWriteCommandHandle {
    write: Arc<RouterDatabaseWriteHandle>,
    semaphore: Arc<tokio::sync::Semaphore>,
    account_write_locks: AccountWriteLockManager,
}

impl ConcurrentWriteCommandHandle {
    pub fn new(write: RouterDatabaseWriteHandle) -> Self {
        Self {
            write: write.into(),
            semaphore: tokio::sync::Semaphore::new(CONCURRENT_WRITE_COMMAND_LIMIT).into(),
            account_write_locks: AccountWriteLockManager::default(),
        }
    }

    pub async fn accquire(&self, account: AccountId) -> ConcurrentWriteHandle {
        let lock = self.account_write_locks.lock_account(account).await;

        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            // Code does not call close method of Semaphore, so this should not
            // panic.
            .expect("Semaphore was closed. This should not happen.");

        ConcurrentWriteHandle {
            write: self.write.clone(),
            _permit: permit,
            _account_write_lock: lock,
        }
    }
}

pub struct ConcurrentWriteHandle {
    write: Arc<RouterDatabaseWriteHandle>,
    _permit: tokio::sync::OwnedSemaphorePermit,
    _account_write_lock: OwnedMutexGuard<AccountHandle>,
}

impl fmt::Debug for ConcurrentWriteHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConcurrentWriteHandle").finish()
    }
}

impl ConcurrentWriteHandle {
    pub async fn save_to_tmp(
        &self,
        id: AccountIdInternal,
        stream: BodyStream,
    ) -> Result<ContentId, DataError> {
        self.write
            .user_write_commands_account()
            .save_to_tmp(id, stream)
            .await
    }

    pub async fn next_profiles(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<ProfileLink>, DataError> {
        self.write
            .user_write_commands_account()
            .next_profiles(id)
            .await
    }

    pub async fn reset_profile_iterator(&self, id: AccountIdInternal) -> Result<(), DataError> {
        self.write
            .user_write_commands_account()
            .reset_profile_iterator(id)
            .await
    }
}

/// Commands that can run concurrently with other write commands, but which have
/// limitation that one account can execute only one command at a time.
/// It possible to run this and normal write command concurrently for
/// one account.
pub struct WriteCommandsConcurrent<'a> {
    current_write: &'a CurrentDataWriteHandle,
    history_write: &'a HistoryWriteHandle,
    cache: &'a DatabaseCache,
    file_dir: &'a FileDir,
    location: LocationIndexIteratorGetter<'a>,
}

impl<'a> WriteCommandsConcurrent<'a> {
    pub fn new(
        current_write: &'a CurrentDataWriteHandle,
        history_write: &'a HistoryWriteHandle,
        cache: &'a DatabaseCache,
        file_dir: &'a FileDir,
        location: LocationIndexIteratorGetter<'a>,
    ) -> Self {
        Self {
            current_write,
            history_write,
            cache,
            file_dir,
            location,
        }
    }

    pub async fn save_to_tmp(
        &self,
        id: AccountIdInternal,
        stream: BodyStream,
    ) -> Result<ContentId, DataError> {
        let content_id = ContentId::new_random_id();

        // Clear tmp dir if previous image writing failed and there is no
        // content ID in the database about it.
        self.file_dir
            .tmp_dir(id.as_id())
            .remove_contents_if_exists()
            .await
            .change_context(DataError::File)?;

        let raw_img = self
            .file_dir
            .unprocessed_image_upload(id.as_id(), content_id);
        raw_img
            .save_stream(stream)
            .await
            .change_context(DataError::File)?;

        // TODO: image safety checks and processing

        Ok(content_id)
    }

    pub async fn next_profiles(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<ProfileLink>, DataError> {
        let location = self
            .cache
            .read_cache(id.as_id(), |e| {
                e.profile.as_ref().map(|p| p.location.clone())
            })
            .await
            .into_data_error(id)?
            .ok_or(DataError::FeatureDisabled)?;

        let iterator = self.location.get().ok_or(DataError::FeatureDisabled)?;
        let (next_state, profiles) = iterator.next_profiles(location.current_iterator).await;
        self.cache
            .write_cache(id.as_id(), |e| {
                e.profile
                    .as_mut()
                    .map(move |p| p.location.current_iterator = next_state);
                Ok(())
            })
            .await
            .into_data_error(id)?;

        Ok(profiles.unwrap_or(Vec::new()))
    }

    pub async fn reset_profile_iterator(&self, id: AccountIdInternal) -> Result<(), DataError> {
        let location = self
            .cache
            .read_cache(id.as_id(), |e| {
                e.profile.as_ref().map(|p| p.location.clone())
            })
            .await
            .into_data_error(id)?
            .ok_or(DataError::FeatureDisabled)?;

        let iterator = self.location.get().ok_or(DataError::FeatureDisabled)?;
        let next_state =
            iterator.reset_iterator(location.current_iterator, location.current_position);
        self.cache
            .write_cache(id.as_id(), |e| {
                e.profile
                    .as_mut()
                    .map(move |p| p.location.current_iterator = next_state);
                Ok(())
            })
            .await
            .into_data_error(id)?;
        Ok(())
    }

    fn history(&self) -> HistoryWriteCommands {
        HistoryWriteCommands::new(&self.history_write)
    }
}
