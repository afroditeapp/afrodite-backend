use error_stack::Result;
use model_profile::AccountId;
use server_data::{
    cache::{CacheError, common::CacheEntryCommon, profile::CachedProfile},
    db_manager::{InternalReading, InternalWriting},
};

pub trait CacheReadProfile {
    async fn read_cache_profile_and_common<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&CachedProfile, &CacheEntryCommon) -> Result<T, CacheError>,
    ) -> Result<T, CacheError>;

    async fn read_cache_profile_and_common_for_all_accounts(
        &self,
        cache_operation: impl FnMut(&CachedProfile, &CacheEntryCommon),
    ) -> Result<(), CacheError>;
}

impl<R: InternalReading> CacheReadProfile for R {
    async fn read_cache_profile_and_common<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&CachedProfile, &CacheEntryCommon) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        self.cache()
            .read_cache(id, |e| {
                let p = e
                    .profile
                    .as_deref()
                    .ok_or(CacheError::FeatureNotEnabled.report())?;
                cache_operation(p, &e.common)
            })
            .await
    }

    async fn read_cache_profile_and_common_for_all_accounts(
        &self,
        mut cache_operation: impl FnMut(&CachedProfile, &CacheEntryCommon),
    ) -> Result<(), CacheError> {
        self.cache()
            .read_cache_for_all_accounts(|_, e| {
                let p = e
                    .profile
                    .as_deref()
                    .ok_or(CacheError::FeatureNotEnabled.report())?;
                cache_operation(p, &e.common);
                Ok(())
            })
            .await
    }
}

pub trait CacheWriteProfile {
    async fn write_cache_profile<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CachedProfile) -> Result<T, CacheError>,
    ) -> Result<T, CacheError>;

    async fn write_cache_profile_and_common<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CachedProfile, &mut CacheEntryCommon) -> Result<T, CacheError>,
    ) -> Result<T, CacheError>;
}

impl<I: InternalWriting> CacheWriteProfile for I {
    async fn write_cache_profile<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CachedProfile) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        self.cache()
            .write_cache(id, |e| {
                let p = e.profile_data_mut()?;
                cache_operation(p)
            })
            .await
    }

    async fn write_cache_profile_and_common<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CachedProfile, &mut CacheEntryCommon) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        self.cache()
            .write_cache(id, |e| {
                let p = e
                    .profile
                    .as_deref_mut()
                    .ok_or(CacheError::FeatureNotEnabled.report())?;
                cache_operation(p, &mut e.common)
            })
            .await
    }
}
