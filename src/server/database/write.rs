use std::{fmt::Debug, marker::PhantomData, net::SocketAddr};

use axum::extract::BodyStream;
use error_stack::{Report, Result, ResultExt};

use crate::{
    api::{
        media::data::{HandleModerationRequest, Moderation},
        model::{
            Account, AccountIdInternal, AccountIdLight, AccountSetup, ApiKey, ContentId,
            ModerationRequestContent, ProfileInternal, ProfileLink, Location, AuthPair, SignInWithInfo,
        },
    },
    config::Config,
    server::database::DatabaseError,
    utils::{ConvertCommandError, ErrorConversion},
};

use super::{
    cache::{CacheError, DatabaseCache, WriteCacheJson, CachedProfile},
    current::CurrentDataWriteCommands,
    file::{file::ImageSlot, utils::FileDir},
    history::write::HistoryWriteCommands,
    sqlite::{
        CurrentDataWriteHandle, HistoryUpdateJson, HistoryWriteHandle, SqliteDatabaseError,
        SqliteUpdateJson, SqliteSelectJson,
    }, index::{LocationIndexWriteHandle, LocationIndexIteratorHandle, LocationIndexIteratorGetter, LocationIndexWriterGetter},
};

pub struct NoId;

#[derive(Debug, Clone, Copy)]
pub enum DatabaseId {
    Light(AccountIdLight),
    Internal(AccountIdInternal),
    Content(AccountIdLight, ContentId),
    Empty,
}

impl From<AccountIdLight> for DatabaseId {
    fn from(value: AccountIdLight) -> Self {
        DatabaseId::Light(value)
    }
}

impl From<AccountIdInternal> for DatabaseId {
    fn from(value: AccountIdInternal) -> Self {
        DatabaseId::Internal(value)
    }
}

impl From<(AccountIdLight, ContentId)> for DatabaseId {
    fn from(value: (AccountIdLight, ContentId)) -> Self {
        DatabaseId::Content(value.0, value.1)
    }
}

impl From<NoId> for DatabaseId {
    fn from(_: NoId) -> Self {
        DatabaseId::Empty
    }
}

pub type WriteResult<T, Err, WriteContext = T> =
    std::result::Result<T, WriteError<error_stack::Report<Err>, WriteContext>>;
pub type HistoryWriteResult<T, Err, WriteContext = T> =
    std::result::Result<T, HistoryWriteError<error_stack::Report<Err>, WriteContext>>;

#[derive(Debug)]
pub struct WriteError<Err, Target = ()> {
    pub e: Err,
    pub t: PhantomData<Target>,
}

impl<Target> From<error_stack::Report<SqliteDatabaseError>>
    for WriteError<error_stack::Report<SqliteDatabaseError>, Target>
{
    fn from(value: error_stack::Report<SqliteDatabaseError>) -> Self {
        Self {
            t: PhantomData,
            e: value,
        }
    }
}

impl<Target> From<error_stack::Report<CacheError>>
    for WriteError<error_stack::Report<CacheError>, Target>
{
    fn from(value: error_stack::Report<CacheError>) -> Self {
        Self {
            t: PhantomData,
            e: value,
        }
    }
}

impl<Target> From<CacheError> for WriteError<error_stack::Report<CacheError>, Target> {
    fn from(value: CacheError) -> Self {
        Self {
            t: PhantomData,
            e: value.into(),
        }
    }
}

#[derive(Debug)]
pub struct HistoryWriteError<Err, Target = ()> {
    pub e: Err,
    pub t: PhantomData<Target>,
}

impl<Target> From<error_stack::Report<SqliteDatabaseError>>
    for HistoryWriteError<error_stack::Report<SqliteDatabaseError>, Target>
{
    fn from(value: error_stack::Report<SqliteDatabaseError>) -> Self {
        Self {
            t: PhantomData,
            e: value,
        }
    }
}

// TODO: If one commands does multiple writes to database, move writes to happen
// in a transaction.

// TODO: When server starts, check that latest history data matches with current
// data.

/// One Account can do only one write command at a time.
pub struct AccountWriteLock;

/// Globally synchronous write commands.
pub struct WriteCommands<'a> {
    current_write: &'a CurrentDataWriteHandle,
    history_write: &'a HistoryWriteHandle,
    cache: &'a DatabaseCache,
    file_dir: &'a FileDir,
    location: LocationIndexWriterGetter<'a>,
}

