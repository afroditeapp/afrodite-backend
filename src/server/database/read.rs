use std::{fmt::Debug, marker::PhantomData};

use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

use crate::{
    api::{model::{AccountIdInternal, AccountIdLight, ApiKey, ContentId, ModerationRequestContent}, media::data::ModerationRequest},
    utils::ErrorConversion,
};

use super::{
    cache::{DatabaseCache, ReadCacheJson},
    current::SqliteReadCommands,
    file::utils::FileDir,
    sqlite::{SqliteReadHandle, SqliteSelectJson},
    DatabaseError, write::{DatabaseId, NoId},
};

use error_stack::{Result, ResultExt};

// #[derive(Debug, Clone)]
// pub enum ReadCmd {
//     AccountApiKey(AccountIdInternal),
//     AccountState(AccountIdInternal),
//     AccountSetup(AccountIdInternal),
//     Accounts,
//     Profile(AccountIdInternal),
// }

// impl std::fmt::Display for ReadCmd {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.write_fmt(format_args!("Read command: {:?}", self))
//     }
// }

#[derive(Debug, Clone)]
pub struct ReadCmd<T: Debug>(DatabaseId, PhantomData<T>);

impl <T: Debug> ReadCmd<T> {
    pub fn new(id: impl Into<DatabaseId>) -> Self {
        Self(id.into(), PhantomData)
    }
}

impl <T: Debug> std::fmt::Display for ReadCmd<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Read command: {:?}", self))
    }
}

#[derive(Debug, Clone)]
pub struct HistoryRead<T: Debug>(DatabaseId, PhantomData<T>);

impl <T: Debug> HistoryRead<T> {
    pub fn new(id: impl Into<DatabaseId>) -> Self {
        Self(id.into(), PhantomData)
    }
}

impl <T: Debug> std::fmt::Display for HistoryRead<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("History read command: {:?}", self))
    }
}

#[derive(Debug, Clone)]
pub struct CacheRead<T: Debug>(DatabaseId, PhantomData<T>);

impl <T: Debug> CacheRead<T> {
    pub fn new(id: impl Into<DatabaseId>) -> Self {
        Self(id.into(), PhantomData)
    }
}

impl <T: Debug> std::fmt::Display for CacheRead<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Cache write command: {:?}", self))
    }
}


pub struct ReadCommands<'a> {
    sqlite: SqliteReadCommands<'a>,
    cache: &'a DatabaseCache,
    files: &'a FileDir,
}

impl<'a> ReadCommands<'a> {
    pub fn new(sqlite: &'a SqliteReadHandle, cache: &'a DatabaseCache, files: &'a FileDir) -> Self {
        Self {
            sqlite: SqliteReadCommands::new(sqlite),
            cache,
            files,
        }
    }

    pub async fn user_api_key(&self, id: AccountIdLight) -> Result<Option<ApiKey>, DatabaseError> {
        let id = self
            .cache
            .to_account_id_internal(id)
            .await
            .change_context(DatabaseError::Cache)?;
        self.sqlite
            .api_key(id)
            .await
            .change_context(DatabaseError::Sqlite)
    }

    pub async fn account_ids<T: FnMut(AccountIdInternal)>(
        &self,
        mut handler: T,
    ) -> Result<(), DatabaseError> {
        let mut users = self.sqlite.account_ids_stream();
        while let Some(user_id) = users.try_next().await.with_info(ReadCmd::<AccountIdInternal>::new(NoId))? {
            handler(user_id)
        }

        Ok(())
    }

    pub async fn read_json<T: SqliteSelectJson + Debug + ReadCacheJson + Send + Sync + 'static>(
        &self,
        id: AccountIdInternal,
    ) -> Result<T, DatabaseError> {
        if T::CACHED_JSON {
            T::read_from_cache(id.as_light(), self.cache)
                .await
                .with_info_lazy(|| CacheRead::<T>::new(id))
        } else {
            T::select_json(id, &self.sqlite)
                .await
                .with_info_lazy(|| ReadCmd::<T>::new(id))
        }
    }

    pub async fn image_stream(
        &self,
        account_id: AccountIdLight,
        content_id: ContentId,
    ) -> Result<ReaderStream<tokio::fs::File>, DatabaseError> {
        self.files
            .image_content(account_id, content_id)
            .read_stream()
            .await
            .change_context(DatabaseError::File)
    }

    pub async fn image(
        &self,
        account_id: AccountIdLight,
        content_id: ContentId,
    ) -> Result<Vec<u8>, DatabaseError> {
        self.files
            .image_content(account_id, content_id)
            .read_all()
            .await
            .change_context(DatabaseError::File)
    }

    pub async fn moderation_request(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Option<ModerationRequest>, DatabaseError> {
        self.sqlite
            .media()
            .current_moderation_request(account_id)
            .await
            .change_context(DatabaseError::Sqlite)
            .map(|r| r.map(|request| request.into_request()))
    }

    pub async fn moderation_request_from_queue(
        &self,
        _account_id: AccountIdInternal,
    ) -> Result<Option<ModerationRequest>, DatabaseError> {
        let _next_queue_number = self
            .sqlite
            .media()
            .get_next_active_moderation_request(0)
            .await
            .change_context(DatabaseError::Sqlite)?;

        unimplemented!()
    }
}
