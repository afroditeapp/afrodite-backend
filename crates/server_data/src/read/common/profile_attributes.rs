use database::current::read::GetDbReadCommandsCommon;

use crate::{DataError, IntoDataError, define_cmd_wrapper_read, read::DbRead, result::Result};

define_cmd_wrapper_read!(ReadCommandsCommonProfileAttributes);

impl ReadCommandsCommonProfileAttributes<'_> {
    pub async fn profile_attributes_hash(&self) -> Result<Option<String>, DataError> {
        self.db_read(|mut cmds| cmds.common().profile_attributes().profile_attributes_hash())
            .await
            .into_error()
    }
}
