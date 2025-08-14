use database::{DieselDatabaseError, define_current_read_commands};
use diesel::prelude::*;
use error_stack::Result;
use model::{PublicKeyId, UnixTime};
use model_chat::{AccountIdInternal, DataExportPublicKey};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadChatPublicKey);

impl CurrentReadChatPublicKey<'_> {
    pub fn latest_public_key_id(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<Option<PublicKeyId>, DieselDatabaseError> {
        use crate::schema::public_key::dsl::*;

        let query_result: Option<PublicKeyId> = public_key
            .filter(account_id.eq(account_id_value.as_db_id()))
            .select(key_id)
            .order(key_id.desc())
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(query_result)
    }

    pub fn public_key_data(
        &mut self,
        account_id_value: AccountIdInternal,
        key_id_value: PublicKeyId,
    ) -> Result<Option<Vec<u8>>, DieselDatabaseError> {
        use crate::schema::public_key::dsl::*;

        let query_result: Option<Vec<u8>> = public_key
            .filter(account_id.eq(account_id_value.as_db_id()))
            .filter(key_id.eq(key_id_value))
            .select(key_data)
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        Ok(query_result)
    }

    pub fn all_public_keys(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<Vec<DataExportPublicKey>, DieselDatabaseError> {
        use crate::schema::public_key::dsl::*;

        let query_result: Vec<(PublicKeyId, Vec<u8>, UnixTime)> = public_key
            .filter(account_id.eq(account_id_value.as_db_id()))
            .select((key_id, key_data, key_added_unix_time))
            .load(self.conn())
            .into_db_error(())?;

        let data = query_result
            .into_iter()
            .map(|(id, data, time)| DataExportPublicKey::new(id, data, time))
            .collect();

        Ok(data)
    }

    pub fn max_public_key_count_account_config(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<i64, DieselDatabaseError> {
        use crate::schema::chat_state::dsl::*;

        let value: i64 = chat_state
            .find(account_id_value.as_db_id())
            .select(max_public_key_count)
            .first(self.conn())
            .into_db_error(())?;

        Ok(value)
    }
}
