use model::AccountIdInternal;
use tokio_util::io::ReaderStream;

use crate::{DataError, IntoDataError, define_cmd_wrapper_read, file::FileRead, result::Result};

define_cmd_wrapper_read!(ReadCommandsCommonDataExport);

impl ReadCommandsCommonDataExport<'_> {
    pub async fn data_export_archive_stream(
        &self,
        id: AccountIdInternal,
    ) -> Result<(u64, ReaderStream<tokio::fs::File>), DataError> {
        self.files()
            .tmp_dir(id.into())
            .data_export()
            .byte_count_and_read_stream()
            .await
            .into_error()
    }
}
