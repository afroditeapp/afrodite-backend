use email::WriteCommandsAccountEmail;
use model::{
    Account, AccountData, AccountId, AccountIdInternal, AccountInternal, AccountState, Capabilities, ClientId, DemoModeId, NewsIteratorSessionIdInternal, ProfileVisibility, SetAccountSetup
};
use server_data::{
    cache::CacheError, define_server_data_write_commands, result::Result, DataError, DieselDatabaseError, IntoDataError
};

pub mod email;

define_server_data_write_commands!(WriteCommandsAccount);
define_db_transaction_command!(WriteCommandsAccount);
define_db_read_command_for_write!(WriteCommandsAccount);

#[derive(Debug, Clone, Copy)]
pub struct IncrementAdminAccessGrantedCount;

impl<C: server_data::write::WriteCommandsProvider> WriteCommandsAccount<C> {
    pub fn email(self) -> WriteCommandsAccountEmail<C> {
        WriteCommandsAccountEmail::new(self.cmds)
    }

    /// The only method which can modify AccountState, Capabilities and
    /// ProfileVisibility. This also updates profile index if profile component
    /// is enabled and the visibility changed.
    ///
    /// Returns the modified Account.
    pub async fn update_syncable_account_data(
        &self,
        id: AccountIdInternal,
        increment_admin_access_granted: Option<IncrementAdminAccessGrantedCount>,
        modify_action: impl FnOnce(
                &mut AccountState,
                &mut Capabilities,
                &mut ProfileVisibility,
            ) -> error_stack::Result<(), DieselDatabaseError>
            + Send
            + 'static,
    ) -> Result<Account, DataError> {
        let current_account = self
            .db_read_common(move |mut cmds| cmds.common().account(id))
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

        self.common()
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

    pub async fn set_is_bot_account(
        &self,
        id: AccountIdInternal,
        value: bool,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().sign_in_with().set_is_bot_account(id, value)
        })
    }

    pub async fn get_next_client_id(
        &self,
        id: AccountIdInternal,
    ) -> Result<ClientId, DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account().data().get_next_client_id(id)
        })
    }

    pub async fn handle_reset_news_iterator(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<NewsIteratorSessionIdInternal, DataError> {
        let latest_used_id = self.db_read(|mut cmds| cmds.account().news().latest_used_news_id()).await?;
        let session_id = self.cache()
            .write_cache(id.as_id(), |e| {
                if let Some(c) = e.account.as_mut() {
                    Ok(c.news_iterator.reset(latest_used_id))
                } else {
                    Err(CacheError::FeatureNotEnabled.report())
                }
            })
            .await
            .into_data_error(id)?;

        Ok(session_id)
    }
}
