//! Synchronous write commands combining cache and database operations.

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use error_stack::{Result, ResultExt};

use config::Config;
use database::{
    current::{write::{CurrentSyncWriteCommands, CurrentWriteCommands, WriteCmdsMethods, TransactionConnection}, read::CurrentSyncReadCommands},
    diesel::{DieselCurrentWriteHandle, DieselDatabaseError, DieselHistoryWriteHandle, DieselConnection},
    history::write::HistoryWriteCommands,
    sqlite::{CurrentDataWriteHandle, HistoryUpdateJson, HistoryWriteHandle, SqliteUpdateJson}, TransactionError, PoolObject,
};
use model::{Account, AccountIdInternal, AccountIdLight, AccountSetup, SignInWithInfo};
use utils::{IntoReportExt, IntoReportFromString};

use crate::{
    data::DatabaseError,
    media_backup::MediaBackupHandle,
    utils::{ErrorConversion},
};

use super::{
    cache::{DatabaseCache, WriteCacheJson},
    file::utils::FileDir,
    index::{LocationIndexWriterGetter, LocationIndexIteratorGetter},
};

use self::{
    account::WriteCommandsAccount, account_admin::WriteCommandsAccountAdmin,
    chat::WriteCommandsChat, chat_admin::WriteCommandsChatAdmin, common::WriteCommandsCommon,
    media::WriteCommandsMedia, media_admin::WriteCommandsMediaAdmin, profile::WriteCommandsProfile,
    profile_admin::WriteCommandsProfileAdmin,
};

macro_rules! define_write_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: super::WriteCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: super::WriteCommands<'a>) -> Self {
                Self { cmds }
            }

            #[allow(dead_code)]
            fn current_write(&self) -> &database::sqlite::CurrentDataWriteHandle {
                &self.cmds.current_write
            }

            #[allow(dead_code)]
            fn history_write(&self) -> &database::sqlite::HistoryWriteHandle {
                &self.cmds.history_write
            }

            #[allow(dead_code)]
            fn cache(&self) -> &super::super::cache::DatabaseCache {
                &self.cmds.cache
            }

            #[allow(dead_code)]
            fn file_dir(&self) -> &super::super::FileDir {
                &self.cmds.file_dir
            }

            #[allow(dead_code)]
            fn location(&self) -> &super::super::index::LocationIndexWriterGetter<'a> {
                &self.cmds.location
            }

            #[allow(dead_code)]
            fn media_backup(&self) -> &crate::media_backup::MediaBackupHandle {
                &self.cmds.media_backup
            }

            #[allow(dead_code)]
            fn current(&self) -> database::current::write::CurrentWriteCommands {
                database::current::write::CurrentWriteCommands::new(self.current_write())
            }

            #[allow(dead_code)]
            fn history(&self) -> super::super::write::HistoryWriteCommands {
                super::super::write::HistoryWriteCommands::new(&self.history_write())
            }

            #[track_caller]
            pub async fn db_write<
                T: FnOnce(
                        database::current::write::CurrentSyncWriteCommands<'_>,
                    )
                        -> error_stack::Result<R, database::diesel::DieselDatabaseError>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, crate::data::DatabaseError> {
                self.cmds.db_write(cmd).await
            }

            #[track_caller]
            pub async fn db_transaction<
                T: FnOnce(
                        &mut database::diesel::DieselConnection,
                    )
                        -> std::result::Result<R, database::TransactionError<database::diesel::DieselDatabaseError>>
                    + Send
                    + 'static,
                R: Send + 'static,
            >(
                &self,
                cmd: T,
            ) -> error_stack::Result<R, crate::data::DatabaseError> {
                self.cmds.db_transaction(cmd).await
            }

            #[track_caller]
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

            #[track_caller]
            pub async fn write_cache<T, Id: Into<model::AccountIdLight>>(
                &self,
                id: Id,
                cache_operation: impl FnOnce(&mut crate::data::cache::CacheEntry) -> error_stack::Result<T, crate::data::CacheError>,
            ) -> error_stack::Result<T, crate::data::DatabaseError> {
                use error_stack::ResultExt;
                self.cache().write_cache(id, cache_operation).await.change_context(crate::data::DatabaseError::Cache)
            }
        }
    };
}

pub mod account;
pub mod account_admin;
pub mod chat;
pub mod chat_admin;
pub mod common;
pub mod media;
pub mod media_admin;
pub mod profile;
pub mod profile_admin;

// impl<Target> From<error_stack::Report<CacheError>>
//     for WriteError<error_stack::Report<CacheError>, Target>
// {
//     fn from(value: error_stack::Report<CacheError>) -> Self {
//         Self {
//             t: PhantomData,
//             e: value,
//         }
//     }
// }

// impl<Target> From<CacheError> for WriteError<error_stack::Report<CacheError>, Target> {
//     fn from(value: CacheError) -> Self {
//         Self {
//             t: PhantomData,
//             e: value.into(),
//         }
//     }
// }

// TODO: If one commands does multiple writes to database, move writes to happen
// in a transaction.

// TODO: When server starts, check that latest history data matches with current
// data.

