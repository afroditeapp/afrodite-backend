use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, ClientConfigSyncVersion};

use crate::{define_current_read_commands, DieselDatabaseError};

define_current_read_commands!(CurrentReadCommonClientConfig);

impl CurrentReadCommonClientConfig<'_> {
    pub fn client_config_sync_version(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<ClientConfigSyncVersion, DieselDatabaseError> {
        use crate::schema::common_state::dsl::*;

        common_state
            .filter(account_id.eq(id.as_db_id()))
            .select(client_config_sync_version)
            .first(self.conn())
            .change_context(DieselDatabaseError::Execute)
    }
}
