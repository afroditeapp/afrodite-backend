use std::net::SocketAddr;

use error_stack::Result;

use crate::utils::ConvertCommandErrorExt;
use database::{current::read::SqliteReadCommands, ConvertCommandError};
use model::{AccountIdInternal, AccountIdLight, ApiKey, GoogleAccountId};

use super::{
    cache::{CacheError, DatabaseCache},
    DatabaseError,
};
use database::sqlite::SqlxReadHandle;
use database::DatabaseId;

pub struct ApiKeyManager<'a> {
    cache: &'a DatabaseCache,
}

impl<'a> ApiKeyManager<'a> {
    pub fn new(cache: &'a DatabaseCache) -> Self {
        Self { cache }
    }

    pub async fn api_key_exists(&self, api_key: &ApiKey) -> Option<AccountIdInternal> {
        self.cache.access_token_exists(api_key).await
    }

    pub async fn api_key_and_connection_exists(
        &self,
        api_key: &ApiKey,
        connection: SocketAddr,
    ) -> Option<AccountIdInternal> {
        self.cache
            .access_token_and_connection_exists(api_key, connection)
            .await
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
    read_handle: SqliteReadCommands<'a>,
}

impl<'a> AccountIdManager<'a> {
    pub fn new(cache: &'a DatabaseCache, read_handle: &'a SqlxReadHandle) -> Self {
        Self {
            cache,
            read_handle: SqliteReadCommands::new(read_handle),
        }
    }

    pub async fn get_internal_id(
        &self,
        id: AccountIdLight,
    ) -> Result<AccountIdInternal, CacheError> {
        self.cache.to_account_id_internal(id).await.attach(id)
    }

    pub async fn get_account_with_google_account_id(
        &self,
        id: GoogleAccountId,
    ) -> Result<Option<AccountIdInternal>, DatabaseError> {
        self.read_handle
            .account()
            .get_account_with_google_account_id(id)
            .await
            .convert(DatabaseId::Empty)
    }
}
