use database::current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon};
use database_account::current::{read::GetDbReadCommandsAccount, write::GetDbWriteCommandsAccount};
use model::{Account, UnixTime};
use model_account::{AccountBanReasonCategory, AccountBanReasonDetails, AccountIdInternal};
use server_data::{
    DataError, db_transaction, define_cmd_wrapper_write,
    read::DbRead,
    result::Result,
    write::{DbTransaction, GetWriteCommandsCommon},
};

define_cmd_wrapper_write!(WriteCommandsAccountBan);

impl WriteCommandsAccountBan<'_> {
    pub async fn set_account_ban_state(
        &self,
        id: AccountIdInternal,
        admin_id: Option<AccountIdInternal>,
        banned_until: Option<UnixTime>,
        reason_category: Option<AccountBanReasonCategory>,
        reason_details: AccountBanReasonDetails,
    ) -> Result<Option<Account>, DataError> {
        let (ban_state, current_account) = self
            .db_read(move |mut cmds| {
                let ban_state = cmds.account().ban().account_ban_time(id)?;
                let current_account = cmds.common().account(id)?;
                Ok((ban_state, current_account))
            })
            .await?;
        if banned_until == ban_state.banned_until {
            // Already in correct state
            return Ok(None);
        }
        let a = current_account.clone();
        let new_account = db_transaction!(self, move |mut cmds| {
            let a = cmds.common().state().update_syncable_account_data(
                id,
                a,
                move |state_container, _, visibility| {
                    state_container.set_banned(banned_until.is_some());
                    if banned_until.is_some() {
                        visibility.change_to_private_or_pending_private();
                    }
                    Ok(())
                },
            )?;

            cmds.account_admin().ban().set_banned_state(
                id,
                admin_id,
                banned_until,
                reason_category,
                reason_details,
            )?;

            Ok(a)
        })?;

        self.handle()
            .common()
            .internal_handle_new_account_data_after_db_modification(
                id,
                &current_account,
                &new_account,
            )
            .await?;

        Ok(Some(new_account))
    }
}
