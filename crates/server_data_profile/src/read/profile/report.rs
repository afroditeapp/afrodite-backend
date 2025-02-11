use database_profile::current::read::GetDbReadCommandsProfile;
use model_profile::{AccountIdInternal, ProfileReport};
use server_data::{define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError};

define_cmd_wrapper_read!(ReadCommandsProfileReport);

impl ReadCommandsProfileReport<'_> {
    pub async fn profile_report(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
    ) -> Result<ProfileReport, DataError> {
        self.db_read(move |mut cmds| cmds.profile().report().profile_report(creator, target))
            .await
            .into_error()
    }
}
