use database::{define_current_read_commands, DieselDatabaseError, IntoDatabaseError};
use diesel::{
    query_dsl::methods::{FilterDsl, SelectDsl},
    ExpressionMethods, NullableExpressionMethods, RunQueryDsl,
};
use error_stack::Result;
use model::{AccountIdInternal, PublicKeyId, PublicKeyIdAndVersion, PublicKeyVersion};

define_current_read_commands!(CurrentReadChatUtils);

impl CurrentReadChatUtils<'_> {
    pub fn get_latest_public_keys_info(
        &mut self,
        account_id_value: AccountIdInternal,
    ) -> Result<Vec<PublicKeyIdAndVersion>, DieselDatabaseError> {
        use crate::schema::public_key::dsl::*;

        let query_result: Vec<(PublicKeyId, PublicKeyVersion)> = public_key
            .filter(account_id.eq(account_id_value.as_db_id()))
            .filter(public_key_id.is_not_null())
            .select((public_key_id.assume_not_null(), public_key_version))
            .load(self.conn())
            .into_db_error(())?;

        let info_list = query_result
            .into_iter()
            .map(|(id, version)| PublicKeyIdAndVersion { id, version })
            .collect::<Vec<_>>();

        Ok(info_list)
    }
}
