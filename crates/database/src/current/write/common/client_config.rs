use diesel::{prelude::*, update};
use error_stack::Result;
use model::{AccountIdInternal, ClientType, SyncVersion};

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_read_commands};

define_current_read_commands!(CurrentWriteCommonClientConfig);

impl CurrentWriteCommonClientConfig<'_> {
    pub fn increment_client_config_sync_version_for_every_account(
        &mut self,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        update(common_state)
            .filter(client_config_sync_version.lt(SyncVersion::MAX_VALUE))
            .set(client_config_sync_version.eq(client_config_sync_version + 1))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn reset_client_config_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        update(common_state)
            .filter(account_id.eq(id.as_db_id()))
            .set(client_config_sync_version.eq(0))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    pub fn update_client_login_session_platform(
        &mut self,
        id: AccountIdInternal,
        value: ClientType,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        update(common_state)
            .filter(account_id.eq(id.as_db_id()))
            .set(client_login_session_platform.eq(value))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
