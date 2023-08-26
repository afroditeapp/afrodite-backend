use std::net::SocketAddr;

use database::{current::read::SqliteReadCommands, sqlite::SqlxReadHandle};
use error_stack::{Result, ResultExt};
use model::{AccessToken, AccountId, AccountIdInternal};

use super::{cache::DatabaseCache, DataError, IntoDataError};

pub struct AccessTokenManager<'a> {
    cache: &'a DatabaseCache,
}

impl<'a> AccessTokenManager<'a> {
    pub fn new(cache: &'a DatabaseCache) -> Self {
        Self { cache }
    }

    pub async fn access_token_exists(&self, token: &AccessToken) -> Option<AccountIdInternal> {
        self.cache.access_token_exists(token).await
    }

    pub async fn access_token_and_connection_exists(
        &self,
        token: &AccessToken,
        connection: SocketAddr,
    ) -> Option<AccountIdInternal> {
        self.cache
            .access_token_and_connection_exists(token, connection)
            .await
    }
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

    pub async fn get_internal_id(&self, id: AccountId) -> Result<AccountIdInternal, DataError> {
        self.cache
            .to_account_id_internal(id)
            .await
            .into_data_error(id)
    }
}
