//! Write commands that can be run concurrently also with synchronous
//! write commands.

use std::{
    collections::HashMap,
    fmt::{self, Debug},
    sync::{
        Arc,
        atomic::{AtomicI64, Ordering},
    },
};

use axum::body::BodyDataStream;
use futures::Future;
use model::{AccountId, AccountIdInternal, ContentProcessingId, ContentSlot};
use model_server_data::{
    AutomaticProfileSearchIteratorSessionId, AutomaticProfileSearchIteratorSessionIdInternal,
    ProfileIteratorSessionId, ProfileIteratorSessionIdInternal, ProfileLink,
};
use tokio::sync::{Mutex, OwnedMutexGuard, RwLock};

use super::{
    IntoDataError,
    cache::{CacheError, DatabaseCache},
    file::utils::FileDir,
};
use crate::{
    DataError, content_processing::NewContentInfo, db_manager::RouterDatabaseWriteHandle,
    index::LocationIndexIteratorHandle, result::Result,
};

const PROFILE_ITERATOR_PAGE_SIZE: usize = 25;

static NEXT_CONTENT_PROCESSING_ID: AtomicI64 = AtomicI64::new(0);

pub type OutputFuture<R> = Box<dyn Future<Output = R> + Send + 'static>;

pub enum ConcurrentWriteAction<R> {
    Image {
        handle: ConcurrentWriteContentHandle,
        action: Box<dyn FnOnce(ConcurrentWriteContentHandle) -> OutputFuture<R> + Send + 'static>,
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
    /// Profile index write queue
    profile_index_queue: Arc<tokio::sync::Semaphore>,
    account_write_locks: AccountWriteLockManager,
}

impl ConcurrentWriteCommandHandle {
    pub fn new(write: Arc<RouterDatabaseWriteHandle>) -> Self {
        Self {
            write,
            profile_index_queue: tokio::sync::Semaphore::new(num_cpus::get()).into(),
            account_write_locks: AccountWriteLockManager::default(),
        }
    }

    pub async fn accquire(&self, account: AccountId) -> ConcurrentWriteSelectorHandle {
        let lock = self.account_write_locks.lock_account(account).await;

        ConcurrentWriteSelectorHandle {
            write: self.write.clone(),
            profile_index_queue: self.profile_index_queue.clone(),
            _account_write_lock: lock,
        }
    }
}

pub struct ConcurrentWriteSelectorHandle {
    write: Arc<RouterDatabaseWriteHandle>,
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
        let handle = ConcurrentWriteContentHandle {
            write: self.write,
            _account_write_lock: self._account_write_lock,
        };

        ConcurrentWriteAction::Image {
            handle,
            action: Box::new(action),
        }
    }

    pub async fn profile_blocking(self) -> ConcurrentWriteProfileHandleBlocking {
        let permit = self
            .profile_index_queue
            .clone()
            .acquire_owned()
            .await
            // Code does not call close method of Semaphore, so this should not
            // panic.
            .expect("Semaphore was closed. This should not happen.");

        ConcurrentWriteProfileHandleBlocking {
            write: self.write,
            _permit: permit,
            account_write_lock: self._account_write_lock,
        }
    }

    pub fn into_lock(self) -> OwnedMutexGuard<AccountHandle> {
        self._account_write_lock
    }
}

pub struct ConcurrentWriteContentHandle {
    write: Arc<RouterDatabaseWriteHandle>,
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
        slot: ContentSlot,
        stream: BodyDataStream,
    ) -> Result<NewContentInfo, DataError> {
        self.write
            .user_write_commands_account()
            .save_to_tmp(id, slot, stream)
            .await
    }
}

pub struct ConcurrentWriteProfileHandleBlocking {
    write: Arc<RouterDatabaseWriteHandle>,
    _permit: tokio::sync::OwnedSemaphorePermit,
    account_write_lock: OwnedMutexGuard<AccountHandle>,
}

