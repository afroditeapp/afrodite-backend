use model::AccountIdInternal;

use crate::{DataError, define_cmd_wrapper_write, file::FileWrite, result::Result};

define_cmd_wrapper_write!(WriteCommandsCommonDataExport);

impl WriteCommandsCommonDataExport<'_> {
    pub async fn delete_data_export(&self, id: AccountIdInternal) -> Result<(), DataError> {
        self.files()
            .tmp_dir(id.into())
            .data_export()
            .overwrite_and_remove_if_exists()
            .await?;
        Ok(())
    }
}
