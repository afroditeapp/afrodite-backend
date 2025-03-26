
use std::collections::HashMap;

use database::current::write::GetDbWriteCommandsCommon;
use model::{AccountIdDb, ApiUsage};

use crate::{
    define_cmd_wrapper_write, result::Result, write::db_transaction, DataError
};

use crate::write::DbTransaction;

define_cmd_wrapper_write!(WriteCommandsCommonAdminApiUsage);

impl WriteCommandsCommonAdminApiUsage<'_> {
    pub async fn save_api_usage_data(
        &self,
        data: HashMap<AccountIdDb, ApiUsage>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common_admin().api_usage().save_api_usage_data(data)
        })
    }
}
