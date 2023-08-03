macro_rules! define_read_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: ReadCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: ReadCommands<'a>) -> Self {
                Self { cmds }
            }

            fn db(&self) -> &SqliteReadCommands<'_> {
                &self.cmds.db
            }

            fn cache(&self) -> &DatabaseCache {
                &self.cmds.cache
            }

            fn files(&self) -> &FileDir {
                &self.cmds.files
            }
        }
    };
}

pub mod account;
pub mod account_admin;
pub mod chat;
pub mod chat_admin;
pub mod media;
pub mod media_admin;
pub mod profile;
pub mod profile_admin;

use std::{fmt::Debug, marker::PhantomData};

use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

use crate::{
    api::{
        media::data::{MediaContentInternal, ModerationRequest},
        model::{AccountIdInternal, AccountIdLight, ContentId},
    },
    utils::{ConvertCommandError, ErrorConversion},
};

use self::{
    account::ReadCommandsAccount, account_admin::ReadCommandsAccountAdmin, chat::ReadCommandsChat,
    chat_admin::ReadCommandsChatAdmin, media::ReadCommandsMedia,
    media_admin::ReadCommandsMediaAdmin, profile::ReadCommandsProfile,
    profile_admin::ReadCommandsProfileAdmin,
};

use super::{
    cache::{CacheError, DatabaseCache, ReadCacheJson},
    database::sqlite::{SqliteDatabaseError, SqliteReadHandle, SqliteSelectJson},
    file::{utils::FileDir, FileError},
    DatabaseError,
};

use crate::server::data::database::current::SqliteReadCommands;
use error_stack::Result;

pub type ReadResult<T, Err, WriteContext = T> =
    std::result::Result<T, ReadError<error_stack::Report<Err>, WriteContext>>;
pub type HistoryReadResult<T, Err, WriteContext = T> =
    std::result::Result<T, HistoryReadError<error_stack::Report<Err>, WriteContext>>;

#[derive(Debug)]
pub struct ReadError<Err, Target = ()> {
    pub e: Err,
    pub t: PhantomData<Target>,
}

impl<Target> From<error_stack::Report<SqliteDatabaseError>>
    for ReadError<error_stack::Report<SqliteDatabaseError>, Target>
{
    fn from(value: error_stack::Report<SqliteDatabaseError>) -> Self {
        Self {
            t: PhantomData,
            e: value,
        }
    }
}

impl<Target> From<error_stack::Report<CacheError>>
    for ReadError<error_stack::Report<CacheError>, Target>
{
    fn from(value: error_stack::Report<CacheError>) -> Self {
        Self {
            t: PhantomData,
            e: value,
        }
    }
}

impl<Target> From<error_stack::Report<FileError>>
    for ReadError<error_stack::Report<FileError>, Target>
{
    fn from(value: error_stack::Report<FileError>) -> Self {
        Self {
            t: PhantomData,
            e: value,
        }
    }
}

impl<Target> From<SqliteDatabaseError>
    for ReadError<error_stack::Report<SqliteDatabaseError>, Target>
{
    fn from(value: SqliteDatabaseError) -> Self {
        Self {
            t: PhantomData,
            e: value.into(),
        }
    }
}

impl<Target> From<CacheError> for ReadError<error_stack::Report<CacheError>, Target> {
    fn from(value: CacheError) -> Self {
        Self {
            t: PhantomData,
            e: value.into(),
        }
    }
}

impl<Target> From<FileError> for ReadError<error_stack::Report<FileError>, Target> {
    fn from(value: FileError) -> Self {
        Self {
            t: PhantomData,
            e: value.into(),
        }
    }
}

#[derive(Debug)]
pub struct HistoryReadError<Err, Target = ()> {
    pub e: Err,
    pub t: PhantomData<Target>,
}

impl<Target> From<error_stack::Report<SqliteDatabaseError>>
    for HistoryReadError<error_stack::Report<SqliteDatabaseError>, Target>
{
    fn from(value: error_stack::Report<SqliteDatabaseError>) -> Self {
        Self {
            t: PhantomData,
            e: value,
        }
    }
}

pub struct ReadCommands<'a> {
    db: SqliteReadCommands<'a>,
    cache: &'a DatabaseCache,
    files: &'a FileDir,
}

impl<'a> ReadCommands<'a> {
    pub fn new(sqlite: &'a SqliteReadHandle, cache: &'a DatabaseCache, files: &'a FileDir) -> Self {
        Self {
            db: SqliteReadCommands::new(sqlite),
            cache,
            files,
        }
    }

    pub fn account(self) -> ReadCommandsAccount<'a> {
        ReadCommandsAccount::new(self)
    }

    pub fn account_admin(self) -> ReadCommandsAccountAdmin<'a> {
        ReadCommandsAccountAdmin::new(self)
    }

    pub fn media(self) -> ReadCommandsMedia<'a> {
        ReadCommandsMedia::new(self)
    }

    pub fn media_admin(self) -> ReadCommandsMediaAdmin<'a> {
        ReadCommandsMediaAdmin::new(self)
    }

    pub fn profile(self) -> ReadCommandsProfile<'a> {
        ReadCommandsProfile::new(self)
    }

    pub fn profile_admin(self) -> ReadCommandsProfileAdmin<'a> {
        ReadCommandsProfileAdmin::new(self)
    }

    pub fn chat(self) -> ReadCommandsChat<'a> {
        ReadCommandsChat::new(self)
    }

    pub fn chat_admin(self) -> ReadCommandsChatAdmin<'a> {
        ReadCommandsChatAdmin::new(self)
    }

    pub async fn read_json<T: SqliteSelectJson + Debug + ReadCacheJson + Send + Sync + 'static>(
        &self,
        id: AccountIdInternal,
    ) -> Result<T, DatabaseError> {
        if T::CACHED_JSON {
            T::read_from_cache(id.as_light(), self.cache)
                .await
                .with_info_lazy(|| {
                    format!("Cache read {:?} failed, id: {:?}", PhantomData::<T>, id)
                })
        } else {
            T::select_json(id, &self.db)
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

    pub async fn all_account_media(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Vec<MediaContentInternal>, DatabaseError> {
        self.db
            .media()
            .get_account_media(account_id)
            .await
            .convert(account_id)
    }

    pub async fn moderation_request(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Option<ModerationRequest>, DatabaseError> {
        self.db
            .media()
            .current_moderation_request(account_id)
            .await
            .convert(account_id)
            .map(|r| r.map(|request| request.into_request()))
    }

    pub async fn profile_visibility(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Option<bool>, DatabaseError> {
        self.cache
            .read_cache(account_id.as_light(), |e| {
                e.profile.as_ref().map(|p| p.public).flatten()
            })
            .await
            .convert(account_id)
    }
}
