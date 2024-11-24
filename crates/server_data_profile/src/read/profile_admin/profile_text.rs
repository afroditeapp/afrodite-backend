use model_profile::{AccountIdInternal, GetProfileTextPendingModerationList, GetProfileTextPendingModerationParams};
use server_data::{
    define_server_data_read_commands, read::ReadCommandsProvider, result::Result, DataError, IntoDataError
};

define_server_data_read_commands!(ReadCommandsProfileText);
define_db_read_command!(ReadCommandsProfileText);

impl<C: ReadCommandsProvider> ReadCommandsProfileText<C> {
    pub async fn profile_text_pending_moderation_list(
        &mut self,
        moderator_id: AccountIdInternal,
        params: GetProfileTextPendingModerationParams,
    ) -> Result<GetProfileTextPendingModerationList, DataError> {
        self.db_read(move |mut cmds| cmds.profile_admin().profile_text().profile_text_pending_moderation_list(moderator_id, params))
            .await
            .into_error()
    }
}
