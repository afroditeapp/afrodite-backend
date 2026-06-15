use database_account::current::write::GetDbWriteCommandsAccount;
use model::{AccountIdInternal, CustomEmailSendingLimits, UnixTime};
use model_account::{CustomEmailId, UpdateCustomEmail};
use server_data::{
    DataError,
    app::{GetConfig, GetEmailSender},
    db_transaction, define_cmd_wrapper_write,
    result::{Result, WrappedContextExt},
    write::DbTransaction,
};

use crate::read::GetReadCommandsAccount;

pub struct LimitReached;

fn validate_custom_email_translations(
    translations: &[model_account::CustomEmailTranslation],
) -> Result<(), DataError> {
    let all_non_empty = translations
        .iter()
        .all(|t| !t.subject.trim().is_empty() && !t.body.trim().is_empty());
    let has_default = translations.iter().any(|t| t.locale == "default");

    if !all_non_empty || !has_default {
        return Err(DataError::NotAllowed.report());
    }

    Ok(())
}

define_cmd_wrapper_write!(WriteCommandsAccountCustomEmailAdmin);

impl WriteCommandsAccountCustomEmailAdmin<'_> {
    pub async fn create_custom_email(
        &self,
        id: AccountIdInternal,
    ) -> Result<CustomEmailId, DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin().custom_email().create_custom_email(id)
        })
    }

    pub async fn update_custom_email(&self, data: UpdateCustomEmail) -> Result<(), DataError> {
        validate_custom_email_translations(&data.translations)?;

        db_transaction!(self, move |mut cmds| {
            cmds.account_admin()
                .custom_email()
                .update_custom_email(data)
        })
    }

    async fn validate_custom_email(&self, email_id: CustomEmailId) -> Result<(), DataError> {
        let translations = self
            .handle()
            .read()
            .account_admin()
            .custom_email()
            .custom_email_translations(email_id)
            .await?;

        validate_custom_email_translations(&translations)
    }

    async fn handle_custom_email_limit(
        &self,
        db_limit_updater: impl FnOnce(&mut CustomEmailSendingLimits) -> bool,
    ) -> Result<Option<LimitReached>, DataError> {
        let mut limits = self
            .handle()
            .read()
            .account_admin()
            .custom_email()
            .custom_email_sending_limits()
            .await?
            .unwrap_or_default();

        let now = UnixTime::current_time();
        let month_in_seconds_approximately = 30 * 24 * 60 * 60;
        let next_reset = limits.reset_unix_time.ut + month_in_seconds_approximately;
        if next_reset <= now.ut {
            limits.send_to_all_accounts_monthly_count = 0;
            limits.send_draft_to_my_email_monthly_count = 0;
            limits.reset_unix_time = now;
        }

        if db_limit_updater(&mut limits) {
            return Ok(Some(LimitReached));
        }

        self.upsert_custom_email_sending_limits(&limits).await?;

        Ok(None)
    }

    pub async fn send_custom_email(
        &self,
        email_id: CustomEmailId,
        account_ids: Vec<AccountIdInternal>,
    ) -> Result<Option<LimitReached>, DataError> {
        self.validate_custom_email(email_id).await?;

        let limit = self
            .config()
            .limits_account()
            .custom_email_send_to_all_accounts_monthly_max_count;

        let limit_reached = self
            .handle_custom_email_limit(|db_limits| {
                if db_limits.send_to_all_accounts_monthly_count >= limit as i32 {
                    return true;
                }
                db_limits.send_to_all_accounts_monthly_count += 1;
                false
            })
            .await?;

        if limit_reached.is_some() {
            return Ok(limit_reached);
        }

        db_transaction!(self, move |mut cmds| {
            cmds.account_admin()
                .custom_email()
                .init_custom_email_sending(email_id, &account_ids)?;
            Ok(())
        })?;

        self.email_sender()
            .trigger_custom_email_sending(email_id.eid, None);

        Ok(None)
    }

    pub async fn send_custom_email_draft_to_target(
        &self,
        email_id: CustomEmailId,
        target_account_id: AccountIdInternal,
    ) -> Result<Option<LimitReached>, DataError> {
        self.validate_custom_email(email_id).await?;

        let limit = self
            .config()
            .limits_account()
            .custom_email_send_draft_to_my_email_address_monthly_max_count;

        let limit_reached = self
            .handle_custom_email_limit(|db_limits| {
                if db_limits.send_draft_to_my_email_monthly_count >= limit as i32 {
                    return true;
                }
                db_limits.send_draft_to_my_email_monthly_count += 1;
                false
            })
            .await?;

        if limit_reached.is_some() {
            return Ok(limit_reached);
        }

        self.email_sender()
            .trigger_custom_email_sending(email_id.eid, Some(target_account_id));

        Ok(None)
    }

    pub async fn mark_custom_email_sent(
        &self,
        email_id: CustomEmailId,
        account_id: AccountIdInternal,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin()
                .custom_email()
                .mark_custom_email_sent(email_id, &account_id)?;
            Ok(())
        })
    }

    pub async fn mark_custom_email_sending_completed(
        &self,
        email_id: CustomEmailId,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin()
                .custom_email()
                .set_custom_email_sending_completed(email_id)?;
            Ok(())
        })
    }

    pub async fn upsert_custom_email_sending_limits(
        &self,
        limits: &CustomEmailSendingLimits,
    ) -> Result<(), DataError> {
        let limits = limits.clone();
        db_transaction!(self, move |mut cmds| {
            cmds.account_admin()
                .custom_email()
                .upsert_custom_email_sending_limits(&limits)
        })
    }
}
