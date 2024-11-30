use error_stack::Result;
use model::AccountId;
use server_data::{
    cache::{media::CachedMedia, CacheError},
    db_manager::InternalWriting,
};

pub trait CacheReadMedia {
    async fn read_cache_media<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&CachedMedia) -> Result<T, CacheError>,
    ) -> Result<T, CacheError>;
}

pub trait CacheWriteMedia {
    async fn write_cache_media<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CachedMedia) -> Result<T, CacheError>,
    ) -> Result<T, CacheError>;
}

impl<I: InternalWriting> CacheWriteMedia for I {
    async fn write_cache_media<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CachedMedia) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        self.cache()
            .write_cache(id, |e| {
                let m = e.media_data_mut()?;
                cache_operation(m)
            })
            .await
    }
}
