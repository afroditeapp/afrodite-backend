use diesel::prelude::*;
use error_stack::{Result, ResultExt};
use model::{
    AccountIdInternal, ClientConfigSyncVersion, ClientLanguage, ClientType,
    DynamicClientFeaturesConfig, DynamicClientFeaturesConfigHash, DynamicServerConfig,
};

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
    ) -> Result<Option<ClientLanguage>, DieselDatabaseError> {
        use crate::schema::common_state::dsl::*;

        common_state
            .filter(account_id.eq(id.as_db_id()))
            .select(client_language)
            .first(self.conn())
            .change_context(DieselDatabaseError::Execute)
    }

    pub fn dynamic_client_features(
        &mut self,
    ) -> Result<
        Option<(DynamicClientFeaturesConfigHash, DynamicClientFeaturesConfig)>,
        DieselDatabaseError,
    > {
        use crate::schema::dynamic_client_features_config::dsl::*;

        let value: Option<String> = dynamic_client_features_config
            .filter(row_type.eq(0))
            .select(config_json)
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)?;

        value
            .map(|json| {
                let config =
                    serde_json::from_str(&json).change_context(DieselDatabaseError::Execute)?;
                let hash = DynamicClientFeaturesConfigHash::from_json_string(&json);
                Ok((hash, config))
            })
            .transpose()
    }

    pub fn dynamic_server_config(
        &mut self,
    ) -> Result<Option<DynamicServerConfig>, DieselDatabaseError> {
        use crate::schema::dynamic_server_config::dsl::*;

        let value: Option<String> = dynamic_server_config
            .filter(row_type.eq(0))
            .select(config_json)
            .first(self.conn())
            .optional()
            .change_context(DieselDatabaseError::Execute)?;

        value
            .map(|json| serde_json::from_str(&json).change_context(DieselDatabaseError::Execute))
            .transpose()
    }
}
