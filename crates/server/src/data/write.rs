//! Synchronous write commands combining cache and database operations.

macro_rules! define_write_commands {
    ($struct_name:ident) => {
        pub struct $struct_name<'a> {
            cmds: super::WriteCommands<'a>,
        }

        impl<'a> $struct_name<'a> {
            pub fn new(cmds: super::WriteCommands<'a>) -> Self {
                Self { cmds }
            }

            fn current_write(&self) -> &database::sqlite::CurrentDataWriteHandle {
                &self.cmds.current_write
            }

            fn history_write(&self) -> &database::sqlite::HistoryWriteHandle {
                &self.cmds.history_write
            }

            fn cache(&self) -> &super::super::cache::DatabaseCache {
                &self.cmds.cache
            }

            fn file_dir(&self) -> &super::super::FileDir {
                &self.cmds.file_dir
            }

            fn location(&self) -> &super::super::index::LocationIndexWriterGetter<'a> {
                &self.cmds.location
            }

            fn media_backup(&self) -> &crate::media_backup::MediaBackupHandle {
                &self.cmds.media_backup
            }

            fn current(&self) -> database::current::write::CurrentWriteCommands {
                database::current::write::CurrentWriteCommands::new(self.current_write())
            }

            fn history(&self) -> super::super::write::HistoryWriteCommands {
                super::super::write::HistoryWriteCommands::new(&self.history_write())
            }

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

use std::{fmt::Debug, marker::PhantomData};

use config::Config;
use database::{
    current::write::{CurrentSyncWriteCommands, CurrentWriteCommands},
    diesel::{DieselCurrentWriteHandle, DieselDatabaseError, DieselHistoryWriteHandle},
    history::write::HistoryWriteCommands,
    sqlite::{CurrentDataWriteHandle, HistoryUpdateJson, HistoryWriteHandle, SqliteUpdateJson},
};
use error_stack::{Result, ResultExt};
use model::{Account, AccountIdInternal, AccountIdLight, AccountSetup, SignInWithInfo};
use utils::{IntoReportExt, IntoReportFromString};

use self::{
    account::WriteCommandsAccount, account_admin::WriteCommandsAccountAdmin,
    chat::WriteCommandsChat, chat_admin::WriteCommandsChatAdmin, common::WriteCommandsCommon,
    media::WriteCommandsMedia, media_admin::WriteCommandsMediaAdmin, profile::WriteCommandsProfile,
    profile_admin::WriteCommandsProfileAdmin,
};
use super::{
    cache::{CachedProfile, DatabaseCache, WriteCacheJson},
    file::utils::FileDir,
    index::LocationIndexWriterGetter,
};
use crate::{
    data::DatabaseError,
    media_backup::MediaBackupHandle,
    utils::{ConvertCommandErrorExt, ErrorConversion},
};

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
    config: &'a Config,
    current_write: &'a CurrentDataWriteHandle,
    history_write: &'a HistoryWriteHandle,
    diesel_current_write: &'a DieselCurrentWriteHandle,
    diesel_history_write: &'a DieselHistoryWriteHandle,
    cache: &'a DatabaseCache,
    file_dir: &'a FileDir,
    location: LocationIndexWriterGetter<'a>,
    media_backup: &'a MediaBackupHandle,
}

impl<'a> WriteCommands<'a> {
    pub fn new(
        config: &'a Config,
        current_write: &'a CurrentDataWriteHandle,
        history_write: &'a HistoryWriteHandle,
        diesel_current_write: &'a DieselCurrentWriteHandle,
        diesel_history_write: &'a DieselHistoryWriteHandle,
        cache: &'a DatabaseCache,
        file_dir: &'a FileDir,
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
        Self::register_static(
            id_light,
            sign_in_with_info,
            &self.config,
            self.current_write.clone(),
            self.history_write.clone(),
            self.cache,
        )
        .await
    }

    pub async fn register_static(
        id_light: AccountIdLight,
        sign_in_with_info: SignInWithInfo,
        config: &Config,
        current_data_write: CurrentDataWriteHandle,
        history_wirte: HistoryWriteHandle,
        cache: &DatabaseCache,
    ) -> Result<AccountIdInternal, DatabaseError> {
        let current = CurrentWriteCommands::new(&current_data_write);
        let history = HistoryWriteCommands::new(&history_wirte);

        let account = Account::default();
        let account_setup = AccountSetup::default();

        // TODO: Use transactions here. One for current and other for history.

        let id = current
            .account()
            .store_account_id(id_light)
            .await
            .convert(id_light)?;

        history.store_account_id(id).await.convert(id)?;

        cache.insert_account_if_not_exists(id).await.convert(id)?;

        current
            .account()
            .store_api_key(id, None)
            .await
            .convert(id)?;
        current
            .account()
            .store_refresh_token(id, None)
            .await
            .convert(id)?;

        if config.components().account {
            current
                .account()
                .store_account(id, &account)
                .await
                .convert(id)?;

            history.store_account(id, &account).await.convert(id)?;

            cache
                .write_cache(id.as_light(), |cache| {
                    cache.account = Some(account.clone().into());
                    Ok(())
                })
                .await
                .convert(id)?;

            current
                .account()
                .store_account_setup(id, &account_setup)
                .await
                .convert(id)?;

            history
                .store_account_setup(id, &account_setup)
                .await
                .convert(id)?;

            current
                .account()
                .store_sign_in_with_info(id, &sign_in_with_info)
                .await
                .convert(id)?;
        }

        if config.components().profile {
            let profile = current.profile().init_profile(id).await.convert(id)?;

            // TOOD: update history code
            // history
            //     .store_profile(id, &profile)
            //     .await
            //     .with_history_write_cmd_info::<Profile>(id)?;

            cache
                .write_cache(id.as_light(), |cache| {
                    let p: CachedProfile = profile.into();
                    cache.profile = Some(p.into());
                    Ok(())
                })
                .await
                .convert(id)?;
        }

        if config.components().media {
            current
                .media()
                .init_current_account_media(id)
                .await
                .convert(id)?;
        }

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
}
