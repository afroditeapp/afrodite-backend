use model_profile::{AccountIdInternal, GetProfileTextPendingModerationList, GetProfileTextPendingModerationParams};
use server_data::{
    define_cmd_wrapper_read, result::Result, DataError, IntoDataError
};

use crate::read::DbReadProfile;

define_cmd_wrapper_read!(ReadCommandsProfileText);

impl ReadCommandsProfileText<'_> {
    pub async fn profile_text_pending_moderation_list(
        &self,
        moderator_id: AccountIdInternal,
        params: GetProfileTextPendingModerationParams,
    ) -> Result<GetProfileTextPendingModerationList, DataError> {
        self.db_read(move |mut cmds| cmds.profile_admin().profile_text().profile_text_pending_moderation_list(moderator_id, params))
            .await
            .into_error()
    }
}