impl fmt::Debug for ConcurrentWriteProfileHandleBlocking {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConcurrentWriteProfileHandleBlocking")
            .finish()
    }
}

impl ConcurrentWriteProfileHandleBlocking {
    pub fn next_profiles(
        &self,
        id: AccountIdInternal,
        iterator_id: ProfileIteratorSessionId,
    ) -> Result<Option<Vec<ProfileLink>>, DataError> {
        self.write
            .user_write_commands_account()
            .next_profiles(id, iterator_id)
    }

    pub fn reset_profile_iterator(
        &self,
        id: AccountIdInternal,
    ) -> Result<ProfileIteratorSessionIdInternal, DataError> {
        self.write
            .user_write_commands_account()
            .reset_profile_iterator(id)
    }

    pub fn automatic_profile_search_next_profiles(
        &self,
        id: AccountIdInternal,
        iterator_id: AutomaticProfileSearchIteratorSessionId,
    ) -> Result<Option<Vec<ProfileLink>>, DataError> {
        self.write
            .user_write_commands_account()
            .automatic_profile_search_next_profiles(id, iterator_id)
    }

    pub fn automatic_profile_search_reset_profile_iterator(
        &self,
        id: AccountIdInternal,
    ) -> Result<AutomaticProfileSearchIteratorSessionIdInternal, DataError> {
        self.write
            .user_write_commands_account()
            .automatic_profile_search_reset_profile_iterator(id)
    }

    pub fn into_lock(self) -> OwnedMutexGuard<AccountHandle> {
        self.account_write_lock
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
        slot: ContentSlot,
        stream: BodyDataStream,
    ) -> Result<NewContentInfo, DataError> {
        let processing_id_i64 = NEXT_CONTENT_PROCESSING_ID.fetch_add(1, Ordering::Relaxed);
        let processing_id = ContentProcessingId::new(id.as_id(), slot, processing_id_i64);

        // There might be some content in the tmp dir which does not have
        // content ID in the database if previous content writing failed.
        // Because tmp dir is also used for data export saving it is not
        // possible to clear tmp dir completely at this point.
        // The dir is cleared completely when server starts.

        let tmp_raw_img = self.file_dir.raw_content_upload(id.as_id(), processing_id);
        tmp_raw_img.save_stream(stream).await?;

        let tmp_img = self
            .file_dir
            .processed_content_upload(id.as_id(), processing_id);

        Ok(NewContentInfo {
            processing_id,
            tmp_raw_img,
            tmp_img,
        })
    }

    /// Returns None if profile iterator session ID is
    /// invalid.
    pub fn next_profiles(
        &self,
        id: AccountIdInternal,
        iterator_id_from_client: ProfileIteratorSessionId,
    ) -> Result<Option<Vec<ProfileLink>>, DataError> {
        let (location, query_maker_filters, iterator_id_current) = self
            .cache
            .read_cache_blocking(id.as_id(), |e| {
                let p = &e.profile;
                error_stack::Result::<_, CacheError>::Ok((
                    p.location.clone(),
                    p.filters(),
                    p.profile_iterator_session_id,
                ))
            })
            .into_data_error(id)??;

        let iterator_id_current: Option<ProfileIteratorSessionId> =
            iterator_id_current.map(|v| v.into());
        if iterator_id_current != Some(iterator_id_from_client) {
            return Ok(None);
        }

        let (mut next_state, profiles) = self
            .location
            .next_profiles(location.current_iterator, &query_maker_filters);

        let (next_state, profiles) = if let Some(mut profiles) = profiles {
            loop {
                if profiles.len() >= PROFILE_ITERATOR_PAGE_SIZE {
                    break (next_state, profiles);
                } else {
                    let (new_next_state, new_profiles) = self
                        .location
                        .next_profiles(next_state, &query_maker_filters);
                    next_state = new_next_state;

                    if let Some(new_profiles) = new_profiles {
                        profiles.extend(new_profiles);
                    } else {
                        break (next_state, profiles);
                    }
                }
            }
        } else {
            (next_state, vec![])
        };

        self.cache
            .write_cache_blocking(id.as_id(), |e| {
                e.profile.location.current_iterator = next_state;
                Ok(())
            })
            .into_data_error(id)?;

        Ok(Some(profiles))
    }

