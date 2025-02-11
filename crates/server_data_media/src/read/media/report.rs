use database_media::current::read::GetDbReadCommandsMedia;
use model_media::MediaReport;
use model::AccountIdInternal;
use server_data::{define_cmd_wrapper_read, read::DbRead, result::Result, DataError, IntoDataError};

define_cmd_wrapper_read!(ReadCommandsMediaReport);

impl ReadCommandsMediaReport<'_> {
    pub async fn get_report(
        &self,
        creator: AccountIdInternal,
        target: AccountIdInternal,
    ) -> Result<MediaReport, DataError> {
        self.db_read(move |mut cmds| cmds.media().report().get_report(creator, target))
            .await
            .into_error()
    }
}
