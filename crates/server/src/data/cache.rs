use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use config::Config;
use database::{
    current::read::{CurrentSyncReadCommands, SqliteReadCommands},
    diesel::{DieselCurrentReadHandle, DieselDatabaseError},
    ConvertCommandError, NoId,
};
use error_stack::{Result, ResultExt};
use model::{
    Account, AccountIdInternal, AccountIdLight, ApiKey, LocationIndexKey, ProfileInternal,
    ProfileUpdateInternal,
};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tracing::info;
use utils::{ComponentError, IntoReportExt, IntoReportFromString};

use super::index::{
    location::LocationIndexIteratorState, LocationIndexIteratorGetter, LocationIndexWriterGetter,
};

impl ComponentError for CacheError {
    const COMPONENT_NAME: &'static str = "Cache";
}

#[derive(thiserror::Error, Debug)]
pub enum CacheError {
    #[error("Key already exists")]
    AlreadyExists,

    #[error("Key not exists")]
    KeyNotExists,

    #[error("Data is not in cache")]
    NotInCache,

    #[error("Cache init error")]
    Init,

    #[error("Cache operation failed because of server feature was not enabled")]
    FeatureNotEnabled,
}

#[derive(Debug)]
pub struct AccountEntry {
    pub account_id_internal: AccountIdInternal,
    pub cache: RwLock<CacheEntry>,
}

#[derive(Debug)]
pub struct DatabaseCache {
    /// Accounts which are logged in.
    api_keys: RwLock<HashMap<ApiKey, Arc<AccountEntry>>>,
    /// All accounts registered in the service.
    accounts: RwLock<HashMap<AccountIdLight, Arc<AccountEntry>>>,
}

impl DatabaseCache {
    pub async fn new(
        read: SqliteReadCommands<'_>,
        read_diesel: DieselCurrentReadHandle,
        index_iterator: LocationIndexIteratorGetter<'_>,
        index_writer: LocationIndexWriterGetter<'_>,
        config: &Config,
    ) -> Result<Self, CacheError> {
        let cache = Self {
            api_keys: RwLock::new(HashMap::new()),
            accounts: RwLock::new(HashMap::new()),
        };

        // Load data from database to memory.
        info!("Starting to load data from database to memory");

        let account = read.account();
        let mut accounts = account.account_ids_stream();
        while let Some(r) = accounts.next().await {
            let id = r.attach(NoId).change_context(CacheError::Init)?;
            // Diesel connection used here so no deadlock
            cache
                .load_account_from_db(id, &config, &read_diesel, &index_iterator, &index_writer)
                .await?;
        }

        info!("Loading to memory complete");
        Ok(cache)
    }

    pub async fn load_state_from_external_services() {
        // TODO
        //index_writer.update_profile_link(internal_id.as_light(), ProfileLink::new(internal_id.as_light(), &profile_data.data), location_key).await;
    }

    pub async fn load_account_from_db(
        &self,
        account_id: AccountIdInternal,
        config: &Config,
        read_diesel: &DieselCurrentReadHandle,
        index_iterator: &LocationIndexIteratorGetter<'_>,
        index_writer: &LocationIndexWriterGetter<'_>,
    ) -> Result<(), CacheError> {
        self.insert_account_if_not_exists(account_id)
            .await
            .attach_printable(account_id)?;

        let read_lock = self.accounts.read().await;
        let account_entry = read_lock
            .get(&account_id.as_light())
            .ok_or(CacheError::KeyNotExists)?;

        let api_key = db_read(&read_diesel, move |cmds| {
            cmds.account().access_token(account_id)
        })
        .await?;

        if let Some(key) = api_key {
            let mut write_api_keys = self.api_keys.write().await;
            if write_api_keys.contains_key(&key) {
                return Err(CacheError::AlreadyExists.into()).change_context(CacheError::Init);
            } else {
                write_api_keys.insert(key, account_entry.clone());
            }
        }

        let mut entry = account_entry.cache.write().await;

        if config.components().account {
            let account =
                db_read(&read_diesel, move |cmds| cmds.account().account(account_id)).await?;
            entry.account = Some(account.clone().into())
        }

        if config.components().profile {
            let profile =
                db_read(&read_diesel, move |cmds| cmds.profile().profile(account_id)).await?;

            let mut profile_data: CachedProfile = profile.into();

            let location_key = db_read(&read_diesel, move |cmds| {
                cmds.profile().location_index_key(account_id)
            })
            .await?;
            profile_data.location.current_position = location_key;
            let index_iterator = index_iterator.get().ok_or(CacheError::FeatureNotEnabled)?;
            profile_data.location.current_iterator =
                index_iterator.reset_iterator(profile_data.location.current_iterator, location_key);

            // TODO: Add to location index only if visiblity is public
            let _index_writer = index_writer.get().ok_or(CacheError::FeatureNotEnabled)?;
            //index_writer.update_profile_link(internal_id.as_light(), ProfileLink::new(internal_id.as_light(), &profile_data.data), location_key).await;

            entry.profile = Some(Box::new(profile_data));
        }

        Ok(())
    }

