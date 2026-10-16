use database::current::read::GetDbReadCommandsCommon;
use model::{AccountIdInternal, DataConsentSettingsInternal};
use server_common::data::IntoDataError;

use crate::{DataError, define_cmd_wrapper_read, read::DbRead, result::Result};

define_cmd_wrapper_read!(ReadCommandsCommonConsent);

impl ReadCommandsCommonConsent<'_> {
    pub async fn data_consent_settings(
        &self,
        id: AccountIdInternal,
    ) -> Result<DataConsentSettingsInternal, DataError> {
        self.db_read(move |mut cmds| cmds.common().consent().data_consent_settings(id))
            .await
            .into_error()
    }
}
