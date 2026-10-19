use diesel::{delete, insert_into, prelude::*, update};
use error_stack::ResultExt;
use model::{
    AccountIdInternal, AppAttestationResult, ClientLanguage, ClientType,
    DynamicClientFeaturesConfig, DynamicClientFeaturesConfigHash, DynamicServerConfig,
    IpAddressInternal, PortNumber, SyncVersion, UnixTime,
};
use simple_backend_utils::{Result, db::MyRunQueryDsl};

use crate::{DieselDatabaseError, IntoDatabaseError, define_current_write_commands};

define_current_write_commands!(CurrentWriteCommonClientConfig);

impl CurrentWriteCommonClientConfig<'_> {
    /// Insert a new Apple App Attest key for an account.
    ///
    /// If the key already exists the existing row is updated with the new
    /// public key and creation time.
    pub fn insert_apple_app_attest_key(
        &mut self,
        id: AccountIdInternal,
        key_id_value: &[u8],
        public_key_value: &[u8],
        created_at_value: i64,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::apple_app_attest_keys::dsl::*;

        insert_into(apple_app_attest_keys)
            .values((
                account_id.eq(id.as_db_id()),
                key_id.eq(key_id_value),
                public_key.eq(public_key_value),
                created_at.eq(created_at_value),
            ))
            .on_conflict((account_id, key_id))
            .do_update()
            .set((
                public_key.eq(public_key_value),
                created_at.eq(created_at_value),
            ))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
    }

    /// Delete the oldest Apple App Attest keys for an account until at most
    /// `max_keys` remain.
    pub fn limit_apple_app_attest_keys(
        &mut self,
        id: AccountIdInternal,
        max_keys: i64,
    ) -> Result<(), DieselDatabaseError> {
        use crate::schema::apple_app_attest_keys::dsl::*;

        let count: i64 = apple_app_attest_keys
            .filter(account_id.eq(id.as_db_id()))
            .count()
            .get_result(self.conn())
            .change_context(DieselDatabaseError::Execute)?;

        if count > max_keys {
            let delete_count = count - max_keys;
            let oldest_ids = apple_app_attest_keys
                .filter(account_id.eq(id.as_db_id()))
                .order_by(created_at.asc())
                .select(key_id)
                .limit(delete_count)
                .load::<Vec<u8>>(self.conn())
                .change_context(DieselDatabaseError::Execute)?;

            delete(apple_app_attest_keys)
                .filter(account_id.eq(id.as_db_id()))
                .filter(key_id.eq_any(oldest_ids))
                .execute_my_conn(self.conn())
                .into_db_error(())?;
        }

        Ok(())
    }

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

    pub fn upsert_dynamic_server_config(
        &mut self,
        config: &DynamicServerConfig,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::dynamic_server_config::dsl::*;

        let config_json_value =
            serde_json::to_string(&config).change_context(DieselDatabaseError::SerdeSerialize)?;

        insert_into(dynamic_server_config)
            .values((row_type.eq(0), config_json.eq(&config_json_value)))
            .on_conflict(row_type)
            .do_update()
            .set(config_json.eq(&config_json_value))
            .execute_my_conn(self.conn())
            .into_db_error(())?;

        Ok(())
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

    pub fn update_login_session_info(
        &mut self,
        id: AccountIdInternal,
        platform: ClientType,
        port: Option<PortNumber>,
        ip: IpAddressInternal,
        attestation: Option<AppAttestationResult>,
    ) -> Result<(), DieselDatabaseError> {
        use model::schema::login_session_info::dsl::*;

        let (attestation_type, app_integrity, device_integrity, failed) = match attestation {
            Some(AppAttestationResult::Success {
                attestation_type,
                integrity,
            }) => (
                Some(attestation_type),
                Some(integrity.app_integrity),
                Some(integrity.device_integrity),
                Some(false),
            ),
            Some(AppAttestationResult::Failure { attestation_type }) => {
                (Some(attestation_type), Some(false), Some(false), Some(true))
            }
            None => (None, None, None, None),
        };

        let start_time = UnixTime::current_time();

        insert_into(login_session_info)
            .values((
                account_id.eq(id.as_db_id()),
                client_platform.eq(platform),
                client_port.eq(port),
                client_ip.eq(ip),
                start_unix_time.eq(start_time),
                app_attestation_type_number.eq(attestation_type),
                app_attestation_app_integrity.eq(app_integrity),
                app_attestation_device_integrity.eq(device_integrity),
                app_attestation_failed.eq(failed),
            ))
            .on_conflict(account_id)
            .do_update()
            .set((
                client_platform.eq(platform),
                client_port.eq(port),
                client_ip.eq(ip),
                start_unix_time.eq(start_time),
                app_attestation_type_number.eq(attestation_type),
                app_attestation_app_integrity.eq(app_integrity),
                app_attestation_device_integrity.eq(device_integrity),
                app_attestation_failed.eq(failed),
            ))
            .execute_my_conn(self.conn())
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
