use database_profile::current::read::GetDbReadCommandsProfile;
use model_profile::GetProfileNamePendingModerationList;
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsProfileNameAllowlist);

impl ReadCommandsProfileNameAllowlist<'_> {
    pub async fn profile_name_pending_moderation_list(
        &self,
    ) -> Result<GetProfileNamePendingModerationList, DataError> {
        self.db_read(|mut cmds| {
            cmds.profile_admin()
                .profile_name_allowlist()
                .profile_name_pending_moderation_list()
        })
        .await
        .into_error()
    }
}
