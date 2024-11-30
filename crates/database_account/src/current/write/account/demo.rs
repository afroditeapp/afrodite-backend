use database::{define_current_write_commands, DieselDatabaseError};
use diesel::{insert_into, prelude::*};
use error_stack::Result;
use model::AccountId;
use model_server_state::DemoModeId;

use crate::IntoDatabaseError;

define_current_write_commands!(CurrentWriteAccountDemo);

impl CurrentWriteAccountDemo<'_> {
    pub fn insert_related_account_id(
        &mut self,
        demo_mode_related_id: DemoModeId,
        account_uuid: AccountId,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::demo_mode_account_ids::dsl::*;

        insert_into(demo_mode_account_ids)
            .values((
                demo_mode_id.eq(demo_mode_related_id),
                account_id_uuid.eq(account_uuid),
            ))
            .execute(self.conn())
            .into_db_error(account_uuid)?;

        Ok(())
    }
}
