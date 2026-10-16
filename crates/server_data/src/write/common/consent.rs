use database::current::write::GetDbWriteCommandsCommon;
use model::{AccountIdInternal, UpdateDataConsentSettings};

use crate::{
    DataError, db_transaction, define_cmd_wrapper_write, result::Result, write::DbTransaction,
};

define_cmd_wrapper_write!(WriteCommandsCommonConsent);

impl WriteCommandsCommonConsent<'_> {
    pub async fn upsert_data_consent_settings(
        &self,
        id: AccountIdInternal,
        value: UpdateDataConsentSettings,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common()
                .consent()
                .upsert_data_consent_settings(id, value)
        })
    }
}
