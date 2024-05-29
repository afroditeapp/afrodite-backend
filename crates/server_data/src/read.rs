use database::{current::read::CurrentSyncReadCommands, CurrentReadHandle, DbReader, DbReaderRaw, DieselConnection, DieselDatabaseError};
use model::{
    AccountId, AccountIdInternal, ContentId, MediaContentRaw, ModerationRequest,
    ModerationRequestState,
};
use server_common::data::DataError;
use tokio_util::io::ReaderStream;

use self::{
    common::ReadCommandsCommon,
};
use super::{cache::DatabaseCache, file::utils::FileDir, IntoDataError};
use crate::result::Result;

macro_rules! define_read_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: crate::read::ReadCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: crate::read::ReadCommands<'a>) -> Self {
                Self { cmds }
            }

            #[allow(dead_code)]
            fn cache(&self) -> &crate::DatabaseCache {
                &self.cmds.cache
            }

            #[allow(dead_code)]
            fn files(&self) -> &crate::FileDir {
                &self.cmds.files
            }

            pub async fn db_read<
                T: FnOnce(
                        database::current::read::CurrentSyncReadCommands<
                            &mut database::DieselConnection,
                        >,
                    ) -> error_stack::Result<
                        R,
                        database::DieselDatabaseError,
                    > + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, database::DieselDatabaseError>
            {
                self.cmds.db_read(cmd).await
            }

            // TODO: change cache operation to return Result?
            pub async fn read_cache<T, Id: Into<model::AccountId>>(
                &self,
                id: Id,
                cache_operation: impl Fn(&crate::cache::CacheEntry) -> T,
            ) -> error_stack::Result<T, crate::CacheError> {
                self.cache().read_cache(id, cache_operation).await
            }
        }
    };
}

pub mod common;

pub struct ReadCommands<'a> {
    pub db: &'a CurrentReadHandle,
    pub cache: &'a DatabaseCache,
    pub files: &'a FileDir,
}

impl<'a> ReadCommands<'a> {
    pub fn new(
        current_read_handle: &'a CurrentReadHandle,
        cache: &'a DatabaseCache,
        files: &'a FileDir,
    ) -> Self {
        Self {
            db: current_read_handle,
            cache,
            files,
        }
    }

    pub fn common(self) -> ReadCommandsCommon<'a> {
        ReadCommandsCommon::new(self)
    }

    pub async fn db_read<
        T: FnOnce(
                CurrentSyncReadCommands<&mut DieselConnection>,
            ) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbReader::new(self.db).db_read(cmd).await
    }

    pub async fn db_read_raw<
        T: FnOnce(
                &mut DieselConnection,
            ) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbReaderRaw::new(self.db).db_read(cmd).await
    }
}


// pub fn account(self) -> ReadCommandsAccount<'a> {
//     ReadCommandsAccount::new(self)
// }

// pub fn account_admin(self) -> ReadCommandsAccountAdmin<'a> {
//     ReadCommandsAccountAdmin::new(self)
// }

// pub fn media(self) -> ReadCommandsMedia<'a> {
//     ReadCommandsMedia::new(self)
// }

// pub fn media_admin(self) -> ReadCommandsMediaAdmin<'a> {
//     ReadCommandsMediaAdmin::new(self)
// }

// pub fn profile(self) -> ReadCommandsProfile<'a> {
//     ReadCommandsProfile::new(self)
// }

// pub fn profile_admin(self) -> ReadCommandsProfileAdmin<'a> {
//     ReadCommandsProfileAdmin::new(self)
// }

// pub fn chat(self) -> ReadCommandsChat<'a> {
//     ReadCommandsChat::new(self)
// }

// pub fn chat_admin(self) -> ReadCommandsChatAdmin<'a> {
//     ReadCommandsChatAdmin::new(self)
// }
