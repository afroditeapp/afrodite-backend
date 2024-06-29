use std::{collections::HashMap, fmt::Debug, net::SocketAddr, sync::Arc};

use config::Config;
use error_stack::Result;
use model::{
    AccessToken, AccountId, AccountIdInternal, AccountState, AccountStateRelatedSharedState, Capabilities, LocationIndexKey, LocationIndexProfileData, PendingNotificationFlags, ProfileAttributeFilterValue, ProfileAttributeValue, ProfileContentVersion, ProfileInternal, ProfileQueryMakerDetails, ProfileStateCached, ProfileStateInternal, SortedProfileAttributes
};
pub use server_common::data::cache::CacheError;
use tokio::sync::RwLock;

use super::index::location::LocationIndexIteratorState;
use crate::event::{event_channel, EventReceiver, EventSender};

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

    pub fn accounts(&self) -> &RwLock<HashMap<AccountId, Arc<AccountEntry>>> {
        &self.accounts
    }

    pub fn access_tokens(&self) -> &RwLock<HashMap<AccessToken, Arc<AccountEntry>>> {
        &self.access_tokens
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

    /// Creates new event channel if address is Some.
    pub async fn update_access_token_and_connection(
        &self,
        id: AccountId,
        current_access_token: Option<AccessToken>,
        new_access_token: AccessToken,
        address: Option<SocketAddr>,
    ) -> Result<Option<EventReceiver>, CacheError> {
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
            let event_receiver = if let Some(address) = address {
                let (sender, receiver) = event_channel();
                cache_entry.cache.write().await.current_connection = Some(ConnectionInfo {
                    connection: address,
                    event_sender: sender,
                });
                Ok(Some(receiver))
            } else {
                Ok(None)
            };

            tokens.insert(new_access_token, cache_entry);

            event_receiver
        } else {
            Err(CacheError::AlreadyExists.report())
        }
    }

    /// Delete current connection or specific connection.
    /// Also delete access token if it is Some.
    pub async fn delete_connection_and_specific_access_token(
        &self,
        id: AccountId,
        connection: Option<SocketAddr>,
        token: Option<AccessToken>,
    ) -> Result<(), CacheError> {
        let cache_entry = self
            .accounts
            .read()
            .await
            .get(&id)
            .ok_or(CacheError::KeyNotExists)?
            .clone();

        {
            let mut cache_entry_write = cache_entry.cache.write().await;
            if connection.is_none() ||
                (
                    connection.is_some() &&
                    cache_entry_write.current_connection
                        .as_ref()
                        .map(|info| info.connection) == connection
                )
            {
                cache_entry_write.current_connection = None;
            }
        }

        if let Some(token) = token {
            let mut tokens = self.access_tokens.write().await;
            let _account = tokens.remove(&token).ok_or(CacheError::KeyNotExists)?;
        }

        Ok(())
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
    ) -> Option<(AccountIdInternal, Capabilities, AccountState)> {
        let tokens = self.access_tokens.read().await;
        if let Some(entry) = tokens.get(access_token) {
            let r = entry.cache.read().await;
            if r.current_connection.as_ref().map(|a| a.connection.ip()) == Some(connection.ip()) {
                Some((
                    entry.account_id_internal,
                    r.capabilities.clone(),
                    r.account_state_related_shared_state.account_state_number,
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
        cache_operation(&mut cache_entry)
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
    pub account_id: AccountId,
    pub data: ProfileInternal,
    pub state: ProfileStateCached,
    pub location: LocationData,
    pub attributes: SortedProfileAttributes,
    pub filters: Vec<ProfileAttributeFilterValue>,
}

impl CachedProfile {
    pub fn new(
        account_id: AccountId,
        data: ProfileInternal,
        state: ProfileStateInternal,
        attributes: Vec<ProfileAttributeValue>,
        filters: Vec<ProfileAttributeFilterValue>,
        config: &Config,
    ) -> Self {
        Self {
            account_id,
            data,
            state: state.into(),
            location: LocationData {
                current_position: LocationIndexKey::default(),
                current_iterator: LocationIndexIteratorState::new(),
            },
            attributes: SortedProfileAttributes::new(attributes, config.profile_attributes()),
            filters,
        }
    }

    pub fn filters(&self) -> ProfileQueryMakerDetails {
        ProfileQueryMakerDetails::new(&self.data, &self.state, self.filters.clone())
    }
}

#[derive(Debug, Clone)]
pub struct LocationData {
    pub current_position: LocationIndexKey,
    pub current_iterator: LocationIndexIteratorState,
}

#[derive(Debug, Clone)]
pub struct CachedChatComponentData {
    // This cached version of FcmDeviceToken is now disabled
    // as some extra mapping other way aroud would be needed as
    // same FcmDeviceToken might be used for different account if
    // user logs out and logs in with different account.
    // pub fcm_device_token: Option<FcmDeviceToken>,
}

#[derive(Debug)]
pub struct CachedMedia {
    pub account_id: AccountId,
    pub profile_content_version: ProfileContentVersion,
}

impl CachedMedia {
    pub fn new(
        account_id: AccountId,
        profile_content_version: ProfileContentVersion
    ) -> Self {
        Self {
            account_id,
            profile_content_version,
        }
    }
}

#[derive(Debug)]
pub struct ConnectionInfo {
    pub connection: SocketAddr,
    pub event_sender: EventSender,
}

#[derive(Debug)]
pub struct CacheEntry {
    pub profile: Option<Box<CachedProfile>>,
    pub media: Option<Box<CachedMedia>>,
    pub chat: Option<Box<CachedChatComponentData>>,
    pub capabilities: Capabilities,
    pub account_state_related_shared_state: AccountStateRelatedSharedState,
    pub current_connection: Option<ConnectionInfo>,
    pub pending_notification_flags: PendingNotificationFlags,
}

impl CacheEntry {
    pub fn new() -> Self {
        Self {
            profile: None,
            media: None,
            chat: None,
            capabilities: Capabilities::default(),
            account_state_related_shared_state: AccountStateRelatedSharedState::default(),
            current_connection: None,
            pending_notification_flags: PendingNotificationFlags::empty(),
        }
    }
    // TODO(refactor): Add helper functions to get data related do features
    // that can be disabled. Those should return Result<Data, CacheError>.
    // Also read_cache action closure might need or should to return Result.

    pub fn location_index_profile_data(&self) -> Result<LocationIndexProfileData, CacheError> {
        let profile = self.profile.as_ref().ok_or(CacheError::FeatureNotEnabled)?;

        Ok(LocationIndexProfileData::new(
            profile.account_id,
            &profile.data,
            &profile.state,
            profile.attributes.clone(),
            self.media.as_ref().map(|m| m.profile_content_version),
        ))
    }
}

impl Default for CacheEntry {
    fn default() -> Self {
        Self::new()
    }
}
