use std::{
    collections::{HashMap, hash_map::Entry},
    fmt::Debug,
    net::SocketAddr,
    sync::Arc,
};

use account::CacheAccount;
use chat::CacheChat;
use common::CacheCommon;
use error_stack::Result;
use media::CacheMedia;
use model::{
    AccessToken, AccessTokenUnixTime, AccountId, AccountIdInternal, AccountState,
    IpAddressInternal, PendingNotificationFlags, Permissions, RefreshToken,
};
use model_server_data::{AuthPair, LastSeenUnixTime, LocationIndexProfileData};
use profile::CacheProfile;
pub use server_common::data::cache::CacheError;
use tokio::sync::RwLock;

use crate::{
    db_manager::{InternalReading, InternalWriting},
    event::{EventReceiver, EventSender, event_channel},
};

pub mod account;
pub mod chat;
pub mod common;
pub mod db_iterator;
pub mod media;
pub mod profile;

#[derive(Debug)]
pub struct AccountEntry {
    pub account_id_internal: AccountIdInternal,
    pub cache: RwLock<CacheEntry>,
}

#[derive(Debug, Default)]
pub struct DatabaseCache {
    /// Accounts which are logged in (have valid access token).
    access_tokens: RwLock<HashMap<AccessToken, Arc<AccountEntry>>>,
    /// All accounts registered in the service.
    accounts: RwLock<HashMap<AccountId, Arc<AccountEntry>>>,
}

