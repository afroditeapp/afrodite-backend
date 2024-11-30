use error_stack::Result;
use model::{AccountId, AccountIdInternal};
use server_common::data::cache::CacheError;

use crate::db_manager::InternalReading;

pub trait ToAccountIdInternal {
    async fn to_account_id_internal(&self, id: AccountId) -> Result<AccountIdInternal, CacheError>;
}

impl<T: InternalReading> ToAccountIdInternal for T {
    async fn to_account_id_internal(&self, id: AccountId) -> Result<AccountIdInternal, CacheError> {
        self.cache()
            .to_account_id_internal_optional(id)
            .await
            .ok_or(CacheError::KeyNotExists.report())
    }
}
