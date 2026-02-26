use diesel::{insert_into, prelude::*, update};
use error_stack::{Result, ResultExt};
use model::{
    AccountIdInternal, ClientLanguage, ClientType, DynamicClientFeaturesConfig,
    DynamicClientFeaturesConfigHash, SyncVersion,
};
use simple_backend_utils::db::MyRunQueryDsl;

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_read_commands};

define_current_read_commands!(CurrentWriteCommonClientConfig);

impl CurrentWriteCommonClientConfig<'_> {
    pub fn upsert_dynamic_client_features_config(
        &mut self,
        config: &DynamicClientFeaturesConfig,
    ) -> Result<DynamicClientFeaturesConfigHash, DieselDatabaseError> {
        use model::schema::dynamic_client_features_config::dsl::*;

        let config_json_value =
            serde_json::to_string(&config).change_context(DieselDatabaseError::SerdeSerialize)?;

        insert_into(dynamic_client_features_config)
            .values((row_type.eq(0), config_json.eq(&config_json_value)))
            .on_conflict(row_type)
            .do_update()
            .set(config_json.eq(&config_json_value))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(DynamicClientFeaturesConfigHash::from_json_string(
            &config_json_value,
        ))
    }

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

    pub fn update_client_language(
        &mut self,
        id: AccountIdInternal,
        value: ClientLanguage,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::common_state::dsl::*;

        update(common_state)
            .filter(account_id.eq(id.as_db_id()))
            .set(client_language.eq(value))
            .execute(self.conn())
            .into_db_error(())?;

        Ok(())
    }
}
