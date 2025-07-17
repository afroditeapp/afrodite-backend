use database_media::current::read::GetDbReadCommandsMedia;
use model::AccountIdInternal;
use model_media::{GetMediaContentPendingModerationList, GetMediaContentPendingModerationParams};
use server_common::result::Result;
use server_data::{DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead};

define_cmd_wrapper_read!(ReadCommandsMediaAdmin);

impl ReadCommandsMediaAdmin<'_> {
    pub async fn media_content_pending_moderation_list_using_moderator_id(
        &self,
        moderator_id: AccountIdInternal,
        params: GetMediaContentPendingModerationParams,
    ) -> Result<GetMediaContentPendingModerationList, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media_admin()
                .content()
                .media_content_pending_moderation_list_using_moderator_id(moderator_id, params)
        })
        .await
        .into_error()
    }

    pub async fn profile_content_pending_moderation_list(
        &self,
        is_bot: bool,
        params: GetMediaContentPendingModerationParams,
    ) -> Result<GetMediaContentPendingModerationList, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media_admin()
                .content()
                .media_content_pending_moderation_list(is_bot, params)
        })
        .await
        .into_error()
    }
}
