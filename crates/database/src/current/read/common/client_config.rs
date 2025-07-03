use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{AccountIdInternal, ClientConfigSyncVersion, ClientLanguage, ClientType};

use crate::{DieselDatabaseError, define_current_read_commands};

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

    pub fn client_login_session_platform(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<Option<ClientType>, DieselDatabaseError> {
        use crate::schema::common_state::dsl::*;

        common_state
            .filter(account_id.eq(id.as_db_id()))
            .select(client_login_session_platform)
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)
            .map(|v| v.flatten())
    }

    pub fn client_language(
        &mut self,
        id: AccountIdInternal,
    ) -> Result<ClientLanguage, DieselDatabaseError> {
        use crate::schema::common_state::dsl::*;

        common_state
            .filter(account_id.eq(id.as_db_id()))
            .select(client_language)
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)
            .map(|v| v.unwrap_or_default())
    }
}