impl DatabaseCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn load_tokens_from_db_and_return_entry(
        &self,
        account_id: AccountIdInternal,
        access_token: Option<AccessToken>,
        access_token_unix_time: Option<AccessTokenUnixTime>,
        access_token_ip_address: Option<IpAddressInternal>,
        refresh_token: Option<RefreshToken>,
    ) -> Result<Arc<AccountEntry>, CacheError> {
        let read_lock = self.accounts.read().await;
        let account_entry = read_lock
            .get(&account_id.as_id())
            .ok_or(CacheError::KeyNotExists.report())?;

        if let Some(token) = access_token.clone() {
            let mut access_tokens = self.access_tokens.write().await;
            match access_tokens.entry(token) {
                Entry::Vacant(e) => {
                    e.insert(account_entry.clone());
                }
                Entry::Occupied(_) => return Err(CacheError::AlreadyExists.report()),
            }
        }

        let mut write_lock = account_entry.cache.write().await;
        write_lock.common.load_from_db(
            access_token,
            access_token_unix_time,
            access_token_ip_address,
            refresh_token,
        );

        Ok(account_entry.clone())
    }

    pub async fn insert_account_if_not_exists(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), CacheError> {
        let mut data = self.accounts.write().await;
        if data.get(&id.as_id()).is_none() {
            let value = RwLock::new(CacheEntry::default());
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

    pub fn websocket_cache_cmds(&self) -> WebSocketCacheCmds {
        WebSocketCacheCmds { cache: self }
    }

    pub(crate) async fn logout(&self, id: AccountId) -> Result<(), CacheError> {
        let access_token = self
            .write_cache(id, |e| {
                let access_token = e.common.access_token().cloned();
                e.common.logout();
                e.remove_connection();
                Ok(access_token)
            })
            .await?;

        if let Some(access_token) = access_token {
            self.access_tokens.write().await.remove(&access_token);
        }

        Ok(())
    }

    /// Account logout must be done before calling this.
    pub async fn delete_account_which_is_logged_out(&self, id: AccountId) {
        self.accounts.write().await.remove(&id);
    }

    pub async fn access_token_exists(&self, token: &AccessToken) -> Option<AccountIdInternal> {
        let tokens = self.access_tokens.read().await;
        tokens.get(token).map(|entry| entry.account_id_internal)
    }

    /// Checks that connection comes from the same IP address. WebSocket is
    /// using the cached SocketAddr, so check the IP only.
    pub async fn access_token_and_connection_exists(
        &self,
        access_token: &AccessToken,
        connection: SocketAddr,
    ) -> Option<(AccountIdInternal, Permissions, AccountState)> {
        let tokens = self.access_tokens.read().await;
        if let Some(entry) = tokens.get(access_token) {
            let r = entry.cache.read().await;
            if r.common
                .current_connection
                .as_ref()
                .map(|a| a.connection.ip())
                == Some(connection.ip())
            {
                Some((
                    entry.account_id_internal,
                    r.common.permissions.clone(),
                    r.common
                        .account_state_related_shared_state
                        .state_container()
                        .account_state(),
                ))
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
        self.to_account_id_internal_optional(id)
            .await
            .ok_or(CacheError::KeyNotExists.report())
    }

    pub async fn to_account_id_internal_optional(
        &self,
        id: AccountId,
    ) -> Option<AccountIdInternal> {
        let guard = self.accounts.read().await;
        guard.get(&id).map(|e| e.account_id_internal)
    }

    pub async fn logged_in_clients(&self) -> Vec<AccountIdInternal> {
        let guard = self.access_tokens.read().await;
        guard.values().map(|v| v.account_id_internal).collect()
    }

    pub async fn read_cache_for_logged_in_clients(&self, cache_operation: impl Fn(&CacheEntry)) {
        let guard = self.access_tokens.read().await;
        for v in guard.values() {
            let cache_entry = v.cache.read().await;
            cache_operation(&cache_entry)
        }
    }

    pub async fn read_cache_for_all_accounts(
        &self,
        mut cache_operation: impl FnMut(&AccountIdInternal, &CacheEntry) -> Result<(), CacheError>,
    ) -> Result<(), CacheError> {
        let guard = self.accounts.read().await;
        for v in guard.values() {
            let cache_entry = v.cache.read().await;
            cache_operation(&v.account_id_internal, &cache_entry)?
        }

        Ok(())
    }

    pub async fn read_cache<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&CacheEntry) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        let guard = self.accounts.read().await;
        let cache_entry = guard
            .get(&id.into())
            .ok_or(CacheError::KeyNotExists)?
            .cache
            .read()
            .await;
        cache_operation(&cache_entry)
    }

    pub fn read_cache_blocking<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&CacheEntry) -> T,
    ) -> Result<T, CacheError> {
        let guard = self.accounts.blocking_read();
        let cache_entry = guard
            .get(&id.into())
            .ok_or(CacheError::KeyNotExists)?
            .cache
            .blocking_read();
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
        cache_operation(&mut cache_entry)
    }

    pub fn write_cache_blocking<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CacheEntry) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        let guard = self.accounts.blocking_read();
        let mut cache_entry = guard
            .get(&id.into())
            .ok_or(CacheError::KeyNotExists)?
            .cache
            .blocking_write();
        cache_operation(&mut cache_entry)
    }

    pub async fn write_cache_for_logged_in_clients(
        &self,
        cache_operation: impl Fn(AccountIdInternal, &mut CacheEntry),
    ) {
        let guard = self.access_tokens.read().await;
        for v in guard.values() {
            let mut cache_entry = v.cache.write().await;
            cache_operation(v.account_id_internal, &mut cache_entry)
        }
    }

    pub async fn read_cache_common<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&CacheCommon) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        self.read_cache(id, |e| cache_operation(&e.common)).await
    }

    pub async fn read_cache_common_for_logged_in_clients(
        &self,
        cache_operation: impl Fn(&CacheCommon),
    ) {
        self.read_cache_for_logged_in_clients(|e| cache_operation(&e.common))
            .await
    }

    pub async fn write_cache_common<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CacheCommon) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        self.write_cache(id, |e| cache_operation(&mut e.common))
            .await
    }

    pub async fn write_cache_common_for_logged_in_clients(
        &self,
        cache_operation: impl Fn(AccountIdInternal, &mut CacheCommon),
    ) {
        self.write_cache_for_logged_in_clients(|id, e| cache_operation(id, &mut e.common))
            .await
    }
}

pub struct WebSocketCacheCmds<'a> {
    cache: &'a DatabaseCache,
}

