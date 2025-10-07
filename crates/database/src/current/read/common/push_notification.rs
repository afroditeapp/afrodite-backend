use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, PushNotificationDbState, PushNotificationInfoSyncVersion};

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

    pub fn push_notification_info_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<PushNotificationInfoSyncVersion, DieselDatabaseError> {
        use crate::schema::common_state::dsl::*;

        common_state
            .filter(account_id.eq(id.as_db_id()))
            .select(push_notification_info_sync_version)
            .first(self.conn())
            .change_context(DieselDatabaseError::Execute)
    }

    pub fn vapid_public_key_hash(&mut self) -> Result<Option<String>, DieselDatabaseError> {
        use crate::schema::vapid_public_key_hash::dsl::*;

        vapid_public_key_hash
            .filter(row_type.eq(0))
            .select(sha256_hash)
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)
    }
}
