use database_media::current::read::GetDbReadCommandsMedia;
use model::ContentIdInternal;
use model_media::{
    AccountId, AccountIdInternal, ContentId, CurrentAccountMediaInternal, MediaContentRaw,
    MediaContentSyncVersion,
};
use server_common::{
    data::{DataError, IntoDataError},
    result::Result,
};
use server_data::{
    define_cmd_wrapper_read,
    file::{FileRead, utils::ContentFile},
    read::DbRead,
};

mod notification;

define_cmd_wrapper_read!(ReadCommandsMedia);

impl<'a> ReadCommandsMedia<'a> {
    pub fn notification(self) -> notification::ReadCommandsMediaNotification<'a> {
        notification::ReadCommandsMediaNotification::new(self.0)
    }
}

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

    pub async fn content_state(
        &self,
        content_id: ContentIdInternal,
    ) -> Result<MediaContentRaw, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media()
                .media_content()
                .get_media_content_raw(content_id)
        })
        .await
        .into_error()
    }

    pub async fn content_id_internal(
        &self,
        account_id: AccountIdInternal,
        content_id: ContentId,
    ) -> Result<ContentIdInternal, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media()
                .media_content()
                .content_id_internal(account_id, content_id)
        })
        .await
        .into_error()
    }

    pub async fn media_content_sync_version(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<MediaContentSyncVersion, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media()
                .get_media_state(account_id)
                .map(|v| v.media_content_sync_version)
        })
        .await
        .into_error()
    }

    pub async fn all_account_media_content_count(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<i64, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media()
                .media_content()
                .get_account_media_content_count(account_id)
        })
        .await
        .into_error()
    }
}