/// One Account can do only one write command at a time.
pub struct AccountWriteLock;

/// Globally synchronous write commands.
pub struct WriteCommands<'a> {
    config: &'a Arc<Config>,
    current_write: &'a CurrentDataWriteHandle,
    history_write: &'a HistoryWriteHandle,
    diesel_current_write: &'a DieselCurrentWriteHandle,
    diesel_history_write: &'a DieselHistoryWriteHandle,
    cache: &'a DatabaseCache,
    file_dir: &'a FileDir,
    location_iterator: LocationIndexIteratorGetter<'a>,
    location: LocationIndexWriterGetter<'a>,
    media_backup: &'a MediaBackupHandle,
}

impl<'a> WriteCommands<'a> {
    pub fn new(
        config: &'a Arc<Config>,
        current_write: &'a CurrentDataWriteHandle,
        history_write: &'a HistoryWriteHandle,
        diesel_current_write: &'a DieselCurrentWriteHandle,
        diesel_history_write: &'a DieselHistoryWriteHandle,
        cache: &'a DatabaseCache,
        file_dir: &'a FileDir,
        location_iterator: LocationIndexIteratorGetter<'a>,
        location: LocationIndexWriterGetter<'a>,
        media_backup: &'a MediaBackupHandle,
    ) -> Self {
        Self {
            config,
            current_write,
            history_write,
            diesel_current_write,
            diesel_history_write,
            cache,
            file_dir,
            location_iterator,
            location,
            media_backup,
        }
    }

