use std::{sync::Arc, future::Future};

use api_client::models::AccountId;
use axum::extract::BodyStream;
use error_stack::{Result, ResultExt, Report};
use serde::Serialize;
use tokio::sync::{MutexGuard, Mutex};
use tokio_stream::StreamExt;

use crate::{
    api::{model::{Account, AccountIdInternal, AccountSetup, ApiKey, Profile, AccountIdLight, ContentId, NewModerationRequest}, media::data::Moderation},
    config::Config,
    server::database::{sqlite::SqliteWriteHandle, DatabaseError},
    utils::{ErrorConversion, IntoReportExt, AppendErrorTo},
};

use super::{
    current::write::CurrentDataWriteCommands,
    history::write::HistoryWriteCommands,
    sqlite::{HistoryUpdateJson, SqliteUpdateJson, CurrentDataWriteHandle, HistoryWriteHandle},
    utils::GetReadWriteCmd, cache::{DatabaseCache, WriteCacheJson}, file::{utils::{ FileDir}, file::ImageSlot},
};

#[derive(Debug, Clone)]
pub enum WriteCmd {
    AccountId(AccountIdLight),
    Profile(AccountIdInternal),
    ApiKey(AccountIdInternal),
    AccountState(AccountIdInternal),
    AccountSetup(AccountIdInternal),
    MediaModerationRequest(AccountIdInternal),
    MediaModeration(AccountIdInternal),
}

impl std::fmt::Display for WriteCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Write command: {:?}", self))
    }
}

#[derive(Debug, Clone)]
pub struct HistoryWrite(pub WriteCmd);


impl std::fmt::Display for HistoryWrite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("History write command: {:?}", self))
    }
}

#[derive(Debug, Clone)]
pub struct CacheWrite(pub WriteCmd);


impl std::fmt::Display for CacheWrite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Cache write command: {:?}", self))
    }
}

// TODO: remove
// macro_rules! lock {
//     ($test:expr) => {
//         let s = $test;
//         let mutex = s.cache.get_write_lock_simple(s.locking_id).await.change_context(DatabaseError::Cache)?;
//         let lock = mutex.lock().await;
//         s.to_internal(&lock)
//     };
// }

/// One Account can do only one write command at a time.
pub struct AccountWriteLock;

pub struct WriteCommands<'a> {
    current_write: &'a CurrentDataWriteHandle,
    history_write: &'a HistoryWriteHandle,
    cache: &'a DatabaseCache,
    file_dir: &'a FileDir,
    locking_id: AccountIdLight,
}

impl <'a> WriteCommands<'a> {
    pub fn new(
        current_write: &'a CurrentDataWriteHandle,
        history_write: &'a HistoryWriteHandle,
        cache: &'a DatabaseCache,
        file_dir: &'a FileDir,
        locking_id: AccountIdLight,
    ) -> Self {
        Self {
            current_write,
            history_write,
            cache,
            file_dir,
            locking_id,
        }
    }

    pub async fn register(
        id_light: AccountIdLight,
        config: &Config,
        current_data_write: CurrentDataWriteHandle,
        history_wirte: HistoryWriteHandle,
        cache: &DatabaseCache,
    ) -> Result<AccountIdInternal, DatabaseError> {
        let current = CurrentDataWriteCommands::new(&current_data_write);
        let history = HistoryWriteCommands::new(&history_wirte);

        let account = Account::default();
        let account_setup = AccountSetup::default();
        let profile = Profile::default();

        let id = current
            .store_account_id(id_light)
            .await
            .with_info_lazy(|| WriteCmd::AccountId(id_light))?;

        history
            .store_account_id(id)
            .await
            .with_info_lazy(|| HistoryWrite(WriteCmd::AccountId(id_light)))?;

        cache
            .insert_account_if_not_exists(id)
            .await
            .with_info_lazy(|| CacheWrite(WriteCmd::AccountId(id_light)))?;

        current
            .store_api_key(id, None)
            .await
            .with_info_lazy(|| WriteCmd::ApiKey(id))?;

        if config.components().account {
            current
                .store_account(id, &account)
                .await
                .with_write_cmd_info::<Account>(id)?;

            history
                .store_account(id, &account)
                .await
                .with_history_write_cmd_info::<Account>(id)?;

            cache
                .write_cache(id.as_light(), |cache| cache.account = Some(account.clone().into()))
                .await
                .with_history_write_cmd_info::<Account>(id)?;

            current
                .store_account_setup(id, &account_setup)
                .await
                .with_write_cmd_info::<AccountSetup>(id)?;

            history
                .store_account_setup(id, &account_setup)
                .await
                .with_history_write_cmd_info::<AccountSetup>(id)?;

        }

        if config.components().profile {
            current
                .store_profile(id, &profile)
                .await
                .with_write_cmd_info::<Profile>(id)?;

            history
                .store_profile(id, &profile)
                .await
                .with_history_write_cmd_info::<Profile>(id)?;

            cache
                .write_cache(id.as_light(), |cache| cache.profile = Some(profile.clone().into()))
                .await
                .with_history_write_cmd_info::<Profile>(id)?;
        }

        Ok(id)
    }

