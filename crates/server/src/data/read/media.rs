use model::{AccountId, AccountIdInternal, ContentId, CurrentAccountMediaInternal};

use super::super::DataError;
use crate::{data::IntoDataError, result::Result};

define_read_commands!(ReadCommandsMedia);

impl ReadCommandsMedia<'_> {
    pub async fn content_data(
        &self,
        account_id: AccountId,
        content_id: ContentId,
    ) -> Result<Vec<u8>, DataError> {
        self.files()
            .media_content(account_id, content_id)
            .read_all()
            .await
            .into_data_error((account_id, content_id))
    }

    pub async fn current_account_media(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<CurrentAccountMediaInternal, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media()
                .media_content()
                .current_account_media(account_id)
        })
        .await
        .into_error()
    }
}
