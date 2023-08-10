macro_rules! define_read_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: ReadCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: ReadCommands<'a>) -> Self {
                Self { cmds }
            }

            fn db(&self) -> &database::current::read::SqliteReadCommands<'_> {
                &self.cmds.db
            }

            fn cache(&self) -> &DatabaseCache {
                &self.cmds.cache
            }

            fn files(&self) -> &FileDir {
                &self.cmds.files
            }

            pub async fn db_read<
                T: FnOnce(
                        database::current::read::CurrentSyncReadCommands<'_>,
                    )
                        -> error_stack::Result<R, database::diesel::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, crate::data::DatabaseError> {
                self.cmds.db_read(cmd).await
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

use model::{
    AccountIdInternal, AccountIdLight, ContentId, MediaContentInternal, ModerationRequest,
};

use utils::{IntoReportExt, IntoReportFromString};

use crate::utils::{ConvertCommandErrorExt, ErrorConversion};

use self::{
    account::ReadCommandsAccount, account_admin::ReadCommandsAccountAdmin, chat::ReadCommandsChat,
    chat_admin::ReadCommandsChatAdmin, media::ReadCommandsMedia,
    media_admin::ReadCommandsMediaAdmin, profile::ReadCommandsProfile,
    profile_admin::ReadCommandsProfileAdmin,
};

use database::{
    current::read::{CurrentSyncReadCommands, SqliteReadCommands},
    diesel::{DieselCurrentReadHandle, DieselDatabaseError},
    sqlite::{SqliteSelectJson, SqlxReadHandle},
};

use super::{
    cache::{DatabaseCache, ReadCacheJson},
    file::utils::FileDir,
    DatabaseError,
};

use error_stack::{Result, ResultExt};

// impl<Target> From<error_stack::Report<CacheError>>
//     for ReadError<error_stack::Report<CacheError>, Target>
// {
//     fn from(value: error_stack::Report<CacheError>) -> Self {
//         Self {
//             t: PhantomData,
//             e: value,
//         }
//     }
// }

// impl<Target> From<error_stack::Report<FileError>>
//     for ReadError<error_stack::Report<FileError>, Target>
// {
//     fn from(value: error_stack::Report<FileError>) -> Self {
//         Self {
//             t: PhantomData,
//             e: value,
//         }
//     }
// }

// impl<Target> From<CacheError> for ReadError<error_stack::Report<CacheError>, Target> {
//     fn from(value: CacheError) -> Self {
//         Self {
//             t: PhantomData,
//             e: value.into(),
//         }
//     }
// }

// impl<Target> From<FileError> for ReadError<error_stack::Report<FileError>, Target> {
//     fn from(value: FileError) -> Self {
//         Self {
//             t: PhantomData,
//             e: value.into(),
//         }
//     }
// }

pub struct ReadCommands<'a> {
    db: SqliteReadCommands<'a>,
    diesel_current_read: &'a DieselCurrentReadHandle,
    cache: &'a DatabaseCache,
    files: &'a FileDir,
}

impl<'a> ReadCommands<'a> {
    pub fn new(
        sqlite: &'a SqlxReadHandle,
        cache: &'a DatabaseCache,
        files: &'a FileDir,
        diesel_current_read: &'a DieselCurrentReadHandle,
    ) -> Self {
        Self {
            db: SqliteReadCommands::new(sqlite),
            diesel_current_read,
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

    pub async fn db_read<
        T: FnOnce(CurrentSyncReadCommands<'_>) -> Result<R, DieselDatabaseError> + Send + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> Result<R, DatabaseError> {
        let conn = self
            .diesel_current_read
            .pool()
            .get()
            .await
            .into_error(DieselDatabaseError::GetConnection)
            .change_context(DatabaseError::Diesel)?;

        conn.interact(move |conn| cmd(CurrentSyncReadCommands { conn }))
            .await
            .into_error_string(DieselDatabaseError::Execute)
            .change_context(DatabaseError::Diesel)?
            .change_context(DatabaseError::Diesel)
    }
}