    pub async fn set_new_api_key(&self, id: AccountIdInternal, key: ApiKey) -> Result<(), DatabaseError> {
        let mutex = self.cache.get_write_lock_simple(self.locking_id).await.change_context(DatabaseError::Cache)?;
        let lock = mutex.lock().await;
        self.to_internal(&lock).set_new_api_key(id, key).await
    }

    pub async fn update_json<
        T: GetReadWriteCmd + Serialize + Clone + Send + SqliteUpdateJson + HistoryUpdateJson + WriteCacheJson + Sync + 'static,
    >(
        &mut self,
        id: AccountIdInternal,
        data: &T,
    ) -> Result<(), DatabaseError> {
        let mutex = self.cache.get_write_lock_simple(self.locking_id).await.change_context(DatabaseError::Cache)?;
        let lock = mutex.lock().await;
        self.to_internal(&lock).update_json(id, data).await
    }

    pub async fn save_to_slot(&self, id: AccountIdInternal, slot: ImageSlot, stream: BodyStream) -> Result<ContentId, DatabaseError> {
        let mutex = self.cache.get_write_lock_simple(self.locking_id).await.change_context(DatabaseError::Cache)?;
        let lock = mutex.lock().await;
        self.to_internal(&lock).save_to_slot(id, slot, stream).await
    }

    pub async fn set_moderation_request(&self, account_id: AccountIdInternal, request: NewModerationRequest) -> Result<(), DatabaseError> {
        let mutex = self.cache.get_write_lock_simple(self.locking_id).await.change_context(DatabaseError::Cache)?;
        let lock = mutex.lock().await;
        self.to_internal(&lock).set_moderation_request(account_id, request).await
    }

    pub async fn moderation_get_list_and_create_new_if_necessary(&self, account_id: AccountIdInternal) -> Result<Vec<Moderation>, DatabaseError> {
        let mutex = self.cache.get_write_lock_simple(self.locking_id).await.change_context(DatabaseError::Cache)?;
        let lock = mutex.lock().await;
        self.to_internal(&lock).moderation_get_list_and_create_new_if_necessary(account_id).await
    }

    async fn lock_and_run(&self, account_id: AccountIdInternal) -> Result<(), DatabaseError> {
        // TOOD: remove
        let mutex = self.cache.get_write_lock_simple(self.locking_id).await.change_context(DatabaseError::Cache)?;
        let lock = mutex.lock().await;
        let commands = self.to_internal(&lock);
        // let x = move |c: WriteCommandsInternal| c.cache;
        //let x = x(commands);

        Ok(())
    }

    fn to_internal<'b>(&'b self, lock: &'b AccountWriteLock) -> WriteCommandsInternal<'b>{
        WriteCommandsInternal::new(self.current_write, self.history_write, self.cache, self.file_dir, lock)
    }
}


struct WriteCommandsInternal<'a> {
    current_write: &'a CurrentDataWriteHandle,
    history_write: &'a HistoryWriteHandle,
    cache: &'a DatabaseCache,
    file_dir: &'a FileDir,
    lock: &'a AccountWriteLock,
}

impl <'a> WriteCommandsInternal<'a> {
    pub fn new(
        current_write: &'a CurrentDataWriteHandle,
        history_write: &'a HistoryWriteHandle,
        cache: &'a DatabaseCache,
        file_dir: &'a FileDir,
        lock: &'a AccountWriteLock,
    ) -> Self {
        Self {
            current_write,
            history_write,
            cache,
            file_dir,
            lock,
        }
    }

