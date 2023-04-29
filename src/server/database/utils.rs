

use error_stack::Result;




use crate::{api::model::{
    Account, AccountIdInternal, AccountIdLight, AccountSetup, ApiKey, Profile, ProfileUpdateInternal, ProfileInternal,
}, utils::ConvertCommandError};

use super::{
    cache::{CacheError, DatabaseCache},
};

pub fn current_unix_time() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}

pub struct ApiKeyManager<'a> {
    cache: &'a DatabaseCache,
}

impl<'a> ApiKeyManager<'a> {
    pub fn new(cache: &'a DatabaseCache) -> Self {
        Self { cache }
    }

    pub async fn api_key_exists(&self, api_key: &ApiKey) -> Option<AccountIdInternal> {
        self.cache.api_key_exists(api_key).await
    }

    // pub async fn update_api_key(&self, id: AccountIdLight, api_key: ApiKey) -> Result<(), CacheError> {
    //     self.cache.update_api_key(id, api_key).await
    // }

    // pub async fn delete_api_key(&self, api_key: ApiKey) -> Result<(), CacheError> {
    //     self.cache.delete_api_key(api_key).await
    // }
}

pub struct AccountIdManager<'a> {
    cache: &'a DatabaseCache,
}

impl<'a> AccountIdManager<'a> {
    pub fn new(cache: &'a DatabaseCache) -> Self {
        Self { cache }
    }

    pub async fn get_internal_id(
        &self,
        id: AccountIdLight,
    ) -> Result<AccountIdInternal, CacheError> {
        self.cache.to_account_id_internal(id).await.attach(id)
    }
}