impl<'a> WriteCommands<'a> {
    pub fn new(
        current_write: &'a CurrentDataWriteHandle,
        history_write: &'a HistoryWriteHandle,
        cache: &'a DatabaseCache,
        file_dir: &'a FileDir,
        location: LocationIndexWriterGetter<'a>,
    ) -> Self {
        Self {
            current_write,
            history_write,
            cache,
            file_dir,
            location,
        }
    }

    pub async fn register(
        id_light: AccountIdLight,
        sign_in_with_info: SignInWithInfo,
        config: &Config,
        current_data_write: CurrentDataWriteHandle,
        history_wirte: HistoryWriteHandle,
        cache: &DatabaseCache,
    ) -> Result<AccountIdInternal, DatabaseError> {
        let current = CurrentDataWriteCommands::new(&current_data_write);
        let history = HistoryWriteCommands::new(&history_wirte);

        let account = Account::default();
        let account_setup = AccountSetup::default();

        // TODO: Use transactions here. One for current and other for history.

        let id = current.store_account_id(id_light).await.convert(id_light)?;

        history.store_account_id(id).await.convert(id)?;

        cache.insert_account_if_not_exists(id).await.convert(id)?;

        current.store_api_key(id, None).await.convert(id)?;
        current.store_refresh_token(id, None).await.convert(id)?;

        if config.components().account {
            current.store_account(id, &account).await.convert(id)?;

            history.store_account(id, &account).await.convert(id)?;

            cache
                .write_cache(id.as_light(), |cache| {
                    cache.account = Some(account.clone().into());
                    Ok(())
                })
                .await
                .convert(id)?;

            current
                .store_account_setup(id, &account_setup)
                .await
                .convert(id)?;

            history
                .store_account_setup(id, &account_setup)
                .await
                .convert(id)?;

            current
                .store_sign_in_with_info(id, &sign_in_with_info)
                .await
                .convert(id)?;

        }

        if config.components().profile {
            let profile = current.profile().init_profile(id).await.convert(id)?;

            // TOOD: update history code
            // history
            //     .store_profile(id, &profile)
            //     .await
            //     .with_history_write_cmd_info::<Profile>(id)?;

            cache
                .write_cache(id.as_light(), |cache| {
                    let p: CachedProfile = profile.into();
                    cache.profile = Some(p.into());
                    Ok(())
                })
                .await
                .convert(id)?;
        }

        Ok(id)
    }

    pub async fn set_new_auth_pair(
        &self,
        id: AccountIdInternal,
        pair: AuthPair,
        address: Option<SocketAddr>,
    ) -> Result<(), DatabaseError> {
        let current_access_token = self.current_write.read().access_token(id).await.convert(id)?;

        self.current()
            .update_api_key(id, Some(&pair.access))
            .await
            .convert(id)?;

        self.current()
            .update_refresh_token(id, Some(&pair.refresh))
            .await
            .convert(id)?;

        self.cache
            .update_access_token_and_connection(id.as_light(), current_access_token, pair.access, address)
            .await
            .convert(id)
    }