impl WebSocketCacheCmds<'_> {
    /// Creates new event channel if `new_event_channel` is true.
    ///
    /// Removes current access token from HashMap containing valid
    /// access tokens.
    ///
    /// This will reset cached pending notification.
    pub async fn init_login_session(
        &self,
        id: AccountId,
        new_tokens: AuthPair,
        address: SocketAddr,
        new_event_channel: bool,
    ) -> Result<Option<EventReceiver>, CacheError> {
        let cache_entry = self
            .cache
            .accounts
            .read()
            .await
            .get(&id)
            .ok_or(CacheError::KeyNotExists)?
            .clone();

        let mut tokens = self.cache.access_tokens.write().await;

        if let Some(current) = cache_entry.cache.read().await.common.access_token() {
            tokens.remove(current);
        }

        // Avoid collisions
        if tokens.get(&new_tokens.access).is_none() {
            let event_receiver = if new_event_channel {
                let (sender, receiver) = event_channel();
                let mut write = cache_entry.cache.write().await;
                write.common.current_connection = Some(ConnectionInfo {
                    connection: address,
                    event_sender: sender,
                });
                if let Some(p) = write.profile.as_ref() {
                    p.last_seen_time().update_last_seen_time_to_online_status();
                }
                Ok(Some(receiver))
            } else {
                Ok(None)
            };

            let new_access_token = new_tokens.access.clone();
            let mut lock = cache_entry.cache.write().await;
            lock.common.update_tokens(new_tokens, address.ip().into());
            lock.common.pending_notification_flags = PendingNotificationFlags::empty();
            drop(lock);
            tokens.insert(new_access_token, cache_entry);

            event_receiver
        } else {
            Err(CacheError::AlreadyExists.report())
        }
    }

    /// This will reset cached pending notification.
    pub async fn init_login_session_using_existing_tokens(
        &self,
        id: AccountId,
        address: SocketAddr,
    ) -> Result<EventReceiver, CacheError> {
        let cache_entry = self
            .cache
            .accounts
            .read()
            .await
            .get(&id)
            .ok_or(CacheError::KeyNotExists)?
            .clone();

        let (sender, event_receiver) = event_channel();
        let mut write = cache_entry.cache.write().await;
        write.common.current_connection = Some(ConnectionInfo {
            connection: address,
            event_sender: sender,
        });
        write.common.pending_notification_flags = PendingNotificationFlags::empty();
        if let Some(p) = write.profile.as_ref() {
            p.last_seen_time().update_last_seen_time_to_online_status();
        }
        Ok(event_receiver)
    }

    pub async fn delete_connection(
        &self,
        id: AccountId,
        connection: SocketAddr,
    ) -> Result<(), CacheError> {
        self.cache
            .write_cache(id, |e| {
                if e.common
                    .current_connection
                    .as_ref()
                    .map(|info| info.connection)
                    == Some(connection)
                {
                    e.remove_connection();
                }
                Ok(())
            })
            .await
    }
}

pub trait CacheReadCommon {
    async fn read_cache_common<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&CacheCommon) -> Result<T, CacheError>,
    ) -> Result<T, CacheError>;

    async fn read_cache_common_for_logged_in_clients(&self, cache_operation: impl Fn(&CacheCommon));
}

impl<R: InternalReading> CacheReadCommon for R {
    async fn read_cache_common<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&CacheCommon) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        self.cache().read_cache_common(id, cache_operation).await
    }

    async fn read_cache_common_for_logged_in_clients(
        &self,
        cache_operation: impl Fn(&CacheCommon),
    ) {
        self.cache()
            .read_cache_common_for_logged_in_clients(cache_operation)
            .await
    }
}

pub trait CacheWriteCommon {
    async fn write_cache_common<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CacheCommon) -> Result<T, CacheError>,
    ) -> Result<T, CacheError>;

    async fn write_cache_for_logged_in_clients(
        &self,
        cache_operation: impl Fn(AccountIdInternal, &mut CacheCommon),
    );
}

impl<I: InternalWriting> CacheWriteCommon for I {
    async fn write_cache_common<T, Id: Into<AccountId>>(
        &self,
        id: Id,
        cache_operation: impl FnOnce(&mut CacheCommon) -> Result<T, CacheError>,
    ) -> Result<T, CacheError> {
        self.cache()
            .write_cache_common(id, |e| cache_operation(e))
            .await
    }