    pub async fn insert_account_if_not_exists(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), CacheError> {
        let mut data = self.accounts.write().await;
        if data.get(&id.as_light()).is_none() {
            let value = RwLock::new(CacheEntry::new());
            data.insert(
                id.as_light(),
                AccountEntry {
                    cache: value,
                    account_id_internal: id,
                }
                .into(),
            );
            Ok(())
        } else {
            Err(CacheError::AlreadyExists.into())
        }
    }

    pub async fn update_access_token_and_connection(
        &self,
        id: AccountIdLight,
        current_access_token: Option<ApiKey>,
        new_access_token: ApiKey,
        address: Option<SocketAddr>,
    ) -> Result<(), CacheError> {
        let cache_entry = self
            .accounts
            .read()
            .await
            .get(&id)
            .ok_or(CacheError::KeyNotExists)?
            .clone();

        let mut tokens = self.api_keys.write().await;

        if let Some(current) = current_access_token {
            tokens.remove(&current);
        }

        // Avoid collisions.
        if tokens.get(&new_access_token).is_none() {
            cache_entry.cache.write().await.current_connection = address;
            tokens.insert(new_access_token, cache_entry);
            Ok(())
        } else {
            Err(CacheError::AlreadyExists.into())
        }
    }

    pub async fn delete_access_token_and_connection(
        &self,
        id: AccountIdLight,
        token: Option<ApiKey>,
    ) -> Result<(), CacheError> {
        let cache_entry = self
            .accounts
            .read()
            .await
            .get(&id)
            .ok_or(CacheError::KeyNotExists)?
            .clone();

        cache_entry.cache.write().await.current_connection = None;

        if let Some(token) = token {
            let mut tokens = self.api_keys.write().await;
            let _account = tokens.remove(&token).ok_or(CacheError::KeyNotExists)?;
        }

        Ok(())
    }

    pub async fn access_token_exists(&self, token: &ApiKey) -> Option<AccountIdInternal> {
        let tokens = self.api_keys.read().await;
        if let Some(entry) = tokens.get(token) {
            Some(entry.account_id_internal)
        } else {
            None
        }
    }