    /// Remove current connection address, access and refresh tokens.
    pub async fn logout(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DatabaseError> {
        self.current()
            .update_refresh_token(id, None)
            .await
            .convert(id)?;

        self.end_connection_session(id, true).await?;

        Ok(())
    }

    /// Remove current connection address and access token.
    pub async fn end_connection_session(
        &self,
        id: AccountIdInternal,
        remove_access_token: bool,
    ) -> Result<(), DatabaseError> {
        let current_access_token = if remove_access_token {
            self.current_write.read().access_token(id).await.convert(id)?
        } else {
            None
        };

        self.cache
            .delete_access_token_and_connection(id.as_light(), current_access_token)
            .await
            .convert(id)?;

        self.current()
            .update_api_key(id, None)
            .await
            .convert(id)?;

        Ok(())
    }

    pub async fn update_data<
        T: Clone
            + Debug
            + Send
            + SqliteUpdateJson
            + HistoryUpdateJson
            + WriteCacheJson
            + Sync
            + 'static,
    >(
        &mut self,
        id: AccountIdInternal,
        data: &T,
    ) -> Result<(), DatabaseError> {
        data.update_json(id, &self.current())
            .await
            .with_info_lazy(|| format!("Update {:?} failed, id: {:?}", PhantomData::<T>, id))?;

        // Empty implementation if not really cacheable.
        data.write_to_cache(id.as_light(), &self.cache)
            .await
            .with_info_lazy(|| {
                format!("Cache update {:?} failed, id: {:?}", PhantomData::<T>, id)
            })?;

        data.history_update_json(id, &self.history())
            .await
            .with_info_lazy(|| {
                format!("History update {:?} failed, id: {:?}", PhantomData::<T>, id)
            })
    }

    pub async fn set_moderation_request(
        &self,
        account_id: AccountIdInternal,
        request: ModerationRequestContent,
    ) -> Result<(), DatabaseError> {
        self.current()
            .media()
            .create_new_moderation_request(account_id, request)
            .await
            .convert(account_id)
    }

    pub async fn moderation_get_list_and_create_new_if_necessary(
        self,
        account_id: AccountIdInternal,
    ) -> Result<Vec<Moderation>, DatabaseError> {
        self.current()
            .media_admin()
            .moderation_get_list_and_create_new_if_necessary(account_id)
            .await
            .convert(account_id)
    }

    pub async fn update_moderation(
        self,
        moderator_id: AccountIdInternal,
        moderation_request_owner: AccountIdInternal,
        result: HandleModerationRequest,
    ) -> Result<(), DatabaseError> {
        self.current()
            .media_admin()
            .update_moderation(moderator_id, moderation_request_owner, result)
            .await
            .convert(moderator_id)
    }

    /// Completes previous save_to_tmp.
    pub async fn save_to_slot(
        &self,
        id: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    ) -> Result<(), DatabaseError> {
        // Remove previous slot image.
        let current_content_in_slot = self
            .current_write
            .read()
            .media()
            .get_content_id_from_slot(id, slot)
            .await
            .change_context(DatabaseError::Sqlite)?;
        if let Some(current_id) = current_content_in_slot {
            let path = self
                .file_dir
                .image_content(id.as_light(), current_id.as_content_id());
            path.remove_if_exists()
                .await
                .change_context(DatabaseError::File)?;
            self.current()
                .media()
                .delete_image_from_slot(id, slot)
                .await
                .change_context(DatabaseError::Sqlite)?;
        }

        let transaction = self
            .current()
            .media()
            .store_content_id_to_slot(id, content_id, slot)
            .await
            .change_context(DatabaseError::Sqlite)?;

        let file_operations = || {
            async {
                // Move image from tmp to image dir
                let raw_img = self
                    .file_dir
                    .unprocessed_image_upload(id.as_light(), content_id);
                let processed_content_path = self.file_dir.image_content(id.as_light(), content_id);
                raw_img
                    .move_to(&processed_content_path)
                    .await
                    .change_context(DatabaseError::File)?;

                Ok::<(), Report<DatabaseError>>(())
            }
        };

        match file_operations().await {
            Ok(()) => transaction
                .commit()
                .await
                .change_context(DatabaseError::Sqlite),
            Err(e) => {
                match transaction
                    .rollback()
                    .await
                    .change_context(DatabaseError::Sqlite)
                {
                    Ok(()) => Err(e),
                    Err(another_error) => Err(another_error.attach(e)),
                }
            }
        }
    }

    pub async fn profile_update_visibility(
        self,
        id: AccountIdInternal,
        public: bool,
        update_only_if_no_value: bool,
    ) -> Result<(), DatabaseError> {
        let (visiblity, location, profile_link) = self.cache.write_cache(id.as_light(), |e| {
            let p = e.profile.as_mut().ok_or(CacheError::InitFeatureNotEnabled)?;

            // Handle race condition between remote fetch and update.
            // Update will override the initial fetch.
            if update_only_if_no_value {
                if p.public.is_none() {
                    p.public = Some(public);
                }
            } else {
                p.public = Some(public);
            }

            Ok((p.public.unwrap_or_default(), p.location.current_position, ProfileLink::new(id.as_light(), &p.data)))
        }).await.convert(id)?;

        let index = self.location.get().ok_or(DatabaseError::FeatureDisabled)?;
        if visiblity {
            index.update_profile_link(id.as_light(), profile_link, location).await;
        } else {
            index.remove_profile_link(id.as_light(), location).await;
        }

        Ok(())
    }

    pub async fn profile_update_location(
        self,
        id: AccountIdInternal,
        coordinates: Location,
    ) -> Result<(), DatabaseError> {
        let location = self.cache
            .read_cache(id.as_light(), |e|
            {
                e.profile.as_ref().map(|p| p.location.clone()) }
            ).await
            .convert(id)?
            .ok_or(DatabaseError::FeatureDisabled)?;

        let write_to_index = self.location.get().ok_or(DatabaseError::FeatureDisabled)?;
        let new_location_key = write_to_index.coordinates_to_key(coordinates);

        // TODO: Create new database table for location.
        // TODO: Also update history?
        new_location_key.update_json(id, &self.current()).await.change_context(DatabaseError::Sqlite)?;
        write_to_index.update_profile_location(id.as_light(), location.current_position, new_location_key).await;

        Ok(())
    }

    fn current(&self) -> CurrentDataWriteCommands {
        CurrentDataWriteCommands::new(&self.current_write)
    }

    fn history(&self) -> HistoryWriteCommands {
        HistoryWriteCommands::new(&self.history_write)
    }
}

/// Commands that can run concurrently with other write commands, but which have
/// limitation that one account can execute only one command at a time.
/// It possible to run this and normal write command concurrently for
/// one account.
pub struct WriteCommandsAccount<'a> {
    current_write: &'a CurrentDataWriteHandle,
    history_write: &'a HistoryWriteHandle,
    cache: &'a DatabaseCache,
    file_dir: &'a FileDir,
    location: LocationIndexIteratorGetter<'a>,
}

