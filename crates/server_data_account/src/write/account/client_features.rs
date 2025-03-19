use database::current::write::GetDbWriteCommandsCommon;
use database_account::current::{read::GetDbReadCommandsAccount, write::GetDbWriteCommandsAccount};
use server_data::{
    define_cmd_wrapper_write, result::Result, write::DbTransaction, DataError
};
use tracing::info;

define_cmd_wrapper_write!(WriteCommandsAccountClientFeatures);

impl WriteCommandsAccountClientFeatures<'_> {
    /// Updates the client features sha256 and related sync version for it for every
    /// account if needed.
    pub async fn update_client_features_sha256_and_sync_versions(
        &self,
        sha256: String,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            let current_hash = cmds.read().account().client_features().client_features_hash()?;

            if current_hash.as_deref() != Some(&sha256) {
                info!(
                    "Client features file hash changed from {:?} to {:?}",
                    current_hash,
                    Some(&sha256)
                );

                cmds.account()
                    .client_features()
                    .upsert_client_features_file_hash(&sha256)?;

                cmds.common()
                    .client_config()
                    .increment_client_config_sync_version_for_every_account()?;
            }

            Ok(())
        })
    }
}
