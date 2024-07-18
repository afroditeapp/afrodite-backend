//! Write commands that can be run concurrently also with synchronous
//! write commands.

use std::{collections::HashMap, fmt, fmt::Debug, sync::Arc};

use axum::body::BodyDataStream;
use config::Config;
use futures::Future;
use model::{AccountId, AccountIdInternal, ContentProcessingId, ProfileLink};
use tokio::sync::{Mutex, OwnedMutexGuard, RwLock};

use super::{
    cache::{CacheError, DatabaseCache},
    file::utils::FileDir,
    index::LocationIndexIteratorHandle,
    IntoDataError,
};
use crate::{
    content_processing::NewContentInfo,
    db_manager::RouterDatabaseWriteHandle,
    result::{Result, WrappedContextExt},
    DataError,
};

pub type OutputFuture<R> = Box<dyn Future<Output = R> + Send + 'static>;

pub enum ConcurrentWriteAction<R> {
    Image {
        handle: ConcurrentWriteContentHandle,
        action: Box<dyn FnOnce(ConcurrentWriteContentHandle) -> OutputFuture<R> + Send + 'static>,
    },
    Profile {
        handle: ConcurrentWriteProfileHandle,
        action: Box<dyn FnOnce(ConcurrentWriteProfileHandle) -> OutputFuture<R> + Send + 'static>,
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
    /// Content upload queue
    content_upload_queue: Arc<tokio::sync::Semaphore>,
    /// Profile index write queue
    profile_index_queue: Arc<tokio::sync::Semaphore>,
    account_write_locks: AccountWriteLockManager,
}

impl ConcurrentWriteCommandHandle {
    pub fn new(write: RouterDatabaseWriteHandle, config: &Config) -> Self {
        Self {
            write: write.into(),
            content_upload_queue: tokio::sync::Semaphore::new(config.queue_limits().content_upload)
                .into(),
            profile_index_queue: tokio::sync::Semaphore::new(num_cpus::get()).into(),
            account_write_locks: AccountWriteLockManager::default(),
        }
    }

    pub async fn accquire(&self, account: AccountId) -> ConcurrentWriteSelectorHandle {
        let lock = self.account_write_locks.lock_account(account).await;

        ConcurrentWriteSelectorHandle {
            write: self.write.clone(),
            content_upload_queue: self.content_upload_queue.clone(),
            profile_index_queue: self.profile_index_queue.clone(),
            _account_write_lock: lock,
        }
    }
}

pub struct ConcurrentWriteSelectorHandle {
    write: Arc<RouterDatabaseWriteHandle>,
    content_upload_queue: Arc<tokio::sync::Semaphore>,
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
        A: FnOnce(ConcurrentWriteContentHandle) -> OutputFuture<R> + Send + 'static,
    >(
        self,
        action: A,
    ) -> ConcurrentWriteAction<R> {
        let permit = self
            .content_upload_queue
            .clone()
            .acquire_owned()
            .await
            // Code does not call close method of Semaphore, so this should not
            // panic.
            .expect("Semaphore was closed. This should not happen.");

        let handle = ConcurrentWriteContentHandle {
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

pub struct ConcurrentWriteContentHandle {
    write: Arc<RouterDatabaseWriteHandle>,
    _permit: tokio::sync::OwnedSemaphorePermit,
    _account_write_lock: OwnedMutexGuard<AccountHandle>,
}

impl fmt::Debug for ConcurrentWriteContentHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConcurrentWriteImageHandle").finish()
    }
}

impl ConcurrentWriteContentHandle {
    pub async fn save_to_tmp(
        &self,
        id: AccountIdInternal,
        stream: BodyDataStream,
    ) -> Result<NewContentInfo, DataError> {
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
    cache: &'a DatabaseCache,
    file_dir: &'a FileDir,
    location: LocationIndexIteratorHandle<'a>,
}

impl<'a> WriteCommandsConcurrent<'a> {
    pub fn new(
        cache: &'a DatabaseCache,
        file_dir: &'a FileDir,
        location: LocationIndexIteratorHandle<'a>,
    ) -> Self {
        Self {
            cache,
            file_dir,
            location,
        }
    }

    pub async fn save_to_tmp(
        &self,
        id: AccountIdInternal,
        stream: BodyDataStream,
    ) -> Result<NewContentInfo, DataError> {
        let content_id = ContentProcessingId::new_random_id();

        // Clear tmp dir in case previous content writing failed and there is no
        // content ID in the database about it.
        self.file_dir
            .tmp_dir(id.as_id())
            .remove_contents_if_exists()
            .await?;

        let tmp_raw_img = self
            .file_dir
            .raw_content_upload(id.as_id(), content_id.to_content_id());
        tmp_raw_img.save_stream(stream).await?;

        let tmp_img = self
            .file_dir
            .processed_content_upload(id.as_id(), content_id.to_content_id());

        Ok(NewContentInfo {
            processing_id: content_id,
            tmp_raw_img,
            tmp_img,
        })
    }

    pub async fn next_profiles(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<ProfileLink>, DataError> {
        let (location, query_maker_filters) = self
            .cache
            .read_cache(id.as_id(), |e| {
                let p = e.profile.as_ref().ok_or(CacheError::FeatureNotEnabled)?;
                error_stack::Result::<_, CacheError>::Ok((p.location.clone(), p.filters()))
            })
            .await
            .into_data_error(id)??;

        let (mut next_state, profiles) = self
            .location
            .next_profiles(location.current_iterator, &query_maker_filters)
            .await?;

        let (next_state, profiles) = if let Some(mut profiles) = profiles {
            loop {
                if profiles.len() >= 10 {
                    break (next_state, Some(profiles));
                } else {
                    let (new_next_state, new_profiles) = self
                        .location
                        .next_profiles(next_state, &query_maker_filters)
                        .await?;
                    next_state = new_next_state;

                    if let Some(new_profiles) = new_profiles {
                        profiles.extend(new_profiles);
                    } else {
                        break (next_state, Some(profiles));
                    }
                }
            }
        } else {
            (next_state, None)
        };

        self.cache
            .write_cache(id.as_id(), |e| {
                if let Some(p) = e.profile.as_mut() {
                    p.location.current_iterator = next_state;
                }
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
            .ok_or(DataError::FeatureDisabled.report())?;

        let next_state = self
            .location
            .reset_iterator(location.current_iterator, location.current_position);
        self.cache
            .write_cache(id.as_id(), |e| {
                if let Some(p) = e.profile.as_mut() {
                    p.location.current_iterator = next_state;
                }
                Ok(())
            })
            .await
            .into_data_error(id)?;
        Ok(())
    }
}
