use database_media::current::read::GetDbReadCommandsMedia;
use model_media::{
    AccountId, AccountIdInternal, ContentId, CurrentAccountMediaInternal, MediaContentRaw, ProfileContentSyncVersion,
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

    pub async fn profile_content_moderated_as_accepted(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<bool, DataError> {
        let content = self.db_read(move |mut cmds| {
            cmds.media()
                .media_content()
                .current_account_media(account_id)
        })
        .await
        .into_error()?;

        let mut accepted = content.iter_current_profile_content().count() > 0;
        for c in content.iter_current_profile_content() {
            if !c.state().is_accepted() {
                accepted = false;
            }
        }

        Ok(accepted)
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

    pub async fn profile_content_sync_version(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<ProfileContentSyncVersion, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media()
                .get_media_state(account_id)
                .map(|v| v.profile_content_sync_version)
        })
        .await
        .into_error()
    }
}
