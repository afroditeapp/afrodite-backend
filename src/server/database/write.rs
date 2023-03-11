use error_stack::Result;

use crate::{
    api::model::{
        Account,
        ApiKey, AccountId, AccountState, Profile, AccountSetup,
    },
    server::database::{
        git::util::GitUserDirPath, sqlite::SqliteWriteHandle, DatabaseError,
        GitDatabaseOperationHandle,
    },
    utils::ErrorConversion, config::Config,
};

use super::{git::write::GitDatabaseWriteCommands, sqlite::write::SqliteWriteCommands};

#[derive(Debug, Clone)]
pub enum WriteCmd {
    Register(AccountId),
    RegisterAccount(AccountId),
    RegisterAccountSetup(AccountId),
    RegisterProfile(AccountId),
    UpdateProfile(AccountId),
    UpdateApiKey(AccountId),
    UpdateAccountState(AccountId),
    UpdateAccountSetup(AccountId),
}

impl std::fmt::Display for WriteCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Write command: {:?}", self))
    }
}

/// Write methods should be mutable to make sure that there is no concurrent
/// Git user directory writing.
pub struct WriteCommands {
    user_dir: GitUserDirPath,
    database_handle: GitDatabaseOperationHandle,
    sqlite_database_write: SqliteWriteHandle,
}

impl WriteCommands {
    pub fn new(
        user_dir: GitUserDirPath,
        database_handle: GitDatabaseOperationHandle,
        sqlite_database_write: SqliteWriteHandle,
    ) -> Self {
        Self {
            user_dir,
            database_handle,
            sqlite_database_write,
        }
    }

    pub async fn register(&mut self, config: &Config) -> Result<(), DatabaseError> {
        let account_state = Account::default();
        let account_setup = AccountSetup::default();
        let profile = Profile::default();

        self.git()
            .store_account_id()
            .await
            .with_info_lazy(|| WriteCmd::Register(self.user_dir.id().clone()))?;

        if config.components().account {
            self.git()
                .update_account(&account_state)
                .await
                .with_info_lazy(|| WriteCmd::RegisterAccount(self.user_dir.id().clone()))?;

            self.git()
                .update_account_setup(&account_setup)
                .await
                .with_info_lazy(|| WriteCmd::RegisterAccountSetup(self.user_dir.id().clone()))?;
        }

        if config.components().profile {
            self.git()
                .update_user_profile(&profile)
                .await
                .with_info_lazy(|| WriteCmd::RegisterProfile(self.user_dir.id().clone()))?;
        }

        self.sqlite()
            .store_account_id(self.user_dir.id())
            .await
            .with_info_lazy(|| WriteCmd::Register(self.user_dir.id().clone()))?;

        if config.components().account {
            self.sqlite()
                .store_account(self.user_dir.id(), &account_state)
                .await
                .with_info_lazy(|| WriteCmd::RegisterAccount(self.user_dir.id().clone()))?;

            self.sqlite()
                .store_account_setup(self.user_dir.id(), &account_setup)
                .await
                .with_info_lazy(|| WriteCmd::RegisterAccountSetup(self.user_dir.id().clone()))?;
        }

        if config.components().profile {
            self.sqlite()
                .store_profile(self.user_dir.id(), &profile)
                .await
                .with_info_lazy(|| WriteCmd::RegisterProfile(self.user_dir.id().clone()))?;
        }

        Ok(())
    }

    pub async fn update_user_profile(
        &mut self,
        profile_data: &Profile,
    ) -> Result<(), DatabaseError> {
        self.git()
            .update_user_profile(profile_data)
            .await
            .with_info_lazy(|| WriteCmd::UpdateProfile(self.user_dir.id().clone()))?;
        self.sqlite()
            .update_profile(self.user_dir.id(), profile_data)
            .await
            .with_info_lazy(|| WriteCmd::UpdateProfile(self.user_dir.id().clone()))
    }

    pub async fn update_account_setup(
        &mut self,
        data: &AccountSetup,
    ) -> Result<(), DatabaseError> {
        self.git()
            .update_account_setup(data)
            .await
            .with_info_lazy(|| WriteCmd::UpdateAccountSetup(self.user_dir.id().clone()))?;
        self.sqlite()
            .update_account_setup(self.user_dir.id(), data)
            .await
            .with_info_lazy(|| WriteCmd::UpdateAccountSetup(self.user_dir.id().clone()))
    }

    pub async fn update_current_api_key(&mut self, key: &ApiKey) -> Result<(), DatabaseError> {
        // Token is only stored as a file.
        self.git()
            .update_token(key)
            .await
            .with_info_lazy(|| WriteCmd::UpdateApiKey(self.user_dir.id().clone()))
    }

    fn git(&self) -> GitDatabaseWriteCommands {
        GitDatabaseWriteCommands::new(self.user_dir.clone(), self.database_handle.clone(), None)
    }

    /// Use constant title for commit messages when using write commands
    /// through returned object.
    pub(super) fn git_with_mode_message(&self, message: Option<&str>) -> GitDatabaseWriteCommands {
        GitDatabaseWriteCommands::new(self.user_dir.clone(), self.database_handle.clone(), message)
    }

    pub(super) fn sqlite(&self) -> SqliteWriteCommands {
        SqliteWriteCommands::new(&self.sqlite_database_write)
    }
}
