use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, PushNotificationDbState};

use crate::{DieselDatabaseError, define_current_read_commands};

define_current_read_commands!(CurrentReadCommonPushNotification);

impl CurrentReadCommonPushNotification<'_> {
    pub fn push_notification_db_state(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<PushNotificationDbState, DieselDatabaseError> {
        use crate::schema::common_state::dsl::*;

        common_state
            .filter(account_id.eq(id.as_db_id()))
            .select(PushNotificationDbState::as_select())
            .first(self.conn())
            .change_context(DieselDatabaseError::Execute)
    }
}