    /// Checks that connection comes from the same IP address. WebSocket is
    /// using the cached SocketAddr, so check the IP only.
    pub async fn access_token_and_connection_exists(
        &self,
        access_token: &ApiKey,
        connection: SocketAddr,
    ) -> Option<AccountIdInternal> {
        let tokens = self.api_keys.read().await;
        if let Some(entry) = tokens.get(access_token) {
            let r = entry.cache.read().await;
            if r.current_connection.map(|a| a.ip()) == Some(connection.ip()) {
                Some(entry.account_id_internal)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub async fn to_account_id_internal(
        &self,
        id: AccountIdLight,
    ) -> Result<AccountIdInternal, CacheError> {
        let guard = self.accounts.read().await;
        let data = guard
            .get(&id)
            .ok_or(CacheError::KeyNotExists)?
            .account_id_internal;
        Ok(data)
    }

    pub async fn read_cache<T, Id: Into<AccountIdLight>>(
        &self,
        id: Id,
        cache_operation: impl Fn(&CacheEntry) -> T,
    ) -> Result<T, CacheError> {
        let guard = self.accounts.read().await;
        let cache_entry = guard
            .get(&id.into())
            .ok_or(CacheError::KeyNotExists)?
            .cache
            .read()
            .await;
        Ok(cache_operation(&cache_entry))
    }

    pub async fn write_cache<T, Id: Into<AccountIdLight>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CacheEntry) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        let guard = self.accounts.read().await;
        let mut cache_entry = guard
            .get(&id.into())
            .ok_or(CacheError::KeyNotExists)?
            .cache
            .write()
            .await;
        Ok(cache_operation(&mut cache_entry)?)
    }

    pub async fn account(&self, id: AccountIdLight) -> Result<Account, CacheError> {
        let guard = self.accounts.read().await;
        let data = guard
            .get(&id)
            .ok_or(CacheError::KeyNotExists)?
            .cache
            .read()
            .await
            .account
            .as_ref()
            .map(|data| data.as_ref().clone())
            .ok_or(CacheError::NotInCache)?;

        Ok(data)
    }

    pub async fn update_account(
        &self,
        id: AccountIdLight,
        data: Account,
    ) -> Result<(), CacheError> {
        let mut write_guard = self.accounts.write().await;
        write_guard
            .get_mut(&id)
            .ok_or(CacheError::KeyNotExists)?
            .cache
            .write()
            .await
            .account
            .as_mut()
            .ok_or(CacheError::NotInCache)
            .map(|current_data| *current_data.as_mut() = data)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct CachedProfile {
    /// If None there is no profile visibility value fetched from account server.
    pub public: Option<bool>,
    pub data: ProfileInternal,
    pub location: LocationData,
}

impl From<ProfileInternal> for CachedProfile {
    fn from(value: ProfileInternal) -> Self {
        Self {
            public: None,
            data: value,
            location: LocationData {
                current_position: LocationIndexKey::default(),
                current_iterator: LocationIndexIteratorState::new(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocationData {
    pub current_position: LocationIndexKey,
    pub current_iterator: LocationIndexIteratorState,
}

#[derive(Debug)]
pub struct CacheEntry {
    pub profile: Option<Box<CachedProfile>>,
    pub account: Option<Box<Account>>,
    pub current_connection: Option<SocketAddr>,
}

impl CacheEntry {
    pub fn new() -> Self {
        Self {
            profile: None,
            account: None,
            current_connection: None,
        }
    }
}

#[async_trait]
pub trait ReadCacheJson: Sized + Send {
    const CACHED_JSON: bool = false;

    async fn read_from_cache(
        _id: AccountIdLight,
        _cache: &DatabaseCache,
    ) -> Result<Self, CacheError> {
        Err(CacheError::NotInCache.into())
    }
}

//     async fn read_from_cache(
//         id: AccountIdLight,
//         cache: &DatabaseCache,
//     ) -> Result<Self, CacheError> {
//         let data_in_cache = cache
//             .read_cache(id, |entry| {
//                 entry.profile.as_ref().map(|data| data.data.clone())
//             })
//             .await
//             .attach(id)?;
//         data_in_cache
//             .ok_or(CacheError::NotInCache.into())
//             .map(|p| p)
//     }
// }

// #[async_trait]
// impl ReadCacheJson for Profile {
//     const CACHED_JSON: bool = true;

//     async fn read_from_cache(
//         id: AccountIdLight,
//         cache: &DatabaseCache,
//     ) -> Result<Self, CacheError> {
//         let data_in_cache = cache
//             .read_cache(id, |entry| {
//                 entry
//                     .profile
//                     .as_ref()
//                     .map(|data| data.as_ref().data.clone().into())
//             })
//             .await
//             .attach(id)?;
//         data_in_cache.ok_or(CacheError::NotInCache.into())
//     }
// }

#[async_trait]
pub trait WriteCacheJson: Sized + Send {
    async fn write_to_cache(
        &self,
        _id: AccountIdLight,
        _cache: &DatabaseCache,
    ) -> Result<(), CacheError> {
        Ok(())
    }
}

// impl WriteCacheJson for AccountSetup {}

#[async_trait]
impl WriteCacheJson for Account {
    async fn write_to_cache(
        &self,
        id: AccountIdLight,
        cache: &DatabaseCache,
    ) -> Result<(), CacheError> {
        cache
            .write_cache(id, |entry| {
                entry
                    .account
                    .as_mut()
                    .map(|data| *data.as_mut() = self.clone());
                Ok(())
            })
            .await
            .map(|_| ())
            .attach(id)
    }
}

#[async_trait]
impl WriteCacheJson for ProfileInternal {
    async fn write_to_cache(
        &self,
        id: AccountIdLight,
        cache: &DatabaseCache,
    ) -> Result<(), CacheError> {
        cache
            .write_cache(id, |entry| {
                entry
                    .profile
                    .as_mut()
                    .map(|data| data.as_mut().data = self.clone());
                Ok(())
            })
            .await
            .map(|_| ())
            .attach(id)
    }
}

#[async_trait]
impl WriteCacheJson for ProfileUpdateInternal {
    async fn write_to_cache(
        &self,
        id: AccountIdLight,
        cache: &DatabaseCache,
    ) -> Result<(), CacheError> {
        cache
            .write_cache(id, |entry| {
                entry.profile.as_mut().map(|d| &mut d.data).map(|data| {
                    data.profile_text = self.new_data.profile_text.clone();
                    data.version_uuid = self.version;
                });
                Ok(())
            })
            .await
            .map(|_| ())
            .attach(id)
    }
}

async fn db_read<
    T: FnOnce(CurrentSyncReadCommands<'_>) -> Result<R, DieselDatabaseError> + Send + 'static,
    R: Send + 'static,
>(
    read: &DieselCurrentReadHandle,
    cmd: T,
) -> Result<R, CacheError> {
    let conn = read
        .pool()
        .get()
        .await
        .into_error(DieselDatabaseError::GetConnection)
        .change_context(CacheError::Init)?;

    conn.interact(move |conn| cmd(CurrentSyncReadCommands::new(conn)))
        .await
        .into_error_string(DieselDatabaseError::Execute)
        .change_context(CacheError::Init)?
        .change_context(CacheError::Init)
}
