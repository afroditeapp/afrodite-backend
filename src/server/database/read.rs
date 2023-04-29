use std::{fmt::Debug, marker::PhantomData};

use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

use crate::{
    api::{model::{AccountIdInternal, AccountIdLight, ApiKey, ContentId, ModerationRequestContent}, media::data::ModerationRequest},
    utils::{ErrorConversion, ConvertCommandError},
};

use super::{
    cache::{DatabaseCache, ReadCacheJson, CacheError},
    current::SqliteReadCommands,
    file::{utils::FileDir, FileError},
    sqlite::{SqliteReadHandle, SqliteSelectJson, SqliteDatabaseError},
    DatabaseError, write::{DatabaseId, NoId},
};

use error_stack::{Result, ResultExt};

pub type ReadResult<T, Err, WriteContext = T> = std::result::Result<T, ReadError<error_stack::Report<Err>, WriteContext>>;
pub type HistoryReadResult<T, Err, WriteContext = T> = std::result::Result<T, HistoryReadError<error_stack::Report<Err>, WriteContext>>;

#[derive(Debug)]
pub struct ReadError<Err, Target = ()> {
    pub e: Err,
    pub t: PhantomData<Target>,
}

impl <Target> From<error_stack::Report<SqliteDatabaseError>> for ReadError<error_stack::Report<SqliteDatabaseError>, Target> {
    fn from(value: error_stack::Report<SqliteDatabaseError>) -> Self {
        Self { t: PhantomData, e: value }
    }
}

impl <Target> From<error_stack::Report<CacheError>> for ReadError<error_stack::Report<CacheError>, Target> {
    fn from(value: error_stack::Report<CacheError>) -> Self {
        Self { t: PhantomData, e: value }
    }
}

impl <Target> From<error_stack::Report<FileError>> for ReadError<error_stack::Report<FileError>, Target> {
    fn from(value: error_stack::Report<FileError>) -> Self {
        Self { t: PhantomData, e: value }
    }
}

impl <Target> From<SqliteDatabaseError> for ReadError<error_stack::Report<SqliteDatabaseError>, Target> {
    fn from(value: SqliteDatabaseError) -> Self {
        Self { t: PhantomData, e: value.into() }
    }
}

impl <Target> From<CacheError> for ReadError<error_stack::Report<CacheError>, Target> {
    fn from(value: CacheError) -> Self {
        Self { t: PhantomData, e: value.into() }
    }
}

impl <Target> From<FileError> for ReadError<error_stack::Report<FileError>, Target> {
    fn from(value: FileError) -> Self {
        Self { t: PhantomData, e: value.into() }
    }
}

#[derive(Debug)]
pub struct HistoryReadError<Err, Target = ()> {
    pub e: Err,
    pub t: PhantomData<Target>,
}

impl <Target> From<error_stack::Report<SqliteDatabaseError>> for HistoryReadError<error_stack::Report<SqliteDatabaseError>, Target> {
    fn from(value: error_stack::Report<SqliteDatabaseError>) -> Self {
        Self { t: PhantomData, e: value }
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
            .convert(id)?;
        self.sqlite
            .api_key(id)
            .await
            .convert(id)
    }

    pub async fn account_ids<T: FnMut(AccountIdInternal)>(
        &self,
        mut handler: T,
    ) -> Result<(), DatabaseError> {
        let mut users = self.sqlite.account_ids_stream();
        while let Some(user_id) = users.try_next().await.convert(NoId)? {
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
                .with_info_lazy(|| format!("Cache read {:?} failed, id: {:?}", PhantomData::<T>, id))
        } else {
            T::select_json(id, &self.sqlite)
                .await
                .with_info_lazy(|| format!("Read {:?} failed, id: {:?}", PhantomData::<T>, id))
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
            .convert((account_id, content_id))
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
            .convert((account_id, content_id))
    }

    pub async fn moderation_request(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Option<ModerationRequest>, DatabaseError> {
        self.sqlite
            .media()
            .current_moderation_request(account_id)
            .await
            .convert(account_id)
            .map(|r| r.map(|request| request.into_request()))
    }

    // pub async fn moderation_request_from_queue(
    //     &self,
    //     _account_id: AccountIdInternal,
    // ) -> Result<Option<ModerationRequest>, DatabaseError> {
    //     let _next_queue_number = self
    //         .sqlite
    //         .media()
    //         .get_next_active_moderation_request(0)
    //         .await
    //         .convert(_account_id)?;

    //     unimplemented!()
    // }
}
