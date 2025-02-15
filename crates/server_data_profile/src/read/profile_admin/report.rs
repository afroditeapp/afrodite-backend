use database_profile::current::read::GetDbReadCommandsProfile;
use model_profile::{GetProfileNameReportList, GetProfileTextReportList};
use server_data::{
    define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError,
};

define_cmd_wrapper_read!(ReadCommandsProfileReport);

impl ReadCommandsProfileReport<'_> {
    pub async fn get_profile_name_report_list(
        &self,
    ) -> Result<GetProfileNameReportList, DataError> {
        self.db_read(move |mut cmds| {
            cmds.profile_admin()
                .report()
                .get_profile_name_report_list()
        })
        .await
        .into_error()
    }

    pub async fn get_profile_text_report_list(
        &self,
    ) -> Result<GetProfileTextReportList, DataError> {
        self.db_read(move |mut cmds| {
            cmds.profile_admin()
                .report()
                .get_profile_text_report_list()
        })
        .await
        .into_error()
    }
}
