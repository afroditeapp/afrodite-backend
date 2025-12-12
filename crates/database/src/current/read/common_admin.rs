use diesel::prelude::*;
use error_stack::Result;
use model::AccountIdInternal;
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
    pub fn admin_bot_account_ids(&mut self) -> Result<Vec<AccountIdInternal>, DieselDatabaseError> {
        use crate::schema::{account_id, account_permissions, shared_state};

        account_id::table
            .inner_join(shared_state::table.on(shared_state::account_id.eq(account_id::id)))
            .inner_join(
                account_permissions::table.on(account_permissions::account_id.eq(account_id::id)),
            )
            .filter(shared_state::is_bot_account.eq(true))
            .filter(
                account_permissions::admin_moderate_media_content
                    .eq(true)
                    .or(account_permissions::admin_moderate_profile_names.eq(true))
                    .or(account_permissions::admin_moderate_profile_texts.eq(true)),
            )
            .select(AccountIdInternal::as_select())
            .load(self.conn())
            .into_db_error(())
    }
}
