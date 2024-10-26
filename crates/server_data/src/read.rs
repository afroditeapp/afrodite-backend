use database::{
    current::read::CurrentSyncReadCommands, CurrentReadHandle, DbReader, DbReaderHistoryRaw, DbReaderRaw, DieselConnection, DieselDatabaseError, HistoryReadHandle
};

use self::common::ReadCommandsCommon;
use super::{cache::DatabaseCache, file::utils::FileDir};

macro_rules! define_read_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<C: $crate::read::ReadCommandsProvider> {
            cmds: C,
        }

        impl<C: $crate::read::ReadCommandsProvider> $struct_name<C> {
            pub fn new(cmds: C) -> Self {
                Self { cmds }
            }

            #[allow(dead_code)]
            fn cache(&self) -> &crate::DatabaseCache {
                &self.cmds.read_cmds().cache
            }

            #[allow(dead_code)]
            fn files(&self) -> &crate::FileDir {
                &self.cmds.read_cmds().files
            }

            pub async fn db_read<
                T: FnOnce(
                        database::current::read::CurrentSyncReadCommands<
                            &mut database::DieselConnection,
                        >,
                    ) -> error_stack::Result<R, database::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, database::DieselDatabaseError> {
                self.cmds.read_cmds().db_read(cmd).await
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
    pub db_history: &'a HistoryReadHandle,
    pub cache: &'a DatabaseCache,
    pub files: &'a FileDir,
}

impl<'a> ReadCommands<'a> {
    pub fn new(
        db: &'a CurrentReadHandle,
        db_history: &'a HistoryReadHandle,
        cache: &'a DatabaseCache,
        files: &'a FileDir,
    ) -> Self {
        Self {
            db,
            db_history,
            cache,
            files,
        }
    }

    pub fn common(self) -> ReadCommandsCommon<ReadCommandsContainer<'a>> {
        ReadCommandsCommon::new(ReadCommandsContainer::new(self))
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
        T: FnOnce(&mut DieselConnection) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbReaderRaw::new(self.db).db_read(cmd).await
    }

    pub async fn db_read_history_raw<
        T: FnOnce(&mut DieselConnection) -> error_stack::Result<R, DieselDatabaseError>
            + Send
            + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> error_stack::Result<R, DieselDatabaseError> {
        DbReaderHistoryRaw::new(self.db_history).db_read_history(cmd).await
    }
}

pub struct ReadCommandsContainer<'a> {
    pub cmds: ReadCommands<'a>,
}

impl<'a> ReadCommandsContainer<'a> {
    pub fn new(cmds: ReadCommands<'a>) -> Self {
        Self { cmds }
    }
}

pub trait ReadCommandsProvider {
    fn read_cmds(&self) -> &ReadCommands;
}

impl<'a> ReadCommandsProvider for ReadCommandsContainer<'a> {
    fn read_cmds(&self) -> &ReadCommands {
        &self.cmds
    }
}

impl<'a> ReadCommandsProvider for ReadCommands<'a> {
    fn read_cmds(&self) -> &ReadCommands {
        self
    }
}

pub trait GetReadCommandsCommon<C: ReadCommandsProvider> {
    fn common(self) -> ReadCommandsCommon<C>;
}

impl<C: ReadCommandsProvider> GetReadCommandsCommon<C> for C {
    fn common(self) -> ReadCommandsCommon<C> {
        ReadCommandsCommon::new(self)
    }
}