impl<'a> WriteCommandsAccount<'a> {
    pub fn new(
        current_write: &'a CurrentDataWriteHandle,
        history_write: &'a HistoryWriteHandle,
        cache: &'a DatabaseCache,
        file_dir: &'a FileDir,
        location: LocationIndexIteratorGetter<'a>,
    ) -> Self {
        Self {
            current_write,
            history_write,
            cache,
            file_dir,
            location,
        }
    }

    pub async fn save_to_tmp(
        &self,
        id: AccountIdInternal,
        stream: BodyStream,
    ) -> Result<ContentId, DatabaseError> {
        let content_id = ContentId::new_random_id();

        // Clear tmp dir if previous image writing failed and there is no
        // content ID in the database about it.
        self.file_dir
            .tmp_dir(id.as_light())
            .remove_contents_if_exists()
            .await
            .change_context(DatabaseError::File)?;

        let raw_img = self
            .file_dir
            .unprocessed_image_upload(id.as_light(), content_id);
        raw_img
            .save_stream(stream)
            .await
            .change_context(DatabaseError::File)?;

        // TODO: image safety checks and processing

        Ok(content_id)
    }

    pub async fn next_profiles(
        &self,
        id: AccountIdInternal,
    ) -> Result<Vec<ProfileLink>, DatabaseError> {
        let location = self.cache.read_cache(id.as_light(), |e| e.profile.as_ref().map(|p| p.location.clone())).await
            .convert(id)?
            .ok_or(DatabaseError::FeatureDisabled)?;

        let iterator = self.location.get().ok_or(DatabaseError::FeatureDisabled)?;
        let (next_state, profiles) = iterator.next_profiles(location.current_iterator).await;
        self.cache.write_cache(id.as_light(), |e|
        {
            e.profile.as_mut().map(move |p| p.location.current_iterator = next_state);
            Ok(())
        }).await.convert(id)?;

        Ok(profiles.unwrap_or(Vec::new()))
    }

    pub async fn reset_profile_iterator(
        &self,
        id: AccountIdInternal,
    ) -> Result<(), DatabaseError> {
        let location = self.cache.read_cache(id.as_light(), |e| e.profile.as_ref().map(|p| p.location.clone())).await
            .convert(id)?
            .ok_or(DatabaseError::FeatureDisabled)?;

        let iterator = self.location.get().ok_or(DatabaseError::FeatureDisabled)?;
        let next_state = iterator.reset_iterator(location.current_iterator, location.current_position);
        self.cache.write_cache(id.as_light(), |e| {
            e.profile.as_mut().map(move |p| p.location.current_iterator = next_state);
            Ok(())
        }).await.convert(id)?;
        Ok(())
    }

    fn current(&self) -> CurrentDataWriteCommands {
        CurrentDataWriteCommands::new(&self.current_write)
    }

    fn history(&self) -> HistoryWriteCommands {
        HistoryWriteCommands::new(&self.history_write)
    }
}
