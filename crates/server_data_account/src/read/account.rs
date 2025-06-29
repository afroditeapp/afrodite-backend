use database::current::read::GetDbReadCommandsCommon;
use database_account::current::read::GetDbReadCommandsAccount;
use model_account::{
    AccountData, AccountGlobalState, AccountId, AccountIdInternal, AccountSetup, AppleAccountId,
    GoogleAccountId, SignInWithInfo,
};
use model_server_state::DemoModeId;
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

pub mod ban;
pub mod delete;
pub mod email;
pub mod news;
pub mod notification;

define_cmd_wrapper_read!(ReadCommandsAccount);

impl<'a> ReadCommandsAccount<'a> {
    pub fn ban(self) -> ban::ReadCommandsAccountBan<'a> {
        ban::ReadCommandsAccountBan::new(self.0)
    }

    pub fn delete(self) -> delete::ReadCommandsAccountDelete<'a> {
        delete::ReadCommandsAccountDelete::new(self.0)
    }

    pub fn email(self) -> email::ReadCommandsAccountEmail<'a> {
        email::ReadCommandsAccountEmail::new(self.0)
    }

    pub fn news(self) -> news::ReadCommandsAccountNews<'a> {
        news::ReadCommandsAccountNews::new(self.0)
    }

    pub fn notification(self) -> notification::ReadCommandsAccountNotification<'a> {
        notification::ReadCommandsAccountNotification::new(self.0)
    }
}

impl ReadCommandsAccount<'_> {
    pub async fn account_sign_in_with_info(
        &self,
        id: AccountIdInternal,
    ) -> Result<SignInWithInfo, DataError> {
        self.db_read(move |mut cmds| {
            cmds.account()
                .sign_in_with()
                .sign_in_with_info_raw(id)
                .map(|v| v.into())
        })
        .await
        .into_error()
    }

    pub async fn is_bot_account(&self, id: AccountIdInternal) -> Result<bool, DataError> {
        self.db_read(move |mut cmds| {
            cmds.common()
                .state()
                .other_shared_state(id)
                .map(|v| v.is_bot_account)
        })
        .await
        .into_error()
    }

    pub async fn account_data(&self, id: AccountIdInternal) -> Result<AccountData, DataError> {
        self.db_read(move |mut cmds| cmds.account().data().account_data(id))
            .await
            .into_error()
    }

    pub async fn account_setup(&self, id: AccountIdInternal) -> Result<AccountSetup, DataError> {
        self.db_read(move |mut cmds| cmds.account().data().account_setup(id))
            .await
            .into_error()
    }

    pub async fn apple_account_id_to_account_id(
        &self,
        id: AppleAccountId,
    ) -> Result<Option<AccountIdInternal>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.account()
                .sign_in_with()
                .apple_account_id_to_account_id(id)
        })
        .await
        .into_error()
    }

    pub async fn google_account_id_to_account_id(
        &self,
        id: GoogleAccountId,
    ) -> Result<Option<AccountIdInternal>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.account()
                .sign_in_with()
                .google_account_id_to_account_id(id)
        })
        .await
        .into_error()
    }

    pub async fn demo_mode_related_account_ids(
        &self,
        id: DemoModeId,
    ) -> Result<Vec<AccountId>, DataError> {
        self.db_read(move |mut cmds| cmds.account().demo_mode().related_account_ids(id))
            .await
            .into_error()
    }

    pub async fn global_state(&self) -> Result<AccountGlobalState, DataError> {
        self.db_read(move |mut cmds| cmds.account().data().global_state())
            .await
            .into_error()
    }
}
