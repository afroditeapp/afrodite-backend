use database_account::current::read::GetDbReadCommandsAccount;
use model_account::{AccountIdInternal, ProfileNameAndAge};
use server_data::{
    DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result,
};

define_cmd_wrapper_read!(ReadCommandsProfileUtils);

impl ReadCommandsProfileUtils<'_> {
    pub async fn profile_name_and_age(
        &self,
        id: AccountIdInternal,
    ) -> Result<ProfileNameAndAge, DataError> {
        self.db_read(move |mut cmds| cmds.account_profile_utils().profile_name_and_age(id))
            .await
            .into_error()
    }
}
