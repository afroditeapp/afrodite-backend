use database::current::{read::GetDbReadCommandsCommon, write::GetDbWriteCommandsCommon};
use database_account::current::{read::GetDbReadCommandsAccount, write::GetDbWriteCommandsAccount};
use model::{EventToClientInternal, UnixTime};
use model_account::{
    AccountBanReasonCategory, AccountBanReasonDetails, AccountBannedAdminType, AccountIdInternal,
};
use server_data::{
    DataError,
    app::EventManagerProvider,
    db_transaction, define_cmd_wrapper_write,
    read::DbRead,
    result::Result,
    write::{DbTransaction, GetWriteCommandsCommon},
};

define_cmd_wrapper_write!(WriteCommandsAccountBan);

pub enum SetAccountBanStateMode {
    Clear,
    AutoBan {
        banned_until: UnixTime,
        reason_category: Option<AccountBanReasonCategory>,
        reason_details: Option<AccountBanReasonDetails>,
    },
    BanOrUnban {
        admin_id: AccountIdInternal,
        banned_until: Option<UnixTime>,
        reason_category: Option<AccountBanReasonCategory>,
        reason_details: Option<AccountBanReasonDetails>,
    },
}

impl WriteCommandsAccountBan<'_> {
    pub async fn set_account_ban_state(
        &self,
        id: AccountIdInternal,
        mode: SetAccountBanStateMode,
    ) -> Result<(), DataError> {
        let (banned_until, admin_id, admin_type, reason_category, reason_details) = match mode {
            SetAccountBanStateMode::Clear => (None, None, None, None, None),
            SetAccountBanStateMode::AutoBan {
                banned_until,
                reason_category,
                reason_details,
            } => (
                Some(banned_until),
                None,
                Some(AccountBannedAdminType::Server),
                reason_category,
                reason_details,
            ),
            SetAccountBanStateMode::BanOrUnban {
                admin_id,
                banned_until,
                reason_category,
                reason_details,
            } => {
                let admin_type = self
                    .db_read(move |mut cmds| {
                        cmds.common()
                            .state()
                            .other_shared_state(admin_id)
                            .map(|state| {
                                if state.is_bot() {
                                    AccountBannedAdminType::Bot
                                } else {
                                    AccountBannedAdminType::Human
                                }
                            })
                    })
                    .await?;
                (
                    banned_until,
                    Some(admin_id),
                    Some(admin_type),
                    reason_category,
                    reason_details,
                )
            }
        };

        let (ban_state, current_account) = self
            .db_read(move |mut cmds| {
                let ban_state = cmds.account().ban().account_ban_time(id)?;
                let current_account = cmds.common().account(id)?;
                Ok((ban_state, current_account))
            })
            .await?;

        if banned_until == ban_state.banned_until {
            // Already in correct state
            return Ok(());
        }

        let a = current_account.clone();
        let new_account = db_transaction!(self, move |mut cmds| {
            let a = cmds
                .common()
                .state()
                .update_syncable_account_data(id, a, move |account| {
                    account.state.set_banned(banned_until.is_some());
                    if banned_until.is_some() {
                        account
                            .profile_visibility
                            .change_to_private_or_pending_private();
                    }
                    Ok(())
                })?;

            cmds.account_admin().ban().set_banned_state(
                id,
                admin_id,
                admin_type,
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
                new_account,
            )
            .await?;

        self.event_manager()
            .send_connected_event(id.uuid, EventToClientInternal::AccountStateChanged)
            .await?;

        Ok(())
    }
}
