use std::collections::HashMap;

use database::current::write::GetDbWriteCommandsCommon;
use model::{AccountIdDb, ApiUsage, IpAddressStorage};

use crate::{
    DataError, define_cmd_wrapper_write,
    result::Result,
    write::{DbTransaction, db_transaction},
};

define_cmd_wrapper_write!(WriteCommandsCommonAdminStatistics);

impl WriteCommandsCommonAdminStatistics<'_> {
    pub async fn save_api_usage_data(
        &self,
        data: HashMap<AccountIdDb, ApiUsage>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common_admin().statistics().save_api_usage_data(data)
        })
    }

    pub async fn save_ip_address_data(
        &self,
        data: HashMap<AccountIdDb, IpAddressStorage>,
    ) -> Result<(), DataError> {
        db_transaction!(self, move |mut cmds| {
            cmds.common_admin().statistics().save_ip_address_data(data)
        })
    }
}
