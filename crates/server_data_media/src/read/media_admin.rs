use database_media::current::read::GetDbReadCommandsMedia;
use model::AccountIdInternal;
use model_media::{GetProfileContentPendingModerationList, GetProfileContentPendingModerationParams};
use server_data::{define_cmd_wrapper_read, read::DbRead, DataError, IntoDataError};

use server_common::result::Result;

mod report;

define_cmd_wrapper_read!(ReadCommandsMediaAdmin);

impl<'a> ReadCommandsMediaAdmin<'a> {
    pub fn report(self) -> report::ReadCommandsMediaReport<'a> {
        report::ReadCommandsMediaReport::new(self.0)
    }
}

impl ReadCommandsMediaAdmin<'_> {
    pub async fn profile_content_pending_moderation_list(
        &self,
        moderator_id: AccountIdInternal,
        params: GetProfileContentPendingModerationParams,
    ) -> Result<GetProfileContentPendingModerationList, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media_admin()
                .content()
                .profile_content_pending_moderation_list(moderator_id, params)
        })
        .await
        .into_error()
    }
}
