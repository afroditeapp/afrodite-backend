use database::current::write::GetDbWriteCommandsCommon;
use database_account::current::{read::GetDbReadCommandsAccount, write::GetDbWriteCommandsAccount};
use model::AccountIdInternal;
use model_account::AccountReportContent;
use server_data::{
    define_cmd_wrapper_write,
    result::Result,
    write::DbTransaction,
    DataError,
};
use tracing::info;

define_cmd_wrapper_write!(WriteCommandsAccountReport);

impl WriteCommandsAccountReport<'_> {
    pub async fn update_report(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
        reported_content: AccountReportContent,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.account()
                .report()
                .upsert_report(creator, target, reported_content)?;
            Ok(())
        })?;

        Ok(())
    }

    /// Updates the custom reports sha256 and related sync version for it for every
    /// account if needed.
    pub async fn update_custom_reports_sha256_and_sync_versions(
        &self,
        sha256: String,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            let current_hash = cmds.read().account().report().custom_reports_hash()?;

            if current_hash.as_deref() != Some(&sha256) {
                info!(
                    "Custom reports file hash changed from {:?} to {:?}",
                    current_hash,
                    Some(&sha256)
                );

                cmds.account()
                    .report()
                    .upsert_custom_reports_file_hash(&sha256)?;

                cmds.common()
                    .client_config()
                    .increment_client_config_sync_version_for_every_account()?;
            }

            Ok(())
        })
    }
}
