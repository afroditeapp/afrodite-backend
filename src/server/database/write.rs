use error_stack::Result;
use serde::Serialize;

use crate::{
    api::model::{Account, AccountIdInternal, AccountSetup, ApiKey, Profile, AccountIdLight},
    config::Config,
    server::database::{sqlite::SqliteWriteHandle, DatabaseError},
    utils::ErrorConversion,
};

use super::{
    current::write::CurrentDataWriteCommands,
    history::write::HistoryWriteCommands,
    sqlite::{HistoryUpdateJson, SqliteUpdateJson, CurrentDataWriteHandle, HistoryWriteHandle},
    utils::GetReadWriteCmd,
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
        f.write_fmt(format_args!("Write command: {:?}", self))
    }
}


pub struct WriteCommands<'a> {
    current_write: &'a CurrentDataWriteHandle,
    history_write: &'a HistoryWriteHandle,
}

impl <'a> WriteCommands<'a> {
    pub fn new(
        current_write: &'a CurrentDataWriteHandle,
        history_write: &'a HistoryWriteHandle,
    ) -> Self {
        Self {
            current_write,
            history_write,
        }
    }

    pub async fn register(
        id_light: AccountIdLight,
        config: &Config,
        current_data_write: CurrentDataWriteHandle,
        history_wirte: HistoryWriteHandle,
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
        }

        Ok(id)
    }

    pub async fn update_api_key(&self, id: AccountIdInternal, key: Option<&ApiKey>) -> Result<(), DatabaseError> {
        self.current()
            .update_api_key(id, key)
            .await
            .with_info_lazy(|| WriteCmd::AccountId(id.as_light()))
    }

    pub(super) fn current(&self) -> CurrentDataWriteCommands {
        CurrentDataWriteCommands::new(&self.current_write)
    }

    pub(super) fn history(&self) -> HistoryWriteCommands {
        HistoryWriteCommands::new(&self.history_write)
    }

    pub async fn update_json<
        T: GetReadWriteCmd + Serialize + Clone + Send + SqliteUpdateJson + HistoryUpdateJson + 'static,
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
            .with_history_write_cmd_info::<T>(id)
    }
}
