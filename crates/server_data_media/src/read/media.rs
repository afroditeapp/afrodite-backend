use database_media::current::read::GetDbReadCommandsMedia;
use model_media::{
    AccountId, AccountIdInternal, ContentId, CurrentAccountMediaInternal, MediaContentRaw,
};
use server_common::{
    data::{DataError, IntoDataError},
    result::Result,
};
use server_data::{
    define_cmd_wrapper_read,
    file::{utils::ContentFile, FileRead},
    read::DbRead,
};

define_cmd_wrapper_read!(ReadCommandsMedia);

impl ReadCommandsMedia<'_> {
    pub async fn content_data(
        &self,
        account_id: AccountId,
        content_id: ContentId,
    ) -> Result<ContentFile, DataError> {
        let c = self.files().media_content(account_id, content_id);
        Ok(c)
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

    pub async fn all_account_media_content(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Vec<MediaContentRaw>, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media()
                .media_content()
                .get_account_media_content(account_id)
        })
        .await
        .into_error()
    }
}
