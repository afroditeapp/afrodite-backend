use std::{collections::HashMap, fmt::Debug, net::SocketAddr, sync::Arc};

use config::Config;
use database::{current::read::CurrentSyncReadCommands, CurrentReadHandle};
use error_stack::{Result, ResultExt};
use model::{
    AccessToken, AccountId, AccountIdInternal, Capabilities, LocationIndexKey, ProfileInternal,
    ProfileLink, SharedState,
};
use simple_backend_database::diesel_db::{DieselConnection, DieselDatabaseError};
use simple_backend_utils::{ComponentError, IntoReportFromString};
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use tracing::info;

use super::{
    index::{
        location::LocationIndexIteratorState, LocationIndexIteratorHandle, LocationIndexManager,
    },
    WithInfo,
};
use crate::{data::index::LocationIndexWriteHandle, event::EventMode};

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

impl CacheError {
    #[track_caller]
    pub fn report(self) -> error_stack::Report<Self> {
        error_stack::report!(self)
    }

    #[track_caller]
    pub fn error<Ok>(self) -> error_stack::Result<Ok, Self> {
        Err(error_stack::report!(self))
    }
}

#[derive(Debug)]
pub struct AccountEntry {
    pub account_id_internal: AccountIdInternal,
    pub cache: RwLock<CacheEntry>,
}

#[derive(Debug)]
pub struct DatabaseCache {
    /// Accounts which are logged in (have valid access token).
    access_tokens: RwLock<HashMap<AccessToken, Arc<AccountEntry>>>,
    /// All accounts registered in the service.
    accounts: RwLock<HashMap<AccountId, Arc<AccountEntry>>>,
}

