use database::{define_current_read_commands, DieselDatabaseError};
use diesel::prelude::*;
use error_stack::Result;
use model::{PublicKeyId, PublicKeyVersion};
use model_chat::{
    AccountIdInternal, PublicKey, PublicKeyData
};

use crate::IntoDatabaseError;

define_current_read_commands!(CurrentReadChatPublicKey);

impl CurrentReadChatPublicKey<'_> {
    pub fn public_key(
        &mut self,
        account_id_value: AccountIdInternal,
        version: PublicKeyVersion,
    ) -> Result<Option<PublicKey>, DieselDatabaseError> {
        use crate::schema::public_key::dsl::*;

        let query_result: Option<(Option<PublicKeyId>, Option<PublicKeyData>)> = public_key
            .filter(account_id.eq(account_id_value.as_db_id()))
            .filter(public_key_version.eq(version))
            .select((public_key_id, public_key_data))
            .first(self.conn())
            .optional()
            .into_db_error(())?;

        if let Some((Some(id), Some(data))) = query_result {
            Ok(Some(PublicKey { id, version, data }))
        } else {
            Ok(None)
        }
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
