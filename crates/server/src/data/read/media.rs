use error_stack::{Result, ResultExt};
use model::{AccountId, AccountIdInternal, ContentId, CurrentAccountMediaInternal};

use super::{
    super::{cache::DatabaseCache, file::utils::FileDir, DataError},
    ReadCommands,
};
use crate::data::IntoDataError;

define_read_commands!(ReadCommandsMedia);

impl ReadCommandsMedia<'_> {
    pub async fn image(
        &self,
        account_id: AccountId,
        content_id: ContentId,
    ) -> Result<Vec<u8>, DataError> {
        self.files()
            .image_content(account_id, content_id)
            .read_all()
            .await
            .into_data_error((account_id, content_id))
    }

    pub async fn current_account_media(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<CurrentAccountMediaInternal, DataError> {
        self.db_read(move |mut cmds| cmds.media().current_account_media(account_id))
            .await
    }
}
