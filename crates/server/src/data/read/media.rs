use crate::utils::ConvertCommandErrorExt;

use model::{AccountIdInternal, AccountIdLight, ContentId, CurrentAccountMediaInternal};

use super::{
    super::{cache::DatabaseCache, file::utils::FileDir, DatabaseError},
    ReadCommands,
};

use error_stack::Result;

define_read_commands!(ReadCommandsMedia);

impl ReadCommandsMedia<'_> {
    pub async fn image(
        &self,
        account_id: AccountIdLight,
        content_id: ContentId,
    ) -> Result<Vec<u8>, DatabaseError> {
        self.files()
            .image_content(account_id, content_id)
            .read_all()
            .await
            .convert((account_id, content_id))
    }

    pub async fn current_account_media(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<CurrentAccountMediaInternal, DatabaseError> {
        self.db()
            .media()
            .get_current_account_media(account_id)
            .await
            .convert(account_id)
    }
}
