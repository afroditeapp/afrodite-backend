use database_profile::current::read::GetDbReadCommandsProfile;
use model_profile::{
    AccountIdDbValue, ProfileIteratorPage, ProfileIteratorSettings, ProfileStringIterationPage,
    ProfileStringModerationContentType,
};
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};
use simple_backend_model::UnixTime;

define_cmd_wrapper_read!(ReadCommandsProfileIterator);

impl ReadCommandsProfileIterator<'_> {
    pub async fn get_latest_created_account_id_db(&self) -> Result<AccountIdDbValue, DataError> {
        self.db_read(move |mut cmds| cmds.profile_admin().iterator().get_latest_account_id_db())
            .await
            .into_error()
    }

    pub async fn get_profile_page(
        &self,
        settings: ProfileIteratorSettings,
    ) -> Result<ProfileIteratorPage, DataError> {
        self.db_read(move |mut cmds| cmds.profile_admin().iterator().get_profile_page(settings))
            .await
            .into_error()
    }

    pub async fn profile_string_iteration_page(
        &self,
        content_type: ProfileStringModerationContentType,
        page: i64,
        origin: UnixTime,
    ) -> Result<ProfileStringIterationPage, DataError> {
        self.db_read(move |mut cmds| {
            cmds.profile_admin()
                .iterator()
                .profile_string_iteration_page(content_type, page, origin)
        })
        .await
        .into_error()
    }
}