impl DatabaseCache {
    pub async fn new(
        current_db: &CurrentReadHandle,
        location_index: &LocationIndexManager,
        config: &Config,
    ) -> Result<Self, CacheError> {
        let read = current_db.sqlx_cmds();

        let cache = Self {
            access_tokens: RwLock::new(HashMap::new()),
            accounts: RwLock::new(HashMap::new()),
        };

        // Load data from database to memory.
        info!("Starting to load data from database to memory");

        let account = read.account();
        let data = account.data();
        let mut accounts = data.account_ids_stream();
        while let Some(r) = accounts.next().await {
            let id = r.change_context(CacheError::Init)?;
            // Diesel connection used here so no deadlock
            cache
                .load_account_from_db(
                    id,
                    &config,
                    &current_db,
                    LocationIndexIteratorHandle::new(location_index),
                    LocationIndexWriteHandle::new(location_index),
                )
                .await
                .change_context(CacheError::Init)?;
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
        current_db: &CurrentReadHandle,
        index_iterator: LocationIndexIteratorHandle<'_>,
        index_writer: LocationIndexWriteHandle<'_>,
    ) -> Result<(), CacheError> {
        self.insert_account_if_not_exists(account_id)
            .await
            .with_info(account_id)?;

        let read_lock = self.accounts.read().await;
        let account_entry = read_lock
            .get(&account_id.as_id())
            .ok_or(CacheError::KeyNotExists.report())?;

        let access_token = db_read(current_db, move |mut cmds| {
            cmds.account().token().access_token(account_id)
        })
        .await?;

        if let Some(key) = access_token {
            let mut access_tokens = self.access_tokens.write().await;
            if access_tokens.contains_key(&key) {
                return Err(CacheError::AlreadyExists.report());
            } else {
                access_tokens.insert(key, account_entry.clone());
            }
        }

        let mut entry = account_entry.cache.write().await;

        // Common
        let capabilities = db_read(current_db, move |mut cmds| {
            cmds.common().state().account_capabilities(account_id)
        })
        .await?;
        entry.capabilities = capabilities;
        let state = db_read(current_db, move |mut cmds| {
            cmds.common().state().shared_state(account_id)
        })
        .await?;
        entry.shared_state = state;

        if config.components().account {
            // empty
        }

        if config.components().profile {
            let profile = db_read(current_db, move |mut cmds| {
                cmds.profile().data().profile(account_id)
            })
            .await?;
            let profile_location = db_read(current_db, move |mut cmds| {
                cmds.profile().data().profile_location(account_id)
            })
            .await?;

            let mut profile_data: CachedProfile = profile.into();

            let location_key = index_writer.coordinates_to_key(&profile_location);
            profile_data.location.current_position = location_key;
            profile_data.location.current_iterator =
                index_iterator.reset_iterator(profile_data.location.current_iterator, location_key);

            // TODO: Add to location index only if visiblity is public
            //       Was the visiblity basically stored only on account server?
            //       If so, perhaps the best and clearest option would be creating
            //       new media and profile server specific tables for storing
            //       cached account server state.
            //       Update: simple solution for now, when also account server
            //       mode is enabled then use the visibility value from account.
            if config.components().account {
                let account = db_read(current_db, move |mut cmds| {
                    cmds.account().data().account(account_id)
                })
                .await?;
                if account.capablities().user_view_public_profiles {
                    index_writer
                        .update_profile_link(
                            account_id.uuid,
                            ProfileLink::new(account_id.uuid, &profile_data.data),
                            location_key,
                        )
                        .await
                        .change_context(CacheError::Init)?;
                }
            }

            entry.profile = Some(Box::new(profile_data));
        }

        Ok(())
    }

    pub async fn insert_account_if_not_exists(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), CacheError> {
        let mut data = self.accounts.write().await;
        if data.get(&id.as_id()).is_none() {
            let value = RwLock::new(CacheEntry::new());
            data.insert(
                id.as_id(),
                AccountEntry {
                    cache: value,
                    account_id_internal: id,
                }
                .into(),
            );
            Ok(())
        } else {
            Err(CacheError::AlreadyExists.report())
        }
    }

    pub async fn update_access_token_and_connection(
        &self,
        id: AccountId,
        current_access_token: Option<AccessToken>,
        new_access_token: AccessToken,
        address: Option<SocketAddr>,
    ) -> Result<(), CacheError> {
        let cache_entry = self
            .accounts
            .read()
            .await
            .get(&id)
            .ok_or(CacheError::KeyNotExists)?
            .clone();

        let mut tokens = self.access_tokens.write().await;

        if let Some(current) = current_access_token {
            tokens.remove(&current);
        }

        // Avoid collisions.
        if tokens.get(&new_access_token).is_none() {
            cache_entry.cache.write().await.current_connection = address;
            tokens.insert(new_access_token, cache_entry);
            Ok(())
        } else {
            Err(CacheError::AlreadyExists.report())
        }
    }

    /// Delete connection. Also delete access token if it is Some.
    pub async fn delete_connection_and_specific_access_token(
        &self,
        id: AccountId,
        token: Option<AccessToken>,
    ) -> Result<(), CacheError> {
        let cache_entry = self
            .accounts
            .read()
            .await
            .get(&id)
            .ok_or(CacheError::KeyNotExists)?
            .clone();

        cache_entry.cache.write().await.current_connection = None;
        cache_entry.cache.write().await.current_event_connection = EventMode::None;

        if let Some(token) = token {
            let mut tokens = self.access_tokens.write().await;
            let _account = tokens.remove(&token).ok_or(CacheError::KeyNotExists)?;
        }

        Ok(())
    }

    pub async fn access_token_exists(&self, token: &AccessToken) -> Option<AccountIdInternal> {
        let tokens = self.access_tokens.read().await;
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
        access_token: &AccessToken,
        connection: SocketAddr,
    ) -> Option<(AccountIdInternal, Capabilities)> {
        let tokens = self.access_tokens.read().await;
        if let Some(entry) = tokens.get(access_token) {
            let r = entry.cache.read().await;
            if r.current_connection.map(|a| a.ip()) == Some(connection.ip()) {
                Some((entry.account_id_internal, r.capabilities.clone()))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub async fn to_account_id_internal(
        &self,
        id: AccountId,
    ) -> Result<AccountIdInternal, CacheError> {
        let guard = self.accounts.read().await;
        let data = guard
            .get(&id)
            .ok_or(CacheError::KeyNotExists)?
            .account_id_internal;
        Ok(data)
    }

    pub async fn read_cache<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&CacheEntry) -> T,
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

    pub async fn write_cache<T, Id: Into<AccountId>>(
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

    // pub async fn account(&self, id: AccountId) -> Result<Account, CacheError> {
    //     let guard = self.accounts.read().await;
    //     let data = guard
    //         .get(&id)
    //         .ok_or(CacheError::KeyNotExists)?
    //         .cache
    //         .read()
    //         .await
    //         .account
    //         .as_ref()
    //         .map(|data| data.as_ref().clone())
    //         .ok_or(CacheError::NotInCache)?;

    //     Ok(data)
    // }

    // pub async fn update_account(&self, id: AccountId, data: Account) -> Result<(), CacheError> {
    //     let mut write_guard = self.accounts.write().await;
    //     write_guard
    //         .get_mut(&id)
    //         .ok_or(CacheError::KeyNotExists)?
    //         .cache
    //         .write()
    //         .await
    //         .account
    //         .as_mut()
    //         .ok_or(CacheError::NotInCache)
    //         .map(|current_data| *current_data.as_mut() = data)?;
    //     Ok(())
    // }
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
    pub capabilities: Capabilities,
    pub shared_state: SharedState,
    pub current_connection: Option<SocketAddr>,
    pub current_event_connection: EventMode,
}

impl CacheEntry {
    pub fn new() -> Self {
        Self {
            profile: None,
            capabilities: Capabilities::default(),
            shared_state: SharedState::default(),
            current_connection: None,
            current_event_connection: EventMode::None,
        }
    }
}

async fn db_read<
    T: FnOnce(CurrentSyncReadCommands<&mut DieselConnection>) -> Result<R, DieselDatabaseError>
        + Send
        + 'static,
    R: Send + 'static,
>(
    read: &CurrentReadHandle,
    cmd: T,
) -> Result<R, CacheError> {
    let conn = read
        .0
        .diesel()
        .pool()
        .get()
        .await
        .change_context(DieselDatabaseError::GetConnection)
        .change_context(CacheError::Init)?;

    conn.interact(move |conn| cmd(CurrentSyncReadCommands::new(conn)))
        .await
        .into_error_string(DieselDatabaseError::Execute)
        .change_context(CacheError::Init)?
        .change_context(CacheError::Init)
}
