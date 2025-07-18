use error_stack::Result;
use model::AccountId;
use server_data::{
    cache::{CacheError, chat::CacheChat},
    db_manager::InternalWriting,
};

pub trait CacheReadChat {
    async fn read_cache_chat<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&CacheChat) -> Result<T, CacheError>,
    ) -> Result<T, CacheError>;
}

pub trait CacheWriteChat {
    async fn write_cache_chat<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CacheChat) -> Result<T, CacheError>,
    ) -> Result<T, CacheError>;
}

impl<I: InternalWriting> CacheWriteChat for I {
    async fn write_cache_chat<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CacheChat) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        self.cache()
            .write_cache(id, |e| cache_operation(e.chat_data_mut()?))
            .await
    }
}
