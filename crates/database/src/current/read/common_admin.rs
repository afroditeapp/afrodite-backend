use diesel::prelude::*;
use error_stack::Result;
use model::{AccountIdInternal, BotAccountType};
use simple_backend_utils::db::DieselDatabaseError;

use crate::{IntoDatabaseError, define_current_read_commands};

mod notification;
mod report;
mod statistics;

define_current_read_commands!(CurrentReadCommonAdmin);

impl<'a> CurrentReadCommonAdmin<'a> {
    pub fn notification(self) -> notification::CurrentReadAccountAdminNotification<'a> {
        notification::CurrentReadAccountAdminNotification::new(self.cmds)
    }
    pub fn statistics(self) -> statistics::CurrentReadAccountAdminStatistics<'a> {
        statistics::CurrentReadAccountAdminStatistics::new(self.cmds)
    }
    pub fn report(self) -> report::CurrentReadCommonAdminReport<'a> {
        report::CurrentReadCommonAdminReport::new(self.cmds)
    }
}

impl CurrentReadCommonAdmin<'_> {
    /// `Vec<AccountIdInternal>` order is from oldest to latest
    pub fn admin_bot_account_ids(&mut self) -> Result<Vec<AccountIdInternal>, DieselDatabaseError> {
        use crate::schema::{account_id, shared_state};

        account_id::table
            .inner_join(shared_state::table.on(shared_state::account_id.eq(account_id::id)))
            .filter(shared_state::bot_account_type_number.eq(Some(BotAccountType::Admin)))
            .order(account_id::id.asc())
            .select(AccountIdInternal::as_select())
            .load(self.conn())
            .into_db_error(())
    }

    /// `Vec<AccountIdInternal>` order is from oldest to latest
    pub fn user_bot_account_ids(&mut self) -> Result<Vec<AccountIdInternal>, DieselDatabaseError> {
        use crate::schema::{account_id, shared_state};

        account_id::table
            .inner_join(shared_state::table.on(shared_state::account_id.eq(account_id::id)))
            .filter(shared_state::bot_account_type_number.eq(Some(BotAccountType::User)))
            .order(account_id::id.asc())
            .select(AccountIdInternal::as_select())
            .load(self.conn())
            .into_db_error(())
    }
}
