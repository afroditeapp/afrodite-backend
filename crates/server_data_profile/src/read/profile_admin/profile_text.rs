use model_profile::{AccountIdInternal, GetProfileTextPendingModerationList, GetProfileTextPendingModerationParams};
use server_data::{
    define_cmd_wrapper, result::Result, DataError, IntoDataError
};

use crate::read::DbReadProfile;

define_cmd_wrapper!(ReadCommandsProfileText);

impl<C: DbReadProfile> ReadCommandsProfileText<C> {
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
