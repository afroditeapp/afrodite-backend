use error_stack::Result;
use model::{AccountIdInternal, AccountId, ContentId, CurrentAccountMediaInternal};

use crate::data::IntoDataError;

use super::{
    super::{cache::DatabaseCache, file::utils::FileDir, DatabaseError},
    ReadCommands,
};

define_read_commands!(ReadCommandsMedia);

impl ReadCommandsMedia<'_> {
    pub async fn image(
        &self,
        account_id: AccountId,
        content_id: ContentId,
    ) -> Result<Vec<u8>, DatabaseError> {
        self.files()
            .image_content(account_id, content_id)
            .read_all()
            .await
            .into_data_error((account_id, content_id))
    }

    pub async fn current_account_media(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<CurrentAccountMediaInternal, DatabaseError> {
        self.db_read(move |mut cmds| cmds.media().current_account_media(account_id))
            .await
    }
}
