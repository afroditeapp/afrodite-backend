use database_profile::current::read::GetDbReadCommandsProfile;
use model_profile::{
    AccountIdInternal, GetProfileStringPendingModerationList,
    GetProfileStringPendingModerationParams,
};
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsProfileModeration);

impl ReadCommandsProfileModeration<'_> {
    pub async fn profile_pending_moderation_list_using_moderator_id(
        &self,
        moderator_id: AccountIdInternal,
        params: GetProfileStringPendingModerationParams,
    ) -> Result<GetProfileStringPendingModerationList, DataError> {
        self.db_read(move |mut cmds| {
            cmds.profile_admin()
                .moderation()
                .profile_string_pending_moderation_list_using_moderator_id(moderator_id, params)
        })
        .await
        .into_error()
    }

    pub async fn profile_pending_moderation_list(
        &self,
        is_bot: bool,
        params: GetProfileStringPendingModerationParams,
    ) -> Result<GetProfileStringPendingModerationList, DataError> {
        self.db_read(move |mut cmds| {
            cmds.profile_admin()
                .moderation()
                .profile_string_pending_moderation_list(is_bot, params)
        })
        .await
        .into_error()
    }
}
