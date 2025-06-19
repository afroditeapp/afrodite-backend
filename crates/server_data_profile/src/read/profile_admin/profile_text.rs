use database_profile::current::read::GetDbReadCommandsProfile;
use model_profile::{
    AccountIdInternal, GetProfileTextPendingModerationList, GetProfileTextPendingModerationParams,
};
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsProfileText);

impl ReadCommandsProfileText<'_> {
    pub async fn profile_text_pending_moderation_list_using_moderator_id(
        &self,
        moderator_id: AccountIdInternal,
        params: GetProfileTextPendingModerationParams,
    ) -> Result<GetProfileTextPendingModerationList, DataError> {
        self.db_read(move |mut cmds| {
            cmds.profile_admin()
                .profile_text()
                .profile_text_pending_moderation_list_using_moderator_id(moderator_id, params)
        })
        .await
        .into_error()
    }

    pub async fn profile_text_pending_moderation_list(
        &self,
        is_bot: bool,
        params: GetProfileTextPendingModerationParams,
    ) -> Result<GetProfileTextPendingModerationList, DataError> {
        self.db_read(move |mut cmds| {
            cmds.profile_admin()
                .profile_text()
                .profile_text_pending_moderation_list(is_bot, params)
        })
        .await
        .into_error()
    }
}
