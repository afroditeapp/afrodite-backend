use database_profile::current::read::GetDbReadCommandsProfile;
use model_profile::GetProfileReportList;
use server_data::{
    define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError,
};

define_cmd_wrapper_read!(ReadCommandsProfileReport);

impl ReadCommandsProfileReport<'_> {
    pub async fn get_report_list(
        &self,
    ) -> Result<GetProfileReportList, DataError> {
        self.db_read(move |mut cmds| {
            cmds.profile_admin()
                .report()
                .get_report_list()
        })
        .await
        .into_error()
    }
}