    async fn write_cache_for_logged_in_clients(
        &self,
        cache_operation: impl Fn(AccountIdInternal, &mut CacheCommon),
    ) {
        self.cache()
            .write_cache_for_logged_in_clients(|id, e| cache_operation(id, &mut e.common))
            .await
    }
}

#[derive(Debug)]
pub struct ConnectionInfo {
    pub connection: SocketAddr,
    pub event_sender: EventSender,
}

#[derive(Debug)]
pub struct CacheEntry {
    pub account: Option<Box<CacheAccount>>,
    pub profile: Option<Box<CacheProfile>>,
    pub media: Option<Box<CacheMedia>>,
    pub chat: Option<Box<CacheChat>>,
    pub common: CacheCommon,
}

impl CacheEntry {
    pub fn new() -> Self {
        Self {
            account: None,
            profile: None,
            media: None,
            chat: None,
            common: CacheCommon::default(),
        }
    }

    pub fn account_data(&self) -> Result<&CacheAccount, CacheError> {
        self.account
            .as_ref()
            .map(|v| v.as_ref())
            .ok_or(CacheError::FeatureNotEnabled.report())
    }

    pub fn account_data_mut(&mut self) -> Result<&mut CacheAccount, CacheError> {
        self.account
            .as_mut()
            .map(|v| v.as_mut())
            .ok_or(CacheError::FeatureNotEnabled.report())
    }

    pub fn chat_data(&self) -> Result<&CacheChat, CacheError> {
        self.chat
            .as_ref()
            .map(|v| v.as_ref())
            .ok_or(CacheError::FeatureNotEnabled.report())
    }

    pub fn chat_data_mut(&mut self) -> Result<&mut CacheChat, CacheError> {
        self.chat
            .as_mut()
            .map(|v| v.as_mut())
            .ok_or(CacheError::FeatureNotEnabled.report())
    }

    pub fn profile_data(&self) -> Result<&CacheProfile, CacheError> {
        self.profile
            .as_ref()
            .map(|v| v.as_ref())
            .ok_or(CacheError::FeatureNotEnabled.report())
    }

    pub fn profile_data_mut(&mut self) -> Result<&mut CacheProfile, CacheError> {
        self.profile
            .as_mut()
            .map(|v| v.as_mut())
            .ok_or(CacheError::FeatureNotEnabled.report())
    }

    pub fn media_data(&self) -> Result<&CacheMedia, CacheError> {
        self.media
            .as_ref()
            .map(|v| v.as_ref())
            .ok_or(CacheError::FeatureNotEnabled.report())
    }

    pub fn media_data_mut(&mut self) -> Result<&mut CacheMedia, CacheError> {
        self.media
            .as_mut()
            .map(|v| v.as_mut())
            .ok_or(CacheError::FeatureNotEnabled.report())
    }

    pub fn location_index_profile_data(&self) -> Result<LocationIndexProfileData, CacheError> {
        let profile = self.profile.as_ref().ok_or(CacheError::FeatureNotEnabled)?;

        Ok(LocationIndexProfileData::new(
            profile.account_id,
            profile.profile_internal(),
            &profile.state,
            profile.attributes.clone(),
            self.media.as_ref().map(|m| m.profile_content_version),
            self.common.other_shared_state.unlimited_likes,
            profile.last_seen_time().clone(),
            self.common
                .other_shared_state
                .initial_setup_completed_unix_time,
            self.media.as_ref().map(|m| m.profile_content_edited_time),
            profile.profile_text_character_count(),
        ))
    }

    fn remove_connection(&mut self) {
        self.common.current_connection = None;
        let last_seen_time = LastSeenUnixTime::current_time();
        if let Some(profile_entry) = self.profile.as_mut() {
            profile_entry
                .last_seen_time()
                .update_last_seen_time_to_offline_status(last_seen_time);
        }
    }
}

impl Default for CacheEntry {
    fn default() -> Self {
        Self::new()
    }
}
