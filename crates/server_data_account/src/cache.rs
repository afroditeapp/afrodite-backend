use error_stack::Result;
use model::AccountId;
use server_data::{
    cache::{CacheError, account::CacheAccount},
    db_manager::InternalReading,
};

pub trait CacheWriteAccount {
    async fn write_cache_account<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CacheAccount) -> Result<T, CacheError>,
    ) -> Result<T, CacheError>;
}

impl<I: InternalReading> CacheWriteAccount for I {
    async fn write_cache_account<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CacheAccount) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        self.cache()
            .write_cache(id, |e| cache_operation(e.account_data_mut()))
            .await
    }
}
