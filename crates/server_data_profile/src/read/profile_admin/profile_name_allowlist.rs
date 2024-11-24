use model_profile::GetProfileNamePendingModerationList;
use server_data::{
    define_server_data_read_commands, read::ReadCommandsProvider, result::Result, DataError, IntoDataError
};

define_server_data_read_commands!(ReadCommandsProfileNameAllowlist);
define_db_read_command!(ReadCommandsProfileNameAllowlist);

impl<C: ReadCommandsProvider> ReadCommandsProfileNameAllowlist<C> {
    pub async fn profile_name_pending_moderation_list(
        &mut self,
    ) -> Result<GetProfileNamePendingModerationList, DataError> {
        self.db_read(|mut cmds| cmds.profile_admin().profile_name_allowlist().profile_name_pending_moderation_list())
            .await
            .into_error()
    }
}
