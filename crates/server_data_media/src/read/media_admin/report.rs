use database_media::current::read::GetDbReadCommandsMedia;
use model_media::GetMediaReportList;
use server_data::{
    define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError,
};

define_cmd_wrapper_read!(ReadCommandsMediaReport);

impl ReadCommandsMediaReport<'_> {
    pub async fn get_report_list(
        &self,
    ) -> Result<GetMediaReportList, DataError> {
        self.db_read(move |mut cmds| {
            cmds.media_admin()
                .report()
                .report_list()
        })
        .await
        .into_error()
    }
}
