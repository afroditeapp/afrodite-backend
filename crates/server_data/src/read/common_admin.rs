use database::current::read::GetDbReadCommandsCommon;
use model::AccountIdInternal;
use server_common::data::IntoDataError;

use super::{super::DataError, DbRead};
use crate::{define_cmd_wrapper_read, result::Result};

mod notification;
mod report;
mod statistics;

define_cmd_wrapper_read!(ReadCommandsCommonAdmin);

impl<'a> ReadCommandsCommonAdmin<'a> {
    pub fn notification(self) -> notification::ReadCommandsCommonAdminNotification<'a> {
        notification::ReadCommandsCommonAdminNotification::new(self.0)
    }
    pub fn report(self) -> report::ReadCommandsCommonAdminReport<'a> {
        report::ReadCommandsCommonAdminReport::new(self.0)
    }
    pub fn statistics(self) -> statistics::ReadCommandsCommonAdminStatistics<'a> {
        statistics::ReadCommandsCommonAdminStatistics::new(self.0)
    }
}

impl<'a> ReadCommandsCommonAdmin<'a> {
    pub async fn admin_bot_account_ids(&self) -> Result<Vec<AccountIdInternal>, DataError> {
        self.db_read(move |mut cmds| cmds.common_admin().admin_bot_account_ids())
            .await
            .into_error()
    }
}