    pub async fn set_new_api_key(&self, id: AccountIdInternal, key: ApiKey) -> Result<(), DatabaseError> {
        self.current()
            .update_api_key(id, Some(&key))
            .await
            .with_info_lazy(|| WriteCmd::AccountId(id.as_light()))?;

        self.cache.update_api_key(id.as_light(), key)
            .await
            .with_info_lazy(|| WriteCmd::AccountId(id.as_light()))
    }

    pub async fn update_json<
        T: GetReadWriteCmd + Serialize + Clone + Send + SqliteUpdateJson + HistoryUpdateJson + WriteCacheJson + Sync + 'static,
    >(
        &mut self,
        id: AccountIdInternal,
        data: &T,
    ) -> Result<(), DatabaseError> {
        data.update_json(id, &self.current())
            .await
            .with_write_cmd_info::<T>(id)?;

        data.history_update_json(id, &self.history())
            .await
            .with_history_write_cmd_info::<T>(id)?;

        if T::CACHED_JSON {
            data.write_to_cache(id.as_light(), &self.cache)
                .await
                .with_cache_write_cmd_info::<T>(id)
        } else {
            Ok(())
        }
    }

    pub async fn save_to_slot(&self, id: AccountIdInternal, slot: ImageSlot, stream: BodyStream) -> Result<ContentId, DatabaseError> {
        let current_content_in_slot = self.current_write.read().media().get_content_id_from_slot(id, slot).await.change_context(DatabaseError::Sqlite)?;

        if let Some(current_id) = current_content_in_slot {
            let path = self.file_dir.image_content(id.as_light(), current_id.as_content_id());
            path.remove_if_exists().await.change_context(DatabaseError::File)?;
            self.current().media().delete_image_from_slot(id, slot).await.change_context(DatabaseError::Sqlite)?;
        }

        // Also clear tmp dir if previous image writing failed and there is no
        // content ID in the database about it.
        self.file_dir.tmp_dir(id.as_light()).remove_contents_if_exists().await.change_context(DatabaseError::File)?;

        let content_id = ContentId::new_random_id();
        let transaction = self.current().media().store_content_id_to_slot(id, content_id, slot).await.change_context(DatabaseError::Sqlite)?;

        let file_operations = || {
            async {
                let raw_img = self.file_dir.unprocessed_image_upload(id.as_light(), content_id);
                raw_img.save_stream(stream).await.change_context(DatabaseError::File)?;

                // TODO: real image safety checks and processing
                let processed_content_path = self.file_dir.image_content(id.as_light(), content_id);
                raw_img.move_to(&processed_content_path).await.change_context(DatabaseError::File)?;

                Ok::<ContentId, Report<DatabaseError>>(content_id)
            }
        };

        match file_operations().await {
            Ok(id) =>  {
                transaction.commit().await.change_context(DatabaseError::Sqlite).map(|_| id)
            }
            Err(e) => {
                match transaction.rollback().await.change_context(DatabaseError::Sqlite) {
                    Ok(()) => Err(e),
                    Err(another_error) => Err(another_error.attach(e)),
                }
            }
        }
    }


    pub async fn set_moderation_request(&self, account_id: AccountIdInternal, request: NewModerationRequest) -> Result<(), DatabaseError> {
        self.current()
            .media()
            .create_new_moderation_request(account_id, request)
            .await
            .with_info_lazy(|| WriteCmd::MediaModerationRequest(account_id))
    }

    pub async fn moderation_get_list_and_create_new_if_necessary(self, account_id: AccountIdInternal) -> Result<Vec<Moderation>, DatabaseError> {
        self.current()
            .media()
            .moderation_get_list_and_create_new_if_necessary(account_id)
            .await
            .with_info_lazy(|| WriteCmd::MediaModeration(account_id))
    }

    fn current(&self) -> CurrentDataWriteCommands {
        CurrentDataWriteCommands::new(&self.current_write)
    }

    fn history(&self) -> HistoryWriteCommands {
        HistoryWriteCommands::new(&self.history_write)
    }

}
