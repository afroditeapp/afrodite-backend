use database::{define_current_read_commands, ConnectionProvider, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model::{AccountId, DemoModeId};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadAccountDemo, CurrentSyncReadAccountDemo);

impl<C: ConnectionProvider> CurrentSyncReadAccountDemo<C> {
    pub fn related_account_ids(
        &mut self,
        demo_mode_related_id: DemoModeId,
    ) -> Result<Vec<AccountId>, DieselDatabaseError> {
        use crate::schema::demo_mode_account_ids::dsl::*;

        demo_mode_account_ids
            .filter(demo_mode_id.eq(demo_mode_related_id))
            .select(account_id_uuid)
            .load(self.conn())
            .into_db_error(demo_mode_related_id)
    }
}
