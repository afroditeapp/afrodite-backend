use std::net::SocketAddr;

use model::{AccessToken, AccountId, AccountIdInternal, Capabilities};

use super::{cache::DatabaseCache, DataError, IntoDataError};
use crate::result::Result;

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

    /// Check that token and current connection IP and port matches
    /// with WebSocket connection.
    pub async fn access_token_and_connection_exists(
        &self,
        token: &AccessToken,
        connection: SocketAddr,
    ) -> Option<(AccountIdInternal, Capabilities)> {
        self.cache
            .access_token_and_connection_exists(token, connection)
            .await
    }
}

pub struct AccountIdManager<'a> {
    cache: &'a DatabaseCache,
}

impl<'a> AccountIdManager<'a> {
    pub fn new(cache: &'a DatabaseCache) -> Self {
        Self { cache }
    }

    pub async fn get_internal_id(&self, id: AccountId) -> Result<AccountIdInternal, DataError> {
        self.cache
            .to_account_id_internal(id)
            .await
            .into_data_error(id)
    }
}
