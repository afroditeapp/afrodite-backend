//! Write commands that can be run concurrently also with synchronous
//! write commands.

use std::{collections::HashMap, fmt, fmt::Debug, sync::Arc};

use axum::extract::BodyStream;
use config::Config;
use database::{history::write::HistoryWriteCommands, CurrentWriteHandle, HistoryWriteHandle};
use error_stack::{Result, ResultExt};
use futures::Future;
use model::{AccountId, AccountIdInternal, ContentId, ProfileLink};
use simple_backend::image::ImageProcess;
use tokio::sync::{Mutex, OwnedMutexGuard, RwLock};

use super::{
    cache::DatabaseCache, file::utils::FileDir, index::LocationIndexIteratorHandle, IntoDataError,
    RouterDatabaseWriteHandle,
};
use crate::data::DataError;

pub type OutputFuture<R> = Box<dyn Future<Output = R> + Send + Sync + 'static>;

pub enum ConcurrentWriteAction<R> {
    Image {
        handle: ConcurrentWriteImageHandle,
        action:
            Box<dyn FnOnce(ConcurrentWriteImageHandle) -> OutputFuture<R> + Send + Sync + 'static>,
    },
    Profile {
        handle: ConcurrentWriteProfileHandle,
        action: Box<
            dyn FnOnce(ConcurrentWriteProfileHandle) -> OutputFuture<R> + Send + Sync + 'static,
        >,
    },
}

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
    /// Image upload queue
    image_upload_queue: Arc<tokio::sync::Semaphore>,
    /// Profile index write queue
    profile_index_queue: Arc<tokio::sync::Semaphore>,
    account_write_locks: AccountWriteLockManager,
}

impl ConcurrentWriteCommandHandle {
    pub fn new(write: RouterDatabaseWriteHandle, config: &Config) -> Self {
        Self {
            write: write.into(),
            image_upload_queue: tokio::sync::Semaphore::new(config.queue_limits().image_upload)
                .into(),
            profile_index_queue: tokio::sync::Semaphore::new(num_cpus::get()).into(),
            account_write_locks: AccountWriteLockManager::default(),
        }
    }

    pub async fn accquire(&self, account: AccountId) -> ConcurrentWriteSelectorHandle {
        let lock = self.account_write_locks.lock_account(account).await;

        ConcurrentWriteSelectorHandle {
            write: self.write.clone(),
            image_upload_queue: self.image_upload_queue.clone(),
            profile_index_queue: self.profile_index_queue.clone(),
            _account_write_lock: lock,
        }
    }
}

pub struct ConcurrentWriteSelectorHandle {
    write: Arc<RouterDatabaseWriteHandle>,
    image_upload_queue: Arc<tokio::sync::Semaphore>,
    profile_index_queue: Arc<tokio::sync::Semaphore>,
    _account_write_lock: OwnedMutexGuard<AccountHandle>,
}

impl fmt::Debug for ConcurrentWriteSelectorHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConcurrentWriteSelectorHandle").finish()
    }
}

impl ConcurrentWriteSelectorHandle {
    pub async fn accquire_image<
        R,
        A: FnOnce(ConcurrentWriteImageHandle) -> OutputFuture<R> + Send + Sync + 'static,
    >(
        self,
        action: A,
    ) -> ConcurrentWriteAction<R> {
        let permit = self
            .image_upload_queue
            .clone()
            .acquire_owned()
            .await
            // Code does not call close method of Semaphore, so this should not
            // panic.
            .expect("Semaphore was closed. This should not happen.");

        let handle = ConcurrentWriteImageHandle {
            write: self.write,
            _permit: permit,
            _account_write_lock: self._account_write_lock,
        };

        ConcurrentWriteAction::Image {
            handle,
            action: Box::new(action),
        }
    }

    pub async fn accquire_profile<
        R,
        A: FnOnce(ConcurrentWriteProfileHandle) -> OutputFuture<R> + Send + Sync + 'static,
    >(
        self,
        action: A,
    ) -> ConcurrentWriteAction<R> {
        let permit = self
            .profile_index_queue
            .clone()
            .acquire_owned()
            .await
            // Code does not call close method of Semaphore, so this should not
            // panic.
            .expect("Semaphore was closed. This should not happen.");

        let handle = ConcurrentWriteProfileHandle {
            write: self.write,
            _permit: permit,
            _account_write_lock: self._account_write_lock,
        };

        ConcurrentWriteAction::Profile {
            handle,
            action: Box::new(action),
        }
    }
}

pub struct ConcurrentWriteImageHandle {
    write: Arc<RouterDatabaseWriteHandle>,
    _permit: tokio::sync::OwnedSemaphorePermit,
    _account_write_lock: OwnedMutexGuard<AccountHandle>,
}

impl fmt::Debug for ConcurrentWriteImageHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConcurrentWriteImageHandle").finish()
    }
}

impl ConcurrentWriteImageHandle {
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
}

pub struct ConcurrentWriteProfileHandle {
    write: Arc<RouterDatabaseWriteHandle>,
    _permit: tokio::sync::OwnedSemaphorePermit,
    _account_write_lock: OwnedMutexGuard<AccountHandle>,
}

impl fmt::Debug for ConcurrentWriteProfileHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConcurrentWriteProfileHandle").finish()
    }
}

impl ConcurrentWriteProfileHandle {
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
    current_write_handle: &'a CurrentWriteHandle,
    history_write_handle: &'a HistoryWriteHandle,
    cache: &'a DatabaseCache,
    file_dir: &'a FileDir,
    location: LocationIndexIteratorHandle<'a>,
    image_processing_queue: &'a Arc<tokio::sync::Semaphore>,
}

impl<'a> WriteCommandsConcurrent<'a> {
    pub fn new(
        current_write_handle: &'a CurrentWriteHandle,
        history_write_handle: &'a HistoryWriteHandle,
        cache: &'a DatabaseCache,
        file_dir: &'a FileDir,
        location: LocationIndexIteratorHandle<'a>,
        image_processing_queue: &'a Arc<tokio::sync::Semaphore>,
    ) -> Self {
        Self {
            current_write_handle,
            history_write_handle,
            cache,
            file_dir,
            location,
            image_processing_queue,
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

        let tmp_raw_img = self
            .file_dir
            .unprocessed_image_upload(id.as_id(), content_id);
        tmp_raw_img
            .save_stream(stream)
            .await
            .change_context(DataError::File)?;

        // Limit image processing because of memory usage
        let permit = self
            .image_processing_queue
            .acquire()
            .await
            // Code does not call close method of Semaphore, so this should not
            // panic.
            .expect("Semaphore was closed. This should not happen.");

        let tmp_img = self.file_dir.processed_image_upload(id.as_id(), content_id);
        ImageProcess::start_image_process(tmp_raw_img.as_path(), tmp_img.as_path())
            .await
            .change_context(DataError::ImageProcess)?;

        drop(permit);

        tmp_raw_img
            .remove_if_exists()
            .await
            .change_context(DataError::File)?;

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

        let (next_state, profiles) = self
            .location
            .next_profiles(location.current_iterator)
            .await?;
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

        let next_state = self
            .location
            .reset_iterator(location.current_iterator, location.current_position);
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
        self.history_write_handle.sqlx_cmds()
    }
}
