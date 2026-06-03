use database_profile::current::read::GetDbReadCommandsProfile;
use model_profile::{ProfileStringModerationQueuePage, ProfileStringModerationQueueType};
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsProfileModeration);

impl ReadCommandsProfileModeration<'_> {
    pub async fn profile_string_moderation_page(
        &self,
        content_type: model_profile::ProfileStringModerationContentType,
        queue_type: ProfileStringModerationQueueType,
    ) -> Result<ProfileStringModerationQueuePage, DataError> {
        self.db_read(move |mut cmds| {
            cmds.profile_admin()
                .moderation()
                .profile_string_moderation_page(content_type, queue_type)
        })
        .await
        .into_error()
    }
}
