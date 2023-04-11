use std::sync::Arc;

use axum::extract::BodyStream;
use error_stack::{Result, ResultExt};
use serde::Serialize;
use tokio::sync::{MutexGuard, Mutex};
use tokio_stream::StreamExt;

use crate::{
    api::model::{Account, AccountIdInternal, AccountSetup, ApiKey, Profile, AccountIdLight},
    config::Config,
    server::database::{sqlite::SqliteWriteHandle, DatabaseError},
    utils::{ErrorConversion, IntoReportExt},
};

use super::{
    current::write::CurrentDataWriteCommands,
    history::write::HistoryWriteCommands,
    sqlite::{HistoryUpdateJson, SqliteUpdateJson, CurrentDataWriteHandle, HistoryWriteHandle},
    utils::GetReadWriteCmd, cache::{DatabaseCache, WriteCacheJson}, file::{utils::{SlotFile, FileDir}, file::ImageSlot},
};

#[derive(Debug, Clone)]
pub enum WriteCmd {
    AccountId(AccountIdLight),
    Profile(AccountIdInternal),
    ApiKey(AccountIdInternal),
    AccountState(AccountIdInternal),
    AccountSetup(AccountIdInternal),
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

    pub async fn save_to_slot(&self, id: AccountIdInternal, slot: ImageSlot, stream: BodyStream) -> Result<(), DatabaseError> {
        let mutex = self.cache.get_write_lock_simple(self.locking_id).await.change_context(DatabaseError::Cache)?;
        let lock = mutex.lock().await;
        self.to_internal(&lock).save_to_slot(id, slot, stream).await
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

    pub async fn save_to_slot(&self, id: AccountIdInternal, slot: ImageSlot, stream: BodyStream) -> Result<(), DatabaseError> {
        let path = self.file_dir.slot(id.as_light(), slot);
        path.save_stream(stream).await.change_context(DatabaseError::File)?;
        Ok(())
    }

    fn current(&self) -> CurrentDataWriteCommands {
        CurrentDataWriteCommands::new(&self.current_write)
    }

    fn history(&self) -> HistoryWriteCommands {
        HistoryWriteCommands::new(&self.history_write)
    }

}
