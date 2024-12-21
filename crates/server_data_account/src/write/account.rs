use database::current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon};
use database_account::{current::write::GetDbWriteCommandsAccount, history::write::GetDbHistoryWriteCommandsAccount};
use delete::WriteCommandsAccountDelete;
use email::WriteCommandsAccountEmail;
use model::AccountStateContainer;
use model_account::{
    Account, AccountData, AccountId, AccountIdInternal, AccountInternal, ClientId,
    Permissions, ProfileVisibility, SetAccountSetup,
};
use model_server_state::DemoModeId;
use news::WriteCommandsAccountNews;
use server_data::{
    define_cmd_wrapper_write,
    read::DbRead,
    result::Result,
    write::{DbTransaction, GetWriteCommandsCommon, DbTransactionHistory},
    DataError, DieselDatabaseError,
};

pub mod delete;
pub mod email;
pub mod news;

#[derive(Debug, Clone, Copy)]
pub struct IncrementAdminAccessGrantedCount;

define_cmd_wrapper_write!(WriteCommandsAccount);

impl<'a> WriteCommandsAccount<'a> {
    pub fn delete(self) -> WriteCommandsAccountDelete<'a> {
        WriteCommandsAccountDelete::new(self.0)
    }

    pub fn email(self) -> WriteCommandsAccountEmail<'a> {
        WriteCommandsAccountEmail::new(self.0)
    }

    pub fn news(self) -> WriteCommandsAccountNews<'a> {
        WriteCommandsAccountNews::new(self.0)
    }
}

impl WriteCommandsAccount<'_> {
    /// The only method which can modify AccountState, Permissions and
    /// ProfileVisibility. This also updates profile index if profile component
    /// is enabled and the visibility changed.
    ///
    /// Returns the modified Account.
    pub async fn update_syncable_account_data(
        &self,
        id: AccountIdInternal,
        increment_admin_access_granted: Option<IncrementAdminAccessGrantedCount>,
        modify_action: impl FnOnce(
                &mut AccountStateContainer,
                &mut Permissions,
                &mut ProfileVisibility,
            ) -> error_stack::Result<(), DieselDatabaseError>
            + Send
            + 'static,
    ) -> Result<Account, DataError> {
        let current_account = self
            .db_read(move |mut cmds| cmds.common().account(id))
            .await?;
        let a = current_account.clone();
        let new_account = db_transaction!(self, move |mut cmds| {
            let account =
                cmds.common()
                    .state()
                    .update_syncable_account_data(id, a, modify_action)?;

            if increment_admin_access_granted.is_some() {
                cmds.account()
                    .data()
                    .upsert_increment_admin_access_granted_count()?;
            }

            Ok(account)
        })?;

        self.handle()
            .common()
            .internal_handle_new_account_data_after_db_modification(
                id,
                &current_account,
                &new_account,
            )
            .await?;

        Ok(new_account)
    }

    /// Only server WebSocket code should call this method.
    pub async fn reset_syncable_account_data_version(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common().state().reset_account_data_version_number(id)
        })
    }

    pub async fn account_data(
        &self,
        id: AccountIdInternal,
        account_data: AccountData,
    ) -> Result<(), DataError> {
        let internal = AccountInternal {
            email: account_data.email,
        };

        db_transaction!(self, move |mut cmds| {
            cmds.account().data().account(id, &internal)
        })
    }

    pub async fn account_setup(
        &self,
        id: AccountIdInternal,
        account_setup: SetAccountSetup,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().data().account_setup(id, &account_setup)
        })
    }

    pub async fn insert_demo_mode_related_account_ids(
        &self,
        id: DemoModeId,
        account_id: AccountId,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account()
                .demo_mode()
                .insert_related_account_id(id, account_id)
        })
    }

    pub async fn get_next_client_id(&self, id: AccountIdInternal) -> Result<ClientId, DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().data().get_next_client_id(id)
        })
    }

    pub async fn get_next_unique_account_id(&self) -> Result<AccountId, DataError> {
        db_transaction_history!(self, move |mut cmds| {
            cmds.account_history().new_unique_account_id()
        })
    }
}