    pub fn common(self) -> WriteCommandsCommon<'a> {
        WriteCommandsCommon::new(self)
    }

    pub fn account(self) -> WriteCommandsAccount<'a> {
        WriteCommandsAccount::new(self)
    }

    pub fn account_admin(self) -> WriteCommandsAccountAdmin<'a> {
        WriteCommandsAccountAdmin::new(self)
    }

    pub fn media(self) -> WriteCommandsMedia<'a> {
        WriteCommandsMedia::new(self)
    }

    pub fn media_admin(self) -> WriteCommandsMediaAdmin<'a> {
        WriteCommandsMediaAdmin::new(self)
    }

    pub fn profile(self) -> WriteCommandsProfile<'a> {
        WriteCommandsProfile::new(self)
    }

    pub fn profile_admin(self) -> WriteCommandsProfileAdmin<'a> {
        WriteCommandsProfileAdmin::new(self)
    }

    pub fn chat(self) -> WriteCommandsChat<'a> {
        WriteCommandsChat::new(self)
    }

    pub fn chat_admin(self) -> WriteCommandsChatAdmin<'a> {
        WriteCommandsChatAdmin::new(self)
    }

    pub async fn register(
        &self,
        id_light: AccountIdLight,
        sign_in_with_info: SignInWithInfo,
    ) -> Result<AccountIdInternal, DatabaseError> {
        let account = Account::default();
        let account_setup = AccountSetup::default();

        let config = self.config.clone();
        let id = self.db_transaction_with_history_REMOVE(move |conn, _history_conn|{
            // let mut conn1 = CurrentSyncWriteCommands::new(conn);
            // let conn = &mut conn1;
            let id = conn
                .into_account()
                .insert_account_id(id_light)?;

            // let mut current = CurrentSyncWriteCommands::new(conn);

            conn
                .into_account()
                .insert_access_token(id, None)?;
            conn
                .into_account()
                .insert_refresh_token(id, None)?;

            if config.components().account {
                conn
                    .into_account()
                    .insert_account(id, &account)?;
                conn
                    .into_account()
                    .insert_account_setup(id, &account_setup)?;
                conn
                    .into_account()
                    .insert_sign_in_with_info(id, &sign_in_with_info)?;
            }

            if config.components().profile {
                conn.into_profile().insert_profile(id)?;
            }

            if config.components().media {
                conn
                    .into_media()
                    .insert_current_account_media(id)?;
            }

            // TODO: write to history
            /*
            history.store_account_id(id).await.convert(id)?;
            if config.components().account {
                history.store_account(id, &account).await.convert(id)?;
                history
                    .store_account_setup(id, &account_setup)
                    .await
                    .convert(id)?;
            }

            if config.components().profile {
                // TOOD: update history code
                // history
                //     .store_profile(id, &profile)
                //     .await
                //     .with_history_write_cmd_info::<Profile>(id)?;

            }

            if config.components().media {

            }

             */
            Ok(id.clone())
        }).await?;

        self.cache.load_account_from_db(
            id,
            &self.config,
            &self.diesel_current_write.to_read_handle(),
            &self.location_iterator,
            &self.location,
        ).await.with_info(id)?;

        Ok(id)
    }

    pub async fn update_data<
        T: Clone
            + Debug
            + Send
            + SqliteUpdateJson
            + HistoryUpdateJson
            + WriteCacheJson
            + Sync
            + 'static,
    >(
        &mut self,
        id: AccountIdInternal,
        data: &T,
    ) -> Result<(), DatabaseError> {
        data.update_json(id, &self.current())
            .await
            .with_info_lazy(|| format!("Update {:?} failed, id: {:?}", PhantomData::<T>, id))?;

        // Empty implementation if not really cacheable.
        data.write_to_cache(id.as_light(), &self.cache)
            .await
            .with_info_lazy(|| {
                format!("Cache update {:?} failed, id: {:?}", PhantomData::<T>, id)
            })?;

        data.history_update_json(id, &self.history())
            .await
            .with_info_lazy(|| {
                format!("History update {:?} failed, id: {:?}", PhantomData::<T>, id)
            })
    }

    fn current(&self) -> CurrentWriteCommands {
        CurrentWriteCommands::new(&self.current_write)
    }

    fn history(&self) -> HistoryWriteCommands {
        HistoryWriteCommands::new(&self.history_write)
    }

    #[track_caller]
    pub async fn db_write<
        T: FnOnce(CurrentSyncWriteCommands<'_>) -> Result<R, DieselDatabaseError> + Send + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> Result<R, DatabaseError> {
        let conn = self
            .diesel_current_write
            .pool()
            .get()
            .await
            .into_error(DieselDatabaseError::GetConnection)
            .change_context(DatabaseError::Diesel)?;

        conn.interact(move |conn| cmd(CurrentSyncWriteCommands::new(conn)))
            .await
            .into_error_string(DieselDatabaseError::Execute)
            .change_context(DatabaseError::Diesel)?
            .change_context(DatabaseError::Diesel)
    }

    #[track_caller]
    pub async fn db_transaction<
        T: FnOnce(&mut database::diesel::DieselConnection) -> std::result::Result<R, TransactionError<DieselDatabaseError>> + Send + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> Result<R, DatabaseError> {
        let conn = self
            .diesel_current_write
            .pool()
            .get()
            .await
            .into_error(DieselDatabaseError::GetConnection)
            .change_context(DatabaseError::Diesel)?;

        conn.interact(move |conn| {
                CurrentSyncWriteCommands::new(conn).transaction(cmd)
            })
            .await
            .into_error_string(DieselDatabaseError::Execute)
            .change_context(DatabaseError::Diesel)?
            .change_context(DatabaseError::Diesel)
    }

    #[track_caller]
    pub async fn db_transaction_with_history_REMOVE<
        T,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> Result<R, DatabaseError> where
    T: FnOnce(&mut DieselConnection, PoolObject) -> std::result::Result<R, TransactionError<DieselDatabaseError>> + Send + 'static{
        let conn = self
            .diesel_current_write
            .pool()
            .get()
            .await
            .into_error(DieselDatabaseError::GetConnection)
            .change_context(DatabaseError::Diesel)?;

        let conn_history = self
            .diesel_history_write
            .pool()
            .get()
            .await
            .into_error(DieselDatabaseError::GetConnection)
            .change_context(DatabaseError::Diesel)?;

        conn.interact(move |conn| {
                CurrentSyncWriteCommands::new(conn)
                    .transaction(move |conn| {
                        //let mut transaction_connection = TransactionConnection { conn };
                        cmd(conn, conn_history)
                    })
            })
            .await
            .into_error_string(DieselDatabaseError::Execute)
            .change_context(DatabaseError::Diesel)?
            .change_context(DatabaseError::Diesel)
    }


    #[track_caller]
    pub async fn db_transaction_with_history<
        // 'a1,
        // 'b1: 'a1,
        // C: WriteCmdsMethods<'a1, 'b1>,
        T,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> Result<R, DatabaseError> where
    T: for<'b1> FnOnce(&mut TransactionConnection<'b1>, PoolObject) -> std::result::Result<R, TransactionError<DieselDatabaseError>> + Send + 'static,
     {
        let conn = self
            .diesel_current_write
            .pool()
            .get()
            .await
            .into_error(DieselDatabaseError::GetConnection)
            .change_context(DatabaseError::Diesel)?;

        let conn_history = self
            .diesel_history_write
            .pool()
            .get()
            .await
            .into_error(DieselDatabaseError::GetConnection)
            .change_context(DatabaseError::Diesel)?;

        conn.interact(move |conn| {
                CurrentSyncWriteCommands::new(conn)
                    .transaction(move |conn| {
                        let mut transaction_connection = TransactionConnection { conn };
                        cmd(&mut transaction_connection, conn_history)
                    })
            })
            .await
            .into_error_string(DieselDatabaseError::Execute)
            .change_context(DatabaseError::Diesel)?
            .change_context(DatabaseError::Diesel)
    }

    #[track_caller]
    pub async fn db_read<
        T: FnOnce(CurrentSyncReadCommands<'_>) -> Result<R, DieselDatabaseError> + Send + 'static,
        R: Send + 'static,
    >(
        &self,
        cmd: T,
    ) -> Result<R, DatabaseError> {
        let conn = self
            .diesel_current_write
            .pool()
            .get()
            .await
            .into_error(DieselDatabaseError::GetConnection)
            .change_context(DatabaseError::Diesel)?;

        conn.interact(move |conn| cmd(CurrentSyncReadCommands::new(conn)))
            .await
            .into_error_string(DieselDatabaseError::Execute)
            .change_context(DatabaseError::Diesel)?
            .change_context(DatabaseError::Diesel)
    }
}