    pub fn reset_profile_iterator(
        &self,
        id: AccountIdInternal,
    ) -> Result<ProfileIteratorSessionIdInternal, DataError> {
        self.cache
            .write_cache_blocking(id.as_id(), |e| {
                let p = &mut e.profile;
                let new_id = ProfileIteratorSessionIdInternal::create(
                    &mut p.profile_iterator_session_id_storage,
                );
                let next_state = self
                    .location
                    .new_iterator_state(&p.location.current_position, p.state.random_profile_order);
                p.location.current_iterator = next_state;
                p.profile_iterator_session_id = Some(new_id);
                Ok(new_id)
            })
            .into_data_error(id)
    }

    /// Returns None if profile iterator session ID is
    /// invalid.
    pub fn automatic_profile_search_next_profiles(
        &self,
        id: AccountIdInternal,
        iterator_id_from_client: AutomaticProfileSearchIteratorSessionId,
    ) -> Result<Option<Vec<ProfileLink>>, DataError> {
        let (iterator_state, query_maker_filters, iterator_id_current) = self
            .cache
            .read_cache_blocking(id.as_id(), |e| {
                let p = &e.profile;
                error_stack::Result::<_, CacheError>::Ok((
                    p.automatic_profile_search.current_iterator.clone(),
                    p.automatic_profile_search_filters(),
                    p.automatic_profile_search.iterator_session_id,
                ))
            })
            .into_data_error(id)??;

        let iterator_id_current: Option<AutomaticProfileSearchIteratorSessionId> =
            iterator_id_current.map(|v| v.into());
        if iterator_id_current != Some(iterator_id_from_client) {
            return Ok(None);
        }

        let (mut next_state, profiles) = self
            .location
            .next_profiles(iterator_state, &query_maker_filters);

        let (next_state, profiles) = if let Some(mut profiles) = profiles {
            loop {
                if profiles.len() >= PROFILE_ITERATOR_PAGE_SIZE {
                    break (next_state, profiles);
                } else {
                    let (new_next_state, new_profiles) = self
                        .location
                        .next_profiles(next_state, &query_maker_filters);
                    next_state = new_next_state;

                    if let Some(new_profiles) = new_profiles {
                        profiles.extend(new_profiles);
                    } else {
                        break (next_state, profiles);
                    }
                }
            }
        } else {
            (next_state, vec![])
        };

        self.cache
            .write_cache_blocking(id.as_id(), |e| {
                e.profile.automatic_profile_search.current_iterator = next_state;
                Ok(())
            })
            .into_data_error(id)?;

        Ok(Some(profiles))
    }

    pub fn automatic_profile_search_reset_profile_iterator(
        &self,
        id: AccountIdInternal,
    ) -> Result<AutomaticProfileSearchIteratorSessionIdInternal, DataError> {
        self.cache
            .write_cache_blocking(id.as_id(), |e| {
                let p = &mut e.profile;
                let distance_filters_enabled =
                    p.automatic_profile_search.settings().distance_filters;
                let new_id = AutomaticProfileSearchIteratorSessionIdInternal::create(
                    &mut p.automatic_profile_search.iterator_session_id_storage,
                );
                let area = if distance_filters_enabled {
                    &p.location.current_position
                } else {
                    &p.location
                        .current_position
                        .with_max_area(self.location.index())
                };
                let next_state = self.location.new_iterator_state(area, false);
                p.automatic_profile_search.current_iterator = next_state;
                p.automatic_profile_search.iterator_session_id = Some(new_id);
                Ok(new_id)
            })
            .into_data_error(id)
    }
}
